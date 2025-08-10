use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use crate::db::{SceneActiveModel, SceneColumn, SceneEntity, SCENE_DEFAULT_NAME};
use crate::poly_draw::Color;
use crate::scene::{Scene, SceneOptions, View};
use crate::scene_object::{ObjectType, SceneObject};
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryOrder, Set,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Argument {
    pub types: Vec<String>,
    pub hint: String,
    pub exclusive_object_types: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub object_types: Vec<String>,
    pub arguments: Vec<Argument>,
    pub description: String,
    pub allowed_names: Vec<String>,
    pub group: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneObjectResponse {
    pub name: String,
    pub object_type: String,
    pub properties: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneResponse {
    pub objects: Vec<SceneObjectResponse>,
    pub view: View,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlotPoint {
    pub x: u32,
    pub y: u32,
    pub color: Color,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlotResponse {
    pub points: Vec<(u32, u32, Color)>,
    pub equation: String,
    pub formatted_equations: Vec<String>,
    pub time_taken: f64,
}

#[derive(Debug)]
pub enum SceneOrError {
    Scene(Scene),
    Error(HttpResponse),
}

#[derive(Clone)]
pub struct AppState {
    db: Arc<DatabaseConnection>,
}

impl AppState {
    pub async fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    pub async fn load_scene(&self, scene_id: &str, options: SceneOptions) -> SceneOrError {
        let scene_id = scene_id.parse::<i32>().unwrap();
        let mut scene = Scene::new(scene_id, options);
        match scene.load_objects_and_view(&self.db).await {
            Ok(()) => SceneOrError::Scene(scene),
            Err(e) => SceneOrError::Error(HttpResponse::InternalServerError().json(e.to_string())),
        }
    }
}

#[get("/actions")]
async fn get_actions() -> impl Responder {
    let letters_a_to_k: Vec<String> = ('A'..='K').map(|c| c.to_string()).collect();
    let mut letters_x_to_z_then_t_to_w: Vec<String> = ('X'..='Z').map(|c| c.to_string()).collect();
    letters_x_to_z_then_t_to_w.extend(('T'..='W').map(|c| c.to_string()));

    let actions = vec![
        Action {
            name: "FixedPoint".to_string(),
            object_types: vec![ObjectType::FixedPoint.to_string()],
            arguments: vec![Argument {
                types: vec!["GridPoint".to_string()],
                hint: "Select a point on the grid".to_string(),
                exclusive_object_types: vec![
                    ObjectType::FixedPoint.to_string(),
                    ObjectType::FreePoint.to_string(),
                    ObjectType::Midpoint.to_string(),
                    ObjectType::SlidingPoint.to_string(),
                    ObjectType::IntersectionPoint.to_string(),
                ],
            }],
            description: "Fixed point: a point with constant integer coordinates".to_string(),
            allowed_names: letters_a_to_k,
            group: "Points".to_string(),
        },
        Action {
            name: "FreePoint".to_string(),
            object_types: vec![ObjectType::FreePoint.to_string()],
            arguments: vec![Argument {
                types: vec!["GridPoint".to_string()],
                hint: "Select a point on the grid".to_string(),
                exclusive_object_types: vec![
                    ObjectType::FixedPoint.to_string(),
                    ObjectType::FreePoint.to_string(),
                    ObjectType::Midpoint.to_string(),
                    ObjectType::SlidingPoint.to_string(),
                    ObjectType::IntersectionPoint.to_string(),
                ],
            }],
            description: "Free point: the initial position of a point subject to future constraints"
                .to_string(),
            allowed_names: letters_x_to_z_then_t_to_w.clone(),
            group: "Points".to_string(),
        },
        Action {
            name: "Midpoint".to_string(),
            object_types: vec![ObjectType::Midpoint.to_string()],
            arguments: vec![
                Argument {
                    types: vec!["AnyDefinedOrGridPoint".to_string()],
                    hint: "Select an already defined point or a point on the grid (1 of 2)"
                        .to_string(),
                    exclusive_object_types: vec![],
                },
                Argument {
                    types: vec!["AnyDefinedOrGridPoint".to_string()],
                    hint: "Select an already defined point or a point on the grid (2 of 2)"
                        .to_string(),
                    exclusive_object_types: vec![],
                },
            ],
            description: "Midpoint: the point halfway between two given points".to_string(),
            allowed_names: letters_x_to_z_then_t_to_w.clone(),
            group: "Points".to_string(),
        },
        Action {
            name: "IntersectionPoint".to_string(),
            object_types: vec![ObjectType::IntersectionPoint.to_string()],
            arguments: vec![Argument {
                types: vec!["IntersectionPoint".to_string()],
                hint: "Select a point common to two lines".to_string(),
                exclusive_object_types: vec![ObjectType::IntersectionPoint.to_string()],
            }],
            description: "Intersection point: the point where two lines meet".to_string(),
            allowed_names: letters_x_to_z_then_t_to_w.clone(),
            group: "Points".to_string(),
        },
        Action {
            name: "SlidingPoint".to_string(),
            object_types: vec![ObjectType::SlidingPoint.to_string()],
            arguments: vec![Argument {
                types: vec!["SlidingPoint".to_string()],
                hint: "Select a point on a line".to_string(),
                exclusive_object_types: vec![
                    ObjectType::SlidingPoint.to_string(),
                    ObjectType::FreePoint.to_string(),
                    ObjectType::FixedPoint.to_string(),
                    ObjectType::IntersectionPoint.to_string(),
                ],
            }],
            description: "Sliding point: the initial position of a point constrained to a line".to_string(),
            allowed_names: letters_x_to_z_then_t_to_w.clone(),
            group: "Points".to_string(),
        },
        Action {
            name: "Projection".to_string(),
            object_types: vec![
                ObjectType::Projection.to_string(),
            ],
            arguments: vec![
                Argument {
                    types: vec!["AnyDefinedOrGridPoint".to_string()],
                    hint: "Select the point to be projected (an already defined point or a point on the grid) (1 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
                Argument {
                    types: vec!["Line".to_string()],
                    hint: "Select the line to be projected onto (2 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
            ],
            description:
                "Projection: the point on a line that is the perpendicular projection of a given point onto the line"
                    .to_string(),
            allowed_names: letters_x_to_z_then_t_to_w.clone(),
            group: "Points".to_string(),
        },
        Action {
            name: "Reflection".to_string(),
            object_types: vec![
                ObjectType::Reflection.to_string(),
            ],
            arguments: vec![
                Argument {
                    types: vec!["AnyDefinedOrGridPoint".to_string()],
                    hint: "Select the point to be reflected (an already defined point or a point on the grid) (1 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
                Argument {
                    types: vec!["Line".to_string()],
                    hint: "Select the line to be reflected across (2 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
            ],
            description:
                "Reflection: the point on the other side of a line that is the reflection of a given point across the line"
                    .to_string(),
            allowed_names: letters_x_to_z_then_t_to_w.clone(),
            group: "Points".to_string(),
        },
        Action {
            name: "ScaledVectorPoint".to_string(),
            object_types: vec![ObjectType::ScaledVectorPoint.to_string()],
            arguments: vec![
                Argument {
                    types: vec![],
                    hint: "Enter the expression for the scaling coefficient k (1 of 3)".to_string(),
                    exclusive_object_types: vec![],
                },
                Argument {
                    types: vec!["AnyDefinedOrGridPoint".to_string()],
                    hint: "Choose the start point (A) of the reference vector (2 of 3)".to_string(),
                    exclusive_object_types: vec![],
                },
                Argument {
                    types: vec!["AnyDefinedOrGridPoint".to_string()],
                    hint: "Choose the end point (B) of the reference vector (3 of 3)".to_string(),
                    exclusive_object_types: vec![],
                },
            ],
            description: "Scaled vector point: a point X defined by the vector relation AX = k AB for chosen A and B".to_string(),
            allowed_names: letters_x_to_z_then_t_to_w.clone(),
            group: "Points".to_string(),
        },        
        Action {
            name: "ComputedPoint".to_string(),
            object_types: vec![ObjectType::ComputedPoint.to_string()],
            arguments: vec![
                Argument {
                    types: vec![],
                    hint: "Enter the expression for the X coordinate of the point (may include parameters and reference objects) (1 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
                Argument {
                    types: vec![],
                    hint: "Enter the expression for the Y coordinate of the point (may include parameters and reference objects) (2 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
            ],
            description: "Computed point: a point defined by custom X and Y expressions".to_string(),
            allowed_names: letters_x_to_z_then_t_to_w.clone(),
            group: "Points".to_string(),
        },        
        Action {
            name: "LineAB".to_string(),
            object_types: vec![ObjectType::LineAB.to_string()],
            arguments: vec![
                Argument {
                    types: vec!["AnyDefinedOrGridPoint".to_string()],
                    hint: "Select an already defined point or a point on the grid (1 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
                Argument {
                    types: vec!["AnyDefinedOrGridPoint".to_string()],
                    hint: "Select an already defined point or a point on the grid (2 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
            ],
            description: "Line: a line passing through two given points".to_string(),
            allowed_names: ('A'..='K')
                .map(|c| ("line".to_string() + &c.to_string()))
                .collect(),
            group: "Lines".to_string(),
        },
        Action {
            name: "PpBisector".to_string(),
            object_types: vec![ObjectType::PpBisector.to_string()],
            arguments: vec![
                Argument {
                    types: vec!["AnyDefinedOrGridPoint".to_string()],
                    hint: "Select an already defined point or a point on the grid (1 of 2)"
                        .to_string(),
                    exclusive_object_types: vec![],
                },
                Argument {
                    types: vec!["AnyDefinedOrGridPoint".to_string()],
                    hint: "Select an already defined point or a point on the grid (2 of 2)"
                        .to_string(),
                    exclusive_object_types: vec![],
                },
            ],
            description: "Perpendicular bisector: the line consisting of points equidistant to two given points".to_string(),
            allowed_names: ('A'..='K')
                .map(|c| ("line".to_string() + &c.to_string()))
                .collect(),
            group: "Lines".to_string(),
        },
        Action {
            name: "PpToLine".to_string(),
            object_types: vec![
                ObjectType::PpToLine.to_string(),
            ],
            arguments: vec![Argument {
                types: vec!["AnyDefinedOrGridPoint".to_string()],
                hint: "Select an already defined point or a point on the grid (1 of 2)".to_string(),
                exclusive_object_types: vec![],
            },
            Argument {
                types: vec!["Line".to_string()],
                hint: "Select a line (2 of 2)".to_string(),
                exclusive_object_types: vec![],
            }],
            description:
                "Perpendicular to a line: a line through the given point perpendicular to the given line"
                    .to_string(),
            allowed_names: ('A'..='K')
                .map(|c| ("line".to_string() + &c.to_string()))
                .collect(),
            group: "Lines".to_string(),
        },
        Action {
            name: "PlToLine".to_string(),
            object_types: vec![
                ObjectType::PlToLine.to_string(),
            ],
            arguments: vec![Argument {
                types: vec!["AnyDefinedOrGridPoint".to_string()],
                hint: "Select an already defined point or a point on the grid (1 of 2)".to_string(),
                exclusive_object_types: vec![],
            },
            Argument {
                types: vec!["Line".to_string()],
                hint: "Select a line (2 of 2)".to_string(),
                exclusive_object_types: vec![],
            }],
            description:
                "Parallel to a line: a line through the given point parallel to the given line"
                    .to_string(),
            allowed_names: ('A'..='K')
                .map(|c| ("line".to_string() + &c.to_string()))
                .collect(),
            group: "Lines".to_string(),
        },
        Action {
            name: "Parameter".to_string(),
            object_types: vec![ObjectType::Parameter.to_string()],
            arguments: vec![],
            description: "Parameter: a free variable with 0 initial value, to use in an Invariant"
                .to_string(),
            allowed_names: ('t'..='w').map(|c| c.to_string()).collect(),
            group: "Parameters".to_string(),
        },
        Action {
            name: "DistanceInvariant".to_string(),
            object_types: vec![
                ObjectType::TwoPointDistanceInvariant.to_string(),
                ObjectType::PointToLineDistanceInvariant.to_string(),
            ],
            arguments: vec![
                Argument {
                    types: vec!["AnyDefinedOrGridPoint".to_string()],
                    hint: "Select an already defined point or a point on the grid (1 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
                Argument {
                    types: vec!["AnyDefinedOrGridPoint".to_string(), "Line".to_string()],
                    hint: "Select an already defined point or a point on the grid, or a line (2 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
            ],
            description:
                "Distance Invariant: specifies that the distance from a point to another point or line is constant"
                    .to_string(),
            allowed_names: ('A'..='K')
                .map(|c| ("inv".to_string() + &c.to_string()))
                .collect(),
            group: "Constraints".to_string(),
        },
        Action {
            name: "AngleInvariant".to_string(),
            object_types: vec![
                ObjectType::TwoLineAngleInvariant.to_string(),
            ],
            arguments: vec![
                Argument {
                    types: vec!["Line".to_string()],
                    hint: "Select a line (1 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
                Argument {
                    types: vec!["Line".to_string()],
                    hint: "Select a line (2 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
            ],
            description:
                "Angle Invariant: specifies that the angle between two lines is constant"
                    .to_string(),
            allowed_names: ('A'..='K')
                .map(|c| ("inv".to_string() + &c.to_string()))
                .collect(),
            group: "Constraints".to_string(),
        },
        Action {
            name: "Invariant".to_string(),
            object_types: vec![ObjectType::Invariant.to_string()],
            arguments: vec![Argument {
                types: vec![],
                hint: "Enter the formula for the invariant, e.g., d(A, X)".to_string(),
                exclusive_object_types: vec![],
            }],
            description:
                "Custom invariant: a relation of the form F(object1, object2,..) = C that constrains defined objects (free points, etc.). C is the initial value of the expression."
                    .to_string(),
            allowed_names: ('A'..='K')
                .map(|c| ("inv".to_string() + &c.to_string()))
                .collect(),
            group: "Constraints".to_string(),
        },
        Action {
            name: "Locus".to_string(),
            object_types: vec![ObjectType::Locus.to_string()],
            arguments: vec![Argument {
                types: vec!["MobilePoint".to_string()],
                hint: "Select an already defined mobile (i.e., not fixed) point".to_string(),
                exclusive_object_types: vec![ObjectType::Locus.to_string()],
            }],
            description: "Locus: pick a point to display the curve (all positions of that point satisfying the constraints)"
                .to_string(),
            allowed_names: ('A'..='K')
                .map(|c| ("locus".to_string() + &c.to_string()))
                .collect(),
            group: "Locus".to_string(),
        },
    ];

    HttpResponse::Ok().json(actions)
}

#[get("/scenes/{scene_id}")]
async fn get_scene(data: web::Data<AppState>, scene_id: web::Path<String>) -> impl Responder {
    match data
        .load_scene(&scene_id.into_inner(), SceneOptions::default())
        .await
    {
        SceneOrError::Scene(scene) => {
            let objects: Vec<SceneObjectResponse> = scene
                .objects
                .iter()
                .map(|obj| SceneObjectResponse {
                    name: obj.0.clone(),
                    object_type: obj.1.get_type().to_string(),
                    properties: obj.1.get_properties(),
                })
                .collect();

            match scene.get_view(&data.db).await {
                Ok(view) => {
                    let response = SceneResponse { objects, view };
                    HttpResponse::Ok().json(response)
                }
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        }
        SceneOrError::Error(response) => response,
    }
}

#[post("/scenes/{scene_id}/objects")]
async fn add_object(
    data: web::Data<AppState>,
    path: web::Path<String>,
    object: web::Json<SceneObjectResponse>,
) -> impl Responder {
    match data
        .load_scene(&path.into_inner(), SceneOptions::default())
        .await
    {
        SceneOrError::Scene(mut scene) => {
            match scene
                .add_object(
                    &data.db,
                    object.name.clone(),
                    ObjectType::from_str(&object.object_type).unwrap(),
                    object.properties.clone(),
                )
                .await
            {
                Ok(()) => HttpResponse::Ok().json(object.0),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        }
        SceneOrError::Error(response) => response,
    }
}

#[delete("/scenes/{scene_id}/{object_name}")]
async fn delete_object(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (scene_id, object_name) = path.into_inner();
    match data.load_scene(&scene_id, SceneOptions::default()).await {
        SceneOrError::Scene(mut scene) => match scene.delete_object(&data.db, &object_name).await {
            Ok(dependencies) => HttpResponse::Ok().json(dependencies),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        },
        SceneOrError::Error(response) => response,
    }
}

#[delete("/scenes/{scene_id}")]
async fn delete_scene(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let scene_id = path.into_inner();
    match data.load_scene(&scene_id, SceneOptions::default()).await {
        SceneOrError::Scene(mut scene) => match scene.delete_scene(&data.db).await {
            Ok(()) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        },
        SceneOrError::Error(response) => response,
    }
}

#[get("/scenes/{scene_id}/{object_name}/dependents")]
async fn get_dependents(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (scene_id, object_name) = path.into_inner();
    match data.load_scene(&scene_id, SceneOptions::default()).await {
        SceneOrError::Scene(scene) => {
            let dependents = scene.collect_dependent_objects(&object_name);
            HttpResponse::Ok().json(dependents)
        }
        SceneOrError::Error(response) => response,
    }
}

#[get("/scenes/{scene_id}/plot/{locus_name}")]
async fn get_plot(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let (scene_id, locus_name) = path.into_inner();

    // Parse width and height from query parameters
    let width = query
        .get("width")
        .and_then(|w| w.parse::<u32>().ok())
        .unwrap_or(2000);
    let height = query
        .get("height")
        .and_then(|h| h.parse::<u32>().ok())
        .unwrap_or(2000);
    let reduce_factors = query
        .get("reduce_factors")
        .and_then(|value| value.parse::<bool>().ok())
        .unwrap_or(false);

    match data
        .load_scene(&scene_id, SceneOptions::new(reduce_factors))
        .await
    {
        SceneOrError::Scene(scene) => {
            if let Some(SceneObject::Locus(_locus)) = scene.objects.get(&locus_name) {
                let start_time = Instant::now();
                match scene.solve_and_plot(&locus_name, width, height) {
                    Ok(plot_data) => {
                        let elapsed = start_time.elapsed();
                        let response = PlotResponse {
                            points: plot_data.points,
                            equation: plot_data.equation,
                            formatted_equations: plot_data.formatted_equations,
                            time_taken: elapsed.as_secs_f64(),
                        };
                        HttpResponse::Ok().json(response)
                    }
                    Err(e) => {
                        let elapsed = start_time.elapsed();
                        info!(
                            "Failed to solve for locus {}: {} (took {:.3}s)",
                            locus_name,
                            e.to_string(),
                            elapsed.as_secs_f64()
                        );
                        HttpResponse::InternalServerError().json(format!(
                            "{} (took {:.3}s)",
                            e.to_string(),
                            elapsed.as_secs_f64()
                        ))
                    }
                }
            } else {
                HttpResponse::NotFound().finish()
            }
        }
        SceneOrError::Error(response) => response,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSceneRequest {
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateSceneResponse {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenameSceneRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct RenameSceneResponse {
    pub id: i32,
    pub name: String,
}

#[post("/scenes")]
async fn create_scene(
    data: web::Data<AppState>,
    request: web::Json<CreateSceneRequest>,
) -> impl Responder {
    let db = &*data.db;

    // Create new scene in database
    let scene_name = request
        .name
        .as_deref()
        .unwrap_or(SCENE_DEFAULT_NAME)
        .to_string();
    let scene = SceneActiveModel {
        name: Set(scene_name.clone()),
        ..Default::default()
    };

    match scene.insert(db).await {
        Ok(scene) => {
            // Update the name to "Scene <id>" if the provided name was empty or default
            if request.name.is_none()
                || request.name.as_ref().unwrap().is_empty()
                || scene_name == SCENE_DEFAULT_NAME
            {
                let final_name = format!("Scene {}", scene.id);
                let id = scene.id;
                let mut update_scene = scene.into_active_model();
                update_scene.name = Set(final_name.clone());

                match update_scene.update(db).await {
                    Ok(_) => HttpResponse::Ok().json(CreateSceneResponse {
                        id,
                        name: final_name,
                    }),
                    Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
                }
            } else {
                HttpResponse::Ok().json(CreateSceneResponse {
                    id: scene.id,
                    name: scene.name,
                })
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[patch("/scenes/{scene_id}")]
async fn rename_scene(
    data: web::Data<AppState>,
    path: web::Path<String>,
    request: web::Json<RenameSceneRequest>,
) -> impl Responder {
    let scene_id = path.into_inner();
    let new_name = request.name.clone();

    let db = &*data.db;

    // Update the scene name in the database
    let scene_id_int = scene_id.parse::<i32>().unwrap();
    let scene_model = SceneEntity::find_by_id(scene_id_int)
        .one(db)
        .await
        .unwrap_or(None);

    match scene_model {
        Some(scene_model) => {
            let mut active_model = scene_model.into_active_model();
            active_model.name = Set(new_name.clone());

            match active_model.update(db).await {
                Ok(updated_scene) => HttpResponse::Ok().json(RenameSceneResponse {
                    id: updated_scene.id,
                    name: updated_scene.name,
                }),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        }
        None => HttpResponse::NotFound().json("Scene not found"),
    }
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct SceneInfo {
    pub id: i32,
    pub name: String,
    created_at: DateTime<Utc>,
}

#[get("/scenes")]
async fn get_scenes(data: web::Data<AppState>) -> impl Responder {
    let scenes = SceneEntity::find()
        .order_by_asc(SceneColumn::CreatedAt)
        .all(&*data.db)
        .await
        .unwrap_or_default();
    let scene_infos: Vec<SceneInfo> = scenes
        .into_iter()
        .map(|s| SceneInfo {
            id: s.id,
            name: s.name,
            created_at: s.created_at,
        })
        .collect();
    HttpResponse::Ok().json(scene_infos)
}

#[derive(Debug, Serialize)]
pub struct InitialValuesResponse {
    pub values: Vec<f64>,
}

#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub errors: Vec<String>,
}

#[get("/scenes/{scene_id}/initial")]
async fn get_initial_values(
    data: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let scene_id = path.into_inner();

    // Get the json parameter from query
    let json_param = match query.get("json") {
        Some(json_str) => json_str,
        None => {
            return HttpResponse::BadRequest().json("Missing 'json' query parameter");
        }
    };

    // Decode the base64-encoded JSON
    let decoded_json = match URL_SAFE_NO_PAD.decode(json_param) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => {
                return HttpResponse::BadRequest().json("Invalid UTF-8 in decoded JSON");
            }
        },
        Err(_) => {
            return HttpResponse::BadRequest().json("Invalid base64 encoding");
        }
    };

    // Parse the JSON array of expressions
    let expressions: Vec<String> = match serde_json::from_str(&decoded_json) {
        Ok(exprs) => exprs,
        Err(_) => {
            return HttpResponse::BadRequest()
                .json("Invalid JSON format - expected array of strings");
        }
    };

    // Load the scene
    let scene_or_error = data.load_scene(&scene_id, SceneOptions::default()).await;
    let scene = match scene_or_error {
        SceneOrError::Scene(scene) => scene,
        SceneOrError::Error(response) => return response,
    };

    match scene.evaluate_initial_values(&expressions) {
        Ok(values) => HttpResponse::Ok().json(InitialValuesResponse { values }),
        Err(e) => HttpResponse::InternalServerError()
            .json(format!("Failed to evaluate initial values: {}", e)),
    }
}

#[get("/scenes/{scene_id}/validate")]
async fn validate_expressions(
    data: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let scene_id = path.into_inner();

    let json_param = match query.get("json") {
        Some(json) => json,
        None => {
            return HttpResponse::BadRequest().json("Missing 'json' query parameter");
        }
    };

    let decoded_json = match URL_SAFE_NO_PAD.decode(json_param) {
        Ok(decoded) => decoded,
        Err(_) => {
            return HttpResponse::BadRequest().json("Invalid base64 encoding in 'json' parameter");
        }
    };

    let expressions: Vec<String> =
        match serde_json::from_str(&String::from_utf8_lossy(&decoded_json)) {
            Ok(expressions) => expressions,
            Err(e) => {
                return HttpResponse::BadRequest().json(format!("Invalid JSON format: {}", e));
            }
        };

    let scene_or_error = data.load_scene(&scene_id, SceneOptions::default()).await;
    let scene = match scene_or_error {
        SceneOrError::Scene(scene) => scene,
        SceneOrError::Error(response) => {
            return response;
        }
    };

    // Validate each expression and collect all errors
    let mut all_errors = Vec::new();
    for (index, expression) in expressions.iter().enumerate() {
        let errors = scene.validate_expression(expression.clone());
        for error in errors {
            all_errors.push(format!("Expression {}: {}", index + 1, error));
        }
    }

    HttpResponse::Ok().json(ValidationResponse { errors: all_errors })
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_actions)
        .service(get_scene)
        .service(add_object)
        .service(delete_object)
        .service(delete_scene)
        .service(get_dependents)
        .service(get_plot)
        .service(create_scene)
        .service(rename_scene)
        .service(get_initial_values)
        .service(validate_expressions)
        .service(get_scenes);
}
