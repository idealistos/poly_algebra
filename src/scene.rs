use indexmap::IndexMap;
use log::info;
use sea_orm::prelude::*;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
use std::path::Path;
use std::process::Command;

use crate::db::SceneEntity;
use crate::db::SceneObjectEntity;
use crate::db::SceneObjectModel;
use crate::fint::FInt;
use crate::poly::Poly;
use crate::poly::PolyConversion;
use crate::poly_draw::{Color, XYPolyDraw};
use crate::scene_object::{ObjectType, SceneError, SceneObject};
use crate::scene_utils::SceneUtils;

#[derive(Debug)]
pub struct PlotData {
    pub points: Vec<(u32, u32, Color)>,
    pub equation: String,
    pub formatted_equations: Vec<String>,
}

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
pub struct CurveEquationAndFactors {
    pub curve_equation: Poly,
    pub factors: Vec<Poly>,
}

#[derive(Debug, Clone)]
pub struct SceneOptions {
    pub reduce_factors: bool,
}

impl Default for SceneOptions {
    fn default() -> Self {
        Self {
            reduce_factors: false,
        }
    }
}

impl SceneOptions {
    pub fn new(reduce_factors: bool) -> Self {
        Self { reduce_factors }
    }
}

#[derive(Debug)]
pub struct Scene {
    pub id: i32,
    pub objects: IndexMap<String, SceneObject>,
    pub view: View,
    pub options: SceneOptions,
}

