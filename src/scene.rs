use indexmap::IndexMap;
use log::info;
use sea_orm::prelude::*;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
use std::path::Path;
use std::process::Command;
use std::rc::Rc;

use crate::db::SceneEntity;
use crate::db::SceneObjectEntity;
use crate::db::SceneObjectModel;
use crate::elimination::Elimination;
use crate::fint::FInt;
use crate::poly::PolyConversion;
use crate::poly::{Poly, PolyOperations, SingleOutResult};
use crate::poly_draw::{Color, XYPolyDraw};
use crate::scene_object::{ObjectType, SceneError, SceneObject};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    pub center: Center,
    pub diagonal: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Center {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
pub struct Plot {
    pub name: String,
    pub x: String,
    pub y: String,
}

#[derive(Debug)]
pub struct Scene {
    pub id: i32,
    pub objects: IndexMap<String, SceneObject>,
    pub view: View,
}

impl Scene {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            objects: IndexMap::new(),
            view: View {
                center: Center { x: 0.0, y: 0.0 },
                diagonal: 25.0,
            },
        }
    }

    pub async fn add_object(
        &mut self,
        db: &DatabaseConnection,
        name: String,
        object_type: ObjectType,
        properties: Value,
    ) -> Result<(), SceneError> {
        let scene_object = SceneObject::from_properties(object_type, properties.clone())?;

        for dependency in scene_object.get_dependencies() {
            if !self.objects.contains_key(&dependency) {
                return Err(SceneError::DependencyNotFound(dependency));
            }
        }

        // Save to database
        SceneObjectModel::save_object(db, self.id, &name, object_type, properties).await?;

        // Save to memory
        self.objects.insert(name, scene_object);
        Ok(())
    }

    pub async fn delete_object(
        &mut self,
        db: &DatabaseConnection,
        name: &str,
    ) -> Result<Vec<String>, SceneError> {
        // Collect all objects that should be deleted due to dependencies
        let mut objects_to_delete = self.collect_dependent_objects(name);

        // Delete all dependent objects from database in a single call
        SceneObjectModel::delete_objects(db, self.id, &objects_to_delete).await?;

        // Delete all objects from memory
        for obj_name in &objects_to_delete {
            self.objects.shift_remove(obj_name);
        }

        // Remove the target object from the list (we'll handle it separately)
        objects_to_delete.retain(|obj_name| obj_name != name);

        Ok(objects_to_delete)
    }

    pub async fn delete_scene(&mut self, db: &DatabaseConnection) -> Result<(), SceneError> {
        // Delete all scene objects from database (cascade will handle this automatically)
        // But we'll also delete them explicitly to be sure
        SceneObjectModel::delete_objects(
            db,
            self.id,
            &self.objects.keys().cloned().collect::<Vec<_>>(),
        )
        .await?;

        // Delete the scene from database
        SceneEntity::delete_by_id(self.id)
            .exec(db)
            .await
            .map_err(|e| SceneError::DatabaseError(e.to_string()))?;

        // Clear objects from memory
        self.objects.clear();

        Ok(())
    }

    /// Recursively collect all objects that depend on the given object
    pub fn collect_dependent_objects(&self, target_name: &str) -> Vec<String> {
        let mut to_delete = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with the target object
        queue.push_back(target_name.to_string());

        while let Some(current_name) = queue.pop_front() {
            if to_delete.contains(&current_name) {
                continue; // Already processed
            }

            to_delete.insert(current_name.clone());

            // Find all objects that depend on the current object
            for (obj_name, obj) in &self.objects {
                if !to_delete.contains(obj_name) {
                    let dependencies = obj.get_dependencies();
                    if dependencies.contains(&current_name) {
                        queue.push_back(obj_name.clone());
                    }
                }
            }
        }

        to_delete.into_iter().collect()
    }

    pub async fn load_objects_and_view(
        &mut self,
        db: &DatabaseConnection,
    ) -> Result<(), SceneError> {
        let db_scene_objects = SceneObjectEntity::find()
            .filter(crate::db::SceneObjectColumn::SceneId.eq(self.id))
            .all(db)
            .await
            .map_err(|e| SceneError::DatabaseError(e.to_string()))?;

        self.objects.clear();
        for db_scene_object in db_scene_objects {
            let scene_object = db_scene_object.get_scene_object()?;
            self.objects
                .insert(db_scene_object.object_name, scene_object);
        }
        self.view = self.get_view(db).await?;
        Ok(())
    }

    pub async fn get_view(&self, db: &DatabaseConnection) -> Result<View, SceneError> {
        let scene_model = SceneEntity::find_by_id(self.id)
            .one(db)
            .await
            .map_err(|e| SceneError::DatabaseError(e.to_string()))?
            .ok_or_else(|| SceneError::DatabaseError("Scene not found".to_string()))?;

        let view: View = serde_json::from_str(&scene_model.view)
            .map_err(|e| SceneError::DatabaseError(format!("Failed to parse view JSON: {}", e)))?;

        Ok(view)
    }

    pub fn to_python(&self) -> String {
        self.objects
            .iter()
            .map(|(name, obj)| obj.to_python(name))
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn to_equations(&self) -> Result<(Vec<String>, Vec<Plot>), SceneError> {
        let python_code = format!(
            "from equation_processor import *\n{}\n\n# Print all equations\nfor eq in equations:\n    print(eq)\nprint()\n# Print all plots\nfor plot in plots:\n    print(plot)",
            self.to_python()
        );
        info!("Python code: {}", python_code);

        let py_dir = Path::new("src/py");
        let output = Command::new("python3")
            .current_dir(py_dir)
            .arg("-c")
            .arg(python_code)
            .output()
            .map_err(|e| SceneError::DatabaseError(format!("Failed to run Python: {}", e)))?;
        println!("output: {:?}", output);

        if !output.status.success() {
            return Err(SceneError::DatabaseError(format!(
                "Python execution failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut lines = output_str.lines();

        // Collect equations until we hit an empty line
        let mut equations = Vec::new();
        while let Some(line) = lines.next() {
            if line.is_empty() {
                break;
            }
            equations.push(line.to_string());
        }

        // Collect plots
        let mut plots = Vec::new();
        while let Some(line) = lines.next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                plots.push(Plot {
                    name: parts[0].to_string(),
                    x: parts[1].to_string(),
                    y: parts[2].to_string(),
                });
            }
        }

        Ok((equations, plots))
    }

    pub fn parse_plot_vars(plot: &Plot) -> Result<(u8, u8), SceneError> {
        let x_var =
            Poly::parse_var(&plot.x).map_err(|e| SceneError::InvalidEquation(e.to_string()))?;
        let y_var =
            Poly::parse_var(&plot.y).map_err(|e| SceneError::InvalidEquation(e.to_string()))?;
        Ok((x_var, y_var))
    }

    pub fn get_curve_equation(equations: Vec<&str>, plot: &Plot) -> Result<Poly, SceneError> {
        // Convert equations to polynomials
        let mut polys: Vec<Rc<Poly>> = equations
            .into_iter()
            .map(|s| {
                Rc::new(
                    Poly::new(s)
                        .map_err(|e| SceneError::InvalidEquation(e.to_string()))
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();

        // Convert x and y to variable indices
        let (x_var, y_var) = Self::parse_plot_vars(plot)?;

        // Collect all variables used in polynomials
        let mut vars = [false; 256];
        for poly in &polys {
            poly.fill_in_variables(&mut vars);
        }

        // Process each variable that's not x or y
        for (v, has_var) in vars.iter().enumerate() {
            if *has_var && v != x_var as usize && v != y_var as usize {
                // Get single_out results for all polynomials
                let results: Vec<SingleOutResult> =
                    polys.iter().map(|p| p.single_out(v as u8)).collect();

                // Find a linear result to use for substitution
                let mut linear_idx = None;
                let mut linear_poly = None;
                let mut linear_k = 0;

                for (i, result) in results.iter().enumerate() {
                    if let SingleOutResult::Linear(p, k) = result {
                        linear_idx = Some(i);
                        linear_poly = Some(p.clone());
                        linear_k = *k;
                        break;
                    }
                }

                // If we found a linear result, use it to substitute in other polynomials
                if let (Some(idx), Some(poly)) = (linear_idx, linear_poly) {
                    let mut new_polys = Vec::new();
                    for (i, result) in results.iter().enumerate() {
                        if i == idx {
                            continue; // Skip the polynomial we used for substitution
                        }
                        match result {
                            SingleOutResult::Constant => {
                                new_polys.push(polys[i].clone());
                            }
                            SingleOutResult::Linear(_, _) | SingleOutResult::Nonlinear => {
                                new_polys.push(Rc::new(polys[i].substitute_linear(
                                    v as u8,
                                    poly.clone(),
                                    linear_k,
                                )));
                            }
                        }
                    }
                    polys = new_polys;
                }
            }
        }
        polys = Poly::retain_relevant_polys(polys, x_var, y_var);
        info!(
            "Reduced system: \n{}",
            polys
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join("\n")
        );

        let mut elimination = Elimination::new(&polys, x_var, y_var);
        loop {
            match elimination.get_var_to_eliminate() {
                Some(var_search_result) => {
                    info!(
                        "--- Eliminating variable {} from\n{}",
                        Poly::var_to_string(var_search_result.var),
                        elimination
                            .polys
                            .iter()
                            .map(|p| p.to_string())
                            .collect::<Vec<String>>()
                            .join("\n")
                    );
                    elimination.eliminate_var(var_search_result);
                }
                None => break,
            }
        }
        let polys = elimination.polys.clone();

        // Check if we have exactly one polynomial left
        if polys.len() != 1 {
            return Err(SceneError::InvalidEquation(format!(
                "Expected exactly one equation after elimination, got {}",
                polys.len()
            )));
        }

        // Verify the remaining polynomial only depends on x and y
        let mut vars = [false; 256];
        polys[0].fill_in_variables(&mut vars);
        for (v, has_var) in vars.iter().enumerate() {
            if *has_var && v != x_var as usize && v != y_var as usize {
                return Err(SceneError::InvalidEquation(format!(
                    "Remaining equation depends on variable {}",
                    (v as u8 + b'a') as char
                )));
            }
        }
        let mut result = polys[0].clone();
        Rc::make_mut(&mut result).reduce_coefficients_if_above(1);
        let factors = result
            .factor()
            .map_err(|e| SceneError::InvalidEquation(e))?;
        if factors.len() > 1 {
            info!(
                "Factors: {}",
                factors
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            );
            let mut product = Poly::Constant(1);
            for factor in factors {
                if elimination
                    .check_factor(&factor)
                    .map_err(|e| SceneError::InvalidEquation(e))?
                {
                    product = product.multiply(&factor);
                } else {
                    info!("Skipping factor {}", factor);
                }
            }
            info!("Product: {}", product);
            result = Rc::new(product);
        }

        Ok((*result).clone())
    }

    pub fn solve_and_plot(
        &self,
        locus_name: &str,
        width: u32,
        height: u32,
    ) -> Result<Vec<(u32, u32, Color)>, SceneError> {
        // Convert plot to equations
        let (equations, plots) = self.to_equations()?;
        info!(
            "Found {} equations and {} plots",
            equations.len(),
            plots.len()
        );
        let plot = plots.iter().find(|p| p.name == locus_name).unwrap();

        // Get curve equation
        let curve_equation =
            Scene::get_curve_equation(equations.iter().map(|s| s.as_str()).collect(), plot)
                .map_err(|e| SceneError::InvalidEquation(e.to_string()))?;

        info!("Curve equation: {}", curve_equation);

        let (x_var, y_var) = Self::parse_plot_vars(plot)?;
        // Convert to XYPoly
        let xy_poly = curve_equation
            .as_xy_poly(x_var, y_var)
            .map_err(|e| SceneError::InvalidEquation(e.to_string()))?;
        info!("XYPoly: {}", xy_poly);

        // Create drawer
        let drawer = XYPolyDraw::new(xy_poly);

        // Logical bounds: wl and hl, with wl^2 + hl^2 = diagonal^2 and hl / wl = height / width = ratio
        // wl = diagonal * sqrt(1 / (1 + ratio^2))
        // hl = wl * ratio
        let ratio = height as f64 / width as f64;
        let wl = self.view.diagonal * (1.0 / (1.0 + ratio * ratio)).sqrt();
        let hl = ratio * wl;
        info!(
            "Logical bounds: wl: {}, hl: {} for width = {} and height = {}",
            wl, hl, width, height
        );

        let points = drawer.get_curve_points(
            FInt::new_with_bounds(self.view.center.x - 0.5 * wl, self.view.center.x + 0.5 * wl),
            FInt::new_with_bounds(self.view.center.y - 0.5 * hl, self.view.center.y + 0.5 * hl),
            width * 4,
            height * 4,
        );
        info!("Points: {}", points.len());

        // Get curve points
        let points = drawer.get_curve_points_smoothed(points, width * 4, height * 4);
        info!("Smoothed points: {}", points.len());
        Ok(points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{SceneActiveModel, SceneEntity, SceneObjectEntity};
    use crate::service::{config, AppState, CreateSceneRequest};
    use sea_orm::ActiveValue::Set;
    use sea_orm::{Database, Schema};
    use serde_json::json;

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();

        // Create tables
        let schema = Schema::new(db.get_database_backend());
        let stmt = schema.create_table_from_entity(SceneEntity);
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .unwrap();
        let stmt = schema.create_table_from_entity(SceneObjectEntity);
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .unwrap();
        let scene = SceneActiveModel {
            id: Set(1),
            name: Set("Test Scene".to_string()),
            ..Default::default()
        };
        scene.insert(&db).await.unwrap();

        // Debug: Show table structure
        // let result = JsonValue::find_by_statement(Statement::from_sql_and_values(
        //     DbBackend::Postgres,
        //     "SELECT sql FROM sqlite_master WHERE type='table' AND name='scene_objects'",
        //     [],
        // ))
        // .all(&db)
        // .await
        // .unwrap();
        // println!("Table structure: {:?}", result);

        db
    }

    #[tokio::test]
    async fn test_scene_operations() {
        let db = setup_test_db().await;
        let scene_id = SceneEntity::find().one(&db).await.unwrap().unwrap().id;
        let mut scene = Scene::new(scene_id);

        // Test adding objects
        let point_props = json!({
            "value": "10, 20"
        });
        scene
            .add_object(&db, "P1".to_string(), ObjectType::FixedPoint, point_props)
            .await
            .unwrap();

        let obj = SceneObjectEntity::find()
            .filter(crate::db::SceneObjectColumn::SceneId.eq(scene_id))
            .filter(crate::db::SceneObjectColumn::ObjectName.eq("P1"))
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(obj.object_name, "P1");

        // Test adding objects
        let point_props = json!({
            "value": "30, 40"
        });
        scene
            .add_object(&db, "P2".to_string(), ObjectType::FixedPoint, point_props)
            .await
            .unwrap();

        let line_props = json!({
            "point1": "P1",
            "point2": "P2"
        });
        scene
            .add_object(&db, "L1".to_string(), ObjectType::LineAB, line_props)
            .await
            .unwrap();

        // Test loading objects
        scene.load_objects_and_view(&db).await.unwrap();
        assert_eq!(scene.objects.len(), 3);
        assert!(matches!(
            scene.objects.get("P1"),
            Some(SceneObject::FixedPoint(_))
        ));
        assert!(matches!(
            scene.objects.get("L1"),
            Some(SceneObject::LineAB(_))
        ));

        // Test deleting object
        scene.delete_object(&db, "P1").await.unwrap();
        assert_eq!(scene.objects.len(), 1);
        assert!(scene.objects.get("P1").is_none());

        // Verify deletion in database
        scene.load_objects_and_view(&db).await.unwrap();
        assert_eq!(scene.objects.len(), 1);
        assert!(scene.objects.get("P1").is_none());
        assert!(scene.objects.get("P2").is_some());
    }

    #[tokio::test]
    async fn test_recursive_dependency_deletion() {
        let db = setup_test_db().await;
        let scene_id = SceneEntity::find().one(&db).await.unwrap().unwrap().id;
        let mut scene = Scene::new(scene_id);

        // Create a chain of dependencies: P1 -> L1 -> I1 -> L2
        // P1 (FixedPoint)
        scene
            .add_object(
                &db,
                "P1".to_string(),
                ObjectType::FixedPoint,
                json!({"value": "10, 20"}),
            )
            .await
            .unwrap();

        // P2 (FixedPoint) - needed for LineAB
        scene
            .add_object(
                &db,
                "P2".to_string(),
                ObjectType::FixedPoint,
                json!({"value": "30, 40"}),
            )
            .await
            .unwrap();

        // P3 (FixedPoint) - needed for Invariant
        scene
            .add_object(
                &db,
                "P3".to_string(),
                ObjectType::FixedPoint,
                json!({"value": "50, 60"}),
            )
            .await
            .unwrap();

        // L1 (LineAB) depends on P1
        scene
            .add_object(
                &db,
                "L1".to_string(),
                ObjectType::LineAB,
                json!({"point1": "P1", "point2": "P2"}),
            )
            .await
            .unwrap();

        // I1 (Invariant) depends on L1
        scene
            .add_object(
                &db,
                "I1".to_string(),
                ObjectType::Invariant,
                json!({"formula": "d(L1, P3)"}),
            )
            .await
            .unwrap();

        // L2 (Locus) depends on I1
        scene
            .add_object(
                &db,
                "L2".to_string(),
                ObjectType::Locus,
                json!({"point": "I1"}),
            )
            .await
            .unwrap();

        // Load objects to populate the scene
        scene.load_objects_and_view(&db).await.unwrap();
        assert_eq!(scene.objects.len(), 6);

        // Test collect_dependent_objects returns correct values
        let dependents = scene.collect_dependent_objects("P1");
        println!("dependents for P1: {:?}", dependents);
        let expected: std::collections::HashSet<_> = ["P1", "L1", "I1", "L2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let actual: std::collections::HashSet<_> = dependents.iter().cloned().collect();
        assert_eq!(actual, expected);

        // Test collect_dependent_objects for L1
        let dependents_l1 = scene.collect_dependent_objects("L1");
        println!("dependents for L1: {:?}", dependents_l1);
        let expected_l1: std::collections::HashSet<_> =
            ["L1", "I1", "L2"].iter().map(|s| s.to_string()).collect();
        let actual_l1: std::collections::HashSet<_> = dependents_l1.iter().cloned().collect();
        assert_eq!(actual_l1, expected_l1);

        // Test collect_dependent_objects for P2 (should return P2, L1, I1, L2)
        let dependents_p2 = scene.collect_dependent_objects("P2");
        println!("dependents for P2: {:?}", dependents_p2);
        let expected_p2: std::collections::HashSet<_> = ["P2", "L1", "I1", "L2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let actual_p2: std::collections::HashSet<_> = dependents_p2.iter().cloned().collect();
        assert_eq!(actual_p2, expected_p2);

        // Delete P1 - should cascade to delete L1, I1, and L2
        let deleted_deps = scene.delete_object(&db, "P1").await.unwrap();
        assert_eq!(deleted_deps.len(), 3);
        assert!(deleted_deps.contains(&"L1".to_string()));
        assert!(deleted_deps.contains(&"I1".to_string()));
        assert!(deleted_deps.contains(&"L2".to_string()));
        assert_eq!(scene.objects.len(), 2); // P2 and P3 should remain

        // Verify in database
        scene.load_objects_and_view(&db).await.unwrap();
        assert_eq!(scene.objects.len(), 2);
        assert!(scene.objects.contains_key("P2"));
        assert!(scene.objects.contains_key("P3"));
    }

    #[tokio::test]
    async fn test_python_generation() {
        let db = setup_test_db().await;
        let scene_id = SceneEntity::find().one(&db).await.unwrap().unwrap().id;
        let mut scene = Scene::new(scene_id);

        // Add some objects
        let point_props = json!({
            "value": "10, 20"
        });
        scene
            .add_object(&db, "P1".to_string(), ObjectType::FixedPoint, point_props)
            .await
            .unwrap();

        let free_props = json!({
            "value": "30, 40"
        });
        scene
            .add_object(&db, "P2".to_string(), ObjectType::FreePoint, free_props)
            .await
            .unwrap();

        let line_props = json!({
            "point1": "P1",
            "point2": "0, 1"
        });
        scene
            .add_object(&db, "L1".to_string(), ObjectType::LineAB, line_props)
            .await
            .unwrap();

        let inv_props = json!({
            "formula": "d(P1, P2)"
        });
        scene
            .add_object(&db, "I1".to_string(), ObjectType::Invariant, inv_props)
            .await
            .unwrap();

        let expected = r#"P1 = FixedPoint(10, 20)
P2 = FreePoint(30, 40)
L1 = LineAB(P1, FixedPoint(0, 1))
is_constant(d(P1, P2))"#;
        assert_eq!(scene.to_python(), expected);
    }

    #[tokio::test]
    async fn test_equations_and_plots_generation() {
        let db = setup_test_db().await;
        let scene_id = SceneEntity::find().one(&db).await.unwrap().unwrap().id;
        let mut scene = Scene::new(scene_id);

        // Add fixed point A at (0,0)
        let point_props = json!({
            "value": "0, 0"
        });
        scene
            .add_object(&db, "A".to_string(), ObjectType::FixedPoint, point_props)
            .await
            .unwrap();

        // Add free point X at (3,4)
        let free_props = json!({
            "value": "3, 4"
        });
        scene
            .add_object(&db, "X".to_string(), ObjectType::FreePoint, free_props)
            .await
            .unwrap();

        // Add plot for X
        let locus_props = json!({
            "point": "X"
        });
        scene
            .add_object(&db, "P1".to_string(), ObjectType::Locus, locus_props)
            .await
            .unwrap();

        // Add invariant d_sqr(A, X)
        let inv_props = json!({
            "formula": "d_sqr(A, X)"
        });
        scene
            .add_object(&db, "I1".to_string(), ObjectType::Invariant, inv_props)
            .await
            .unwrap();

        let (equations, plots) = scene.to_equations().unwrap();

        // The equations should contain specific formulas for the distance
        let expected_equations = [
            "0 - a - c",
            "c^2 - d",
            "0 - b - e",
            "e^2 - f",
            "d + f - g",
            "g - 25",
        ];

        for eq in expected_equations.iter() {
            assert!(
                equations.contains(&eq.to_string()),
                "Expected equation '{}' not found in equations: {:?}",
                eq,
                equations
            );
        }

        // Check plots
        assert_eq!(plots.len(), 1);
        assert_eq!(plots[0].x, "a");
        assert_eq!(plots[0].y, "b");
    }

    #[test]
    fn test_get_curve_equation() {
        let poly = Scene::get_curve_equation(
            vec!["a^2 - 2*c", "b^2 - 3*d", "d - c"],
            &Plot {
                name: "plotA".to_string(),
                x: "a".to_string(),
                y: "b".to_string(),
            },
        )
        .unwrap();
        assert_eq!(format!("{}", poly), "2*b^2 - 3*a^2");
    }

    #[tokio::test]
    async fn test_create_scene_via_rest() {
        use actix_web::{test, web, App};

        // Setup test database
        let db = setup_test_db().await;
        // Create app state
        let app_state = AppState::new(db).await;

        // Create test app
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .configure(config),
        )
        .await;

        // Test creating scene with name
        let create_request = CreateSceneRequest {
            name: Some("My Test Scene".to_string()),
        };
        let req = test::TestRequest::post()
            .uri("/scenes")
            .set_json(&create_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("id").is_some());
        assert_eq!(body.get("name").unwrap().as_str().unwrap(), "My Test Scene");

        // Test creating scene without name (should get default name)
        let create_request = CreateSceneRequest { name: None };
        let req = test::TestRequest::post()
            .uri("/scenes")
            .set_json(&create_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        let scene_id = body.get("id").unwrap().as_i64().unwrap();
        let scene_name = body.get("name").unwrap().as_str().unwrap();
        assert_eq!(scene_name, format!("Scene {}", scene_id));

        // Test creating scene with empty name (should get default name)
        let create_request = CreateSceneRequest {
            name: Some("".to_string()),
        };
        let req = test::TestRequest::post()
            .uri("/scenes")
            .set_json(&create_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        let scene_id = body.get("id").unwrap().as_i64().unwrap();
        let scene_name = body.get("name").unwrap().as_str().unwrap();
        assert_eq!(scene_name, format!("Scene {}", scene_id));
    }

    #[tokio::test]
    async fn test_delete_scene() {
        let db = setup_test_db().await;

        // Create another scene
        let scene2 = SceneActiveModel {
            id: Set(2),
            name: Set("Scene to Delete".to_string()),
            ..Default::default()
        };
        let scene2 = scene2.insert(&db).await.unwrap();

        // Create a scene object in the scene to be deleted
        let point_props = json!({
            "value": "10, 20"
        });
        SceneObjectModel::save_object(&db, scene2.id, "P1", ObjectType::FixedPoint, point_props)
            .await
            .unwrap();

        // Verify the scene and its object exist
        let scene_exists = SceneEntity::find_by_id(scene2.id)
            .one(&db)
            .await
            .unwrap()
            .is_some();
        assert!(scene_exists);

        let object_exists = SceneObjectEntity::find()
            .filter(crate::db::SceneObjectColumn::SceneId.eq(scene2.id))
            .one(&db)
            .await
            .unwrap()
            .is_some();
        assert!(object_exists);

        // Load the scene and delete it
        let mut scene = Scene::new(scene2.id);
        scene.load_objects_and_view(&db).await.unwrap();

        // Verify scene has objects before deletion
        assert_eq!(scene.objects.len(), 1);
        assert!(scene.objects.contains_key("P1"));

        // Delete the scene
        scene.delete_scene(&db).await.unwrap();

        // Verify scene is cleared in memory
        assert_eq!(scene.objects.len(), 0);

        // Verify scene and its objects are deleted from database
        let scene_exists = SceneEntity::find_by_id(scene2.id)
            .one(&db)
            .await
            .unwrap()
            .is_none();
        assert!(scene_exists);

        let object_exists = SceneObjectEntity::find()
            .filter(crate::db::SceneObjectColumn::SceneId.eq(scene2.id))
            .one(&db)
            .await
            .unwrap()
            .is_none();
        assert!(object_exists);
    }
}