impl Scene {
    pub fn new(id: i32, options: SceneOptions) -> Self {
        Self {
            id,
            objects: IndexMap::new(),
            view: View {
                center: Center { x: 0.0, y: 0.0 },
                diagonal: 25.0,
            },
            options,
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

    fn to_equations(python_expressions: String) -> Result<(Vec<String>, Vec<Plot>), SceneError> {
        let python_code = format!(
            "from equation_processor import *\n{}\n\n# Print all equations\nfor eq in equations:\n    print(eq)\nprint()\n# Print all plots\nfor plot in plots:\n    print(plot)",
            python_expressions
        );
        info!("Python code: {}", python_code);

        let py_dir = Path::new("src/py");
        let output = Command::new("python3")
            .current_dir(py_dir)
            .arg("-c")
            .arg(python_code)
            .output()
            .map_err(|e| SceneError::DatabaseError(format!("Failed to run Python: {}", e)))?;
        println!("output status: {:?}", output.status);
        println!(
            "output stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        if !output.stderr.is_empty() {
            println!(
                "output stderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

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

    pub fn evaluate_initial_values(
        &self,
        expressions: &Vec<String>,
    ) -> Result<Vec<f64>, SceneError> {
        SceneUtils::evaluate_initial_values(&self.to_python(), expressions)
    }

    pub fn validate_expression(&self, expression: String) -> Vec<String> {
        let mut messages = Vec::new();
        let identifiers = SceneUtils::extract_identifiers(&expression);

        // Validate field names
        let allowed_field_names = vec!["x", "y", "o", "n"];
        for field_name in &identifiers.field_names {
            if !allowed_field_names.contains(&field_name.as_str()) {
                messages.push(format!(
                    "Invalid field name: '{}'. Allowed fields are: {:?}",
                    field_name, allowed_field_names
                ));
            }
        }

        // Validate method names
        let allowed_method_names = vec!["abs", "length", "length_sqr", "rotated90", "contains"];
        for method_name in &identifiers.method_names {
            if !allowed_method_names.contains(&method_name.as_str()) {
                messages.push(format!(
                    "Invalid method name: '{}'. Allowed methods are: {:?}",
                    method_name, allowed_method_names
                ));
            }
        }

        // Validate function names
        let allowed_function_names = vec!["sqrt", "d", "d_sqr", "cot", "Point", "Line", "Vector"];
        for function_name in &identifiers.function_names {
            if !allowed_function_names.contains(&function_name.as_str()) {
                messages.push(format!(
                    "Invalid function name: '{}'. Allowed functions are: {:?}",
                    function_name, allowed_function_names
                ));
            }
        }

        // Validate object names
        for object_name in &identifiers.object_names {
            if let Some(scene_object) = self.objects.get(object_name) {
                let object_type = scene_object.get_type();
                let type_name = format!("{:?}", object_type);

                // Check if the object type is allowed (not ending with "Invariant" and not Locus)
                if type_name.ends_with("Invariant") || type_name == "Locus" {
                    messages.push(format!(
                        "Object '{}' has type '{}' which is not allowed in expressions",
                        object_name, type_name
                    ));
                }
            } else {
                messages.push(format!("Object '{}' not found in scene", object_name));
            }
        }

        messages
    }

    pub fn solve_and_plot(
        &self,
        locus_name: &str,
        width: u32,
        height: u32,
    ) -> Result<PlotData, SceneError> {
        // Convert plot to equations
        let (equations, plots) = SceneUtils::to_equations(self.to_python())?;
        info!(
            "Found {} equations and {} plots",
            equations.len(),
            plots.len()
        );
        let plot = plots.iter().find(|p| p.name == locus_name).unwrap();

        // Get curve equation and factors
        let curve_equation_and_factors = SceneUtils::get_curve_equation_and_factors(
            equations.iter().map(|s| s.as_str()).collect(),
            plot,
            self.options.clone(),
        )
        .map_err(|e| SceneError::InvalidEquation(e.to_string()))?;

        info!(
            "Curve equation: {}",
            curve_equation_and_factors.curve_equation
        );

        let (x_var, y_var) = SceneUtils::parse_plot_vars(plot)?;
        // Convert to XYPoly
        let xy_poly = curve_equation_and_factors
            .curve_equation
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

        let equation_str = format!("{}", curve_equation_and_factors.curve_equation);
        let formatted_equations: Vec<String> = curve_equation_and_factors
            .factors
            .iter()
            .map(|factor| factor.as_formatted_equation(x_var, y_var))
            .collect();

        Ok(PlotData {
            points,
            equation: equation_str,
            formatted_equations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{SceneActiveModel, SceneEntity, SceneObjectEntity};
    use crate::service::{config, AppState, CreateSceneRequest, SceneInfo};
    use sea_orm::ActiveValue::Set;
    use sea_orm::{Database, Schema};
    use serde_json::json;
    use test_log::test;

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
        let mut scene = Scene::new(scene_id, SceneOptions::default());

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
        let mut scene = Scene::new(scene_id, SceneOptions::default());

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
        let mut scene = Scene::new(scene_id, SceneOptions::default());

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
    async fn test_python_expressions_generation() {
        let db = setup_test_db().await;
        let scene_id = SceneEntity::find().one(&db).await.unwrap().unwrap().id;
        let mut scene = Scene::new(scene_id, SceneOptions::default());

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

        let python_expressions = scene.to_python();
        let expected_python_expressions = vec![
            "A = FixedPoint(0, 0)",
            "X = FreePoint(3, 4)",
            "plot(\"P1\", X)",
            "is_constant(d_sqr(A, X))",
        ]
        .join("\n");
        assert_eq!(python_expressions, expected_python_expressions);
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
    async fn test_get_scenes() {
        use actix_web::{test, web, App};
        // Setup test database
        let db = setup_test_db().await;
        let app_state = AppState::new(db.clone()).await;

        let scene2 = SceneActiveModel {
            name: Set("Scene 2".to_string()),
            ..Default::default()
        };
        scene2.insert(&db).await.unwrap();

        // Create test app
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .configure(config),
        )
        .await;

        // Test GET /scenes
        let req = test::TestRequest::get().uri("/scenes").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: Vec<SceneInfo> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);
        assert_eq!(body[0].id, 1);
        assert_eq!(body[1].id, 2);
        assert_eq!(body[0].name, "Test Scene");
        assert_eq!(body[1].name, "Scene 2");
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
        let mut scene = Scene::new(scene2.id, SceneOptions::default());
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

    #[test]
    fn test_float_parsing_logic() {
        // Test the float parsing logic that would be used in evaluate_initial_values
        let test_cases = vec![
            ("3.14", 3.14),
            ("2.718", 2.718),
            ("42.0", 42.0),
            ("-1.5", -1.5),
            ("0.0", 0.0),
            ("1e-6", 1e-6),
            ("1.5e+3", 1.5e+3),
        ];

        for (input, expected) in test_cases {
            match input.parse::<f64>() {
                Ok(value) => {
                    assert!(
                        (value - expected).abs() < f64::EPSILON,
                        "Failed to parse '{}' correctly. Expected: {}, Got: {}",
                        input,
                        expected,
                        value
                    );
                }
                Err(_) => {
                    panic!("Failed to parse '{}' as float", input);
                }
            }
        }
    }

    #[test]
    fn test_invalid_float_parsing() {
        // Test that invalid inputs are properly rejected
        let invalid_inputs = vec![
            "not_a_number",
            "3.14.15",
            "abc123",
            "3.14abc",
            "abc3.14",
            "",
            "   ",
        ];

        for input in invalid_inputs {
            let result = input.parse::<f64>();
            assert!(
                result.is_err(),
                "Expected '{}' to fail parsing as float, but got: {:?}",
                input,
                result
            );
        }
    }

    #[test]
    fn test_empty_and_whitespace_handling() {
        // Test that empty lines and whitespace are handled correctly
        let test_inputs = vec![
            ("3.14", true),
            ("  3.14  ", true), // Should trim whitespace
            ("", false),        // Empty line should be skipped
            ("   ", false),     // Whitespace-only line should be skipped
        ];

        for (input, should_parse) in test_inputs {
            let trimmed = input.trim();
            if should_parse {
                assert!(
                    !trimmed.is_empty(),
                    "Input '{}' should not be empty after trimming",
                    input
                );
                let result = trimmed.parse::<f64>();
                assert!(result.is_ok(), "Failed to parse '{}' as float", input);
            } else {
                assert!(
                    trimmed.is_empty(),
                    "Input '{}' should be empty after trimming",
                    input
                );
            }
        }
    }

    #[test]
    fn test_validate_expression_valid_expressions() {
        let mut scene = Scene::new(1, SceneOptions::default());

        // Add some test objects to the scene
        let point_props = json!({"value": "1, 2"});
        scene.objects.insert(
            "A".to_string(),
            SceneObject::FixedPoint(
                crate::scene_object::fixed_point::FixedPoint::new(point_props).unwrap(),
            ),
        );

        let point_props = json!({"value": "3, 4"});
        scene.objects.insert(
            "B".to_string(),
            SceneObject::FreePoint(
                crate::scene_object::free_point::FreePoint::new(point_props).unwrap(),
            ),
        );

        // Test valid expressions
        let valid_expressions = vec![
            "A.x + B.y",
            "d(A, B)",
            "A.x.abs()",
            "B.length()",
            "Point(1, 2)",
            "A.x + B.y + 5",
        ];

        for expression in valid_expressions {
            let messages = scene.validate_expression(expression.to_string());
            assert!(
                messages.is_empty(),
                "Expression '{}' should be valid but got messages: {:?}",
                expression,
                messages
            );
        }
    }

    #[test]
    fn test_validate_expression_invalid_field_names() {
        let scene = Scene::new(1, SceneOptions::default());

        let invalid_expressions = vec![
            "A.z",         // Invalid field 'z'
            "B.w",         // Invalid field 'w'
            "obj.invalid", // Invalid field 'invalid'
        ];

        for expression in invalid_expressions {
            let messages = scene.validate_expression(expression.to_string());
            assert!(
                !messages.is_empty(),
                "Expression '{}' should have validation errors",
                expression
            );
            assert!(
                messages
                    .iter()
                    .any(|msg| msg.contains("Invalid field name")),
                "Expression '{}' should have field name validation error",
                expression
            );
        }
    }

    #[test]
    fn test_validate_expression_invalid_method_names() {
        let mut scene = Scene::new(1, SceneOptions::default());

        // Add a test object
        let point_props = json!({"value": "1, 2"});
        scene.objects.insert(
            "A".to_string(),
            SceneObject::FixedPoint(
                crate::scene_object::fixed_point::FixedPoint::new(point_props).unwrap(),
            ),
        );

        let invalid_expressions = vec![
            "A.invalid()", // Invalid method 'invalid'
            "B.unknown()", // Invalid method 'unknown'
            "obj.bad()",   // Invalid method 'bad'
        ];

        for expression in invalid_expressions {
            let messages = scene.validate_expression(expression.to_string());
            assert!(
                !messages.is_empty(),
                "Expression '{}' should have validation errors",
                expression
            );
            assert!(
                messages
                    .iter()
                    .any(|msg| msg.contains("Invalid method name")),
                "Expression '{}' should have method name validation error",
                expression
            );
        }
    }

    #[test]
    fn test_validate_expression_invalid_function_names() {
        let scene = Scene::new(1, SceneOptions::default());

        let invalid_expressions = vec![
            "invalid(A, B)",  // Invalid function 'invalid'
            "unknown(1, 2)",  // Invalid function 'unknown'
            "bad_function()", // Invalid function 'bad_function'
        ];

        for expression in invalid_expressions {
            let messages = scene.validate_expression(expression.to_string());
            assert!(
                !messages.is_empty(),
                "Expression '{}' should have validation errors",
                expression
            );
            assert!(
                messages
                    .iter()
                    .any(|msg| msg.contains("Invalid function name")),
                "Expression '{}' should have function name validation error",
                expression
            );
        }
    }

    #[test]
    fn test_validate_expression_nonexistent_objects() {
        let scene = Scene::new(1, SceneOptions::default());

        let invalid_expressions = vec![
            "Nonexistent.x",     // Object doesn't exist
            "Missing.y",         // Object doesn't exist
            "d(Nonexistent, B)", // Object doesn't exist
        ];

        for expression in invalid_expressions {
            let messages = scene.validate_expression(expression.to_string());
            assert!(
                !messages.is_empty(),
                "Expression '{}' should have validation errors",
                expression
            );
            assert!(
                messages
                    .iter()
                    .any(|msg| msg.contains("not found in scene")),
                "Expression '{}' should have object not found error",
                expression
            );
        }
    }

    #[test]
    fn test_validate_expression_disallowed_object_types() {
        let mut scene = Scene::new(1, SceneOptions::default());

        // Add an Invariant object (disallowed)
        let invariant_props = json!({"formula": "x^2 + y^2 - 1"});
        scene.objects.insert(
            "Inv".to_string(),
            SceneObject::Invariant(
                crate::scene_object::invariant::Invariant::new(invariant_props).unwrap(),
            ),
        );

        let invalid_expressions = vec![
            "Inv.x",     // Invariant object not allowed
            "d(Inv, A)", // Invariant object not allowed
        ];

        for expression in invalid_expressions {
            let messages = scene.validate_expression(expression.to_string());
            assert!(
                !messages.is_empty(),
                "Expression '{}' should have validation errors",
                expression
            );
            assert!(
                messages
                    .iter()
                    .any(|msg| msg.contains("not allowed in expressions")),
                "Expression '{}' should have disallowed object type error",
                expression
            );
        }
    }

    #[test]
    fn test_validate_expression_multiple_errors() {
        let scene = Scene::new(1, SceneOptions::default());

        // Expression with multiple issues
        let expression = "Nonexistent.invalid() + bad_function(Missing)";
        let messages = scene.validate_expression(expression.to_string());

        assert!(
            messages.len() >= 3,
            "Should have at least 3 validation errors"
        );
        println!("{:?}", messages);

        // Check for specific error types
        let has_object_not_found = messages
            .iter()
            .any(|msg| msg.contains("not found in scene"));
        let has_invalid_method = messages
            .iter()
            .any(|msg| msg.contains("Invalid method name"));
        let has_invalid_function = messages
            .iter()
            .any(|msg| msg.contains("Invalid function name"));

        assert!(has_object_not_found, "Should have object not found error");
        assert!(has_invalid_method, "Should have invalid method name error");
        assert!(
            has_invalid_function,
            "Should have invalid function name error"
        );
    }

    #[test]
    fn test_validate_expression_empty_expression() {
        let scene = Scene::new(1, SceneOptions::default());

        let messages = scene.validate_expression("".to_string());
        assert!(messages.is_empty(), "Empty expression should be valid");
    }

    #[test]
    fn test_validate_expression_numbers_only() {
        let scene = Scene::new(1, SceneOptions::default());

        let messages = scene.validate_expression("123 + 456".to_string());
        assert!(
            messages.is_empty(),
            "Expression with only numbers should be valid"
        );
    }
}
