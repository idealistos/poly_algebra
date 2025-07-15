use actix_web::{delete, get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use log::info;
use sea_orm::ActiveValue::NotSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use crate::db::{SceneActiveModel, SceneColumn, SceneEntity, SCENE_DEFAULT_NAME};
use crate::poly_draw::Color;
use crate::scene::{Scene, View};
use crate::scene_object::{ObjectType, SceneError, SceneObject};
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
    pub object_type: String,
    pub arguments: Vec<Argument>,
    pub description: String,
    pub allowed_names: Vec<String>,
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

    pub async fn load_scene(&self, scene_id: &str) -> SceneOrError {
        let scene_id = scene_id.parse::<i32>().unwrap();
        let mut scene = Scene::new(scene_id);
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
            object_type: ObjectType::FixedPoint.to_string(),
            arguments: vec![Argument {
                types: vec!["GridPoint".to_string()],
                hint: "Select a point on the grid".to_string(),
                exclusive_object_types: vec![
                    ObjectType::FixedPoint.to_string(),
                    ObjectType::FreePoint.to_string(),
                ],
            }],
            description: "Fixed point: a point with constant integer coordinates".to_string(),
            allowed_names: letters_a_to_k,
        },
        Action {
            name: "FreePoint".to_string(),
            object_type: ObjectType::FreePoint.to_string(),
            arguments: vec![Argument {
                types: vec!["GridPoint".to_string()],
                hint: "Select a point on the grid".to_string(),
                exclusive_object_types: vec![
                    ObjectType::FixedPoint.to_string(),
                    ObjectType::FreePoint.to_string(),
                ],
            }],
            description: "Free point: the initial position of a point subject to future constraints"
                .to_string(),
            allowed_names: letters_x_to_z_then_t_to_w,
        },
        Action {
            name: "Midpoint".to_string(),
            object_type: ObjectType::Midpoint.to_string(),
            arguments: vec![
                Argument {
                    types: vec!["AnyDefinedPoint".to_string(), "GridPoint".to_string()],
                    hint: "Select an already defined point or a point on the grid (1 of 2)"
                        .to_string(),
                    exclusive_object_types: vec![],
                },
                Argument {
                    types: vec!["AnyDefinedPoint".to_string(), "GridPoint".to_string()],
                    hint: "Select an already defined point or a point on the grid (2 of 2)"
                        .to_string(),
                    exclusive_object_types: vec![],
                },
            ],
            description: "Midpoint: the point halfway between two given points".to_string(),
            allowed_names: ('A'..='K')
                .map(|c| ("midpoint".to_string() + &c.to_string()))
                .collect(),
        },
        Action {
            name: "IntersectionPoint".to_string(),
            object_type: ObjectType::IntersectionPoint.to_string(),
            arguments: vec![Argument {
                types: vec!["IntersectionPoint".to_string()],
                hint: "Select a point common to two lines".to_string(),
                exclusive_object_types: vec![ObjectType::IntersectionPoint.to_string()],
            }],
            description: "Intersection point: the point where two lines meet".to_string(),
            allowed_names: ('A'..='K')
                .map(|c| ("x".to_string() + &c.to_string()))
                .collect(),
        },
        Action {
            name: "SlidingPoint".to_string(),
            object_type: ObjectType::SlidingPoint.to_string(),
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
            allowed_names: ('A'..='K')
                .map(|c| ("point".to_string() + &c.to_string()))
                .collect(),
        },
        Action {
            name: "LineAB".to_string(),
            object_type: ObjectType::LineAB.to_string(),
            arguments: vec![
                Argument {
                    types: vec!["AnyDefinedPoint".to_string(), "GridPoint".to_string()],
                    hint: "Select a point on the grid (1 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
                Argument {
                    types: vec!["AnyDefinedPoint".to_string(), "GridPoint".to_string()],
                    hint: "Select a point on the grid (2 of 2)".to_string(),
                    exclusive_object_types: vec![],
                },
            ],
            description: "Line: a line passing through two given points".to_string(),
            allowed_names: ('A'..='K')
                .map(|c| ("line".to_string() + &c.to_string()))
                .collect(),
        },
        Action {
            name: "Parameter".to_string(),
            object_type: ObjectType::Parameter.to_string(),
            arguments: vec![],
            description: "Parameter: a free variable with 0 initial value, to use in an Invariant"
                .to_string(),
            allowed_names: ('t'..='w').map(|c| c.to_string()).collect(),
        },
        Action {
            name: "Invariant".to_string(),
            object_type: ObjectType::Invariant.to_string(),
            arguments: vec![Argument {
                types: vec![],
                hint: "Enter the formula for the invariant, e.g., d(A, X)".to_string(),
                exclusive_object_types: vec![],
            }],
            description:
                "Invariant: a relation of the form F(object1, object2,..) = C that constrains defined objects (free points, etc.). C is the initial value of the expression."
                    .to_string(),
            allowed_names: ('A'..='K')
                .map(|c| ("inv".to_string() + &c.to_string()))
                .collect(),
        },
        Action {
            name: "Locus".to_string(),
            object_type: ObjectType::Locus.to_string(),
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
        },
    ];

    HttpResponse::Ok().json(actions)
}

#[get("/scenes/{scene_id}")]
async fn get_scene(data: web::Data<AppState>, scene_id: web::Path<String>) -> impl Responder {
    match data.load_scene(&scene_id.into_inner()).await {
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
    match data.load_scene(&path.into_inner()).await {
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
    match data.load_scene(&scene_id).await {
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
    match data.load_scene(&scene_id).await {
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
    match data.load_scene(&scene_id).await {
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

    match data.load_scene(&scene_id).await {
        SceneOrError::Scene(scene) => {
            if let Some(SceneObject::Locus(_locus)) = scene.objects.get(&locus_name) {
                match scene.solve_and_plot(&locus_name, width, height) {
                    Ok(points) => HttpResponse::Ok().json(points),
                    Err(e) => {
                        info!(
                            "Failed to solve for locus {}: {}",
                            locus_name,
                            e.to_string()
                        );
                        HttpResponse::InternalServerError().json(e.to_string())
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

#[derive(Debug, Serialize)]
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

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_actions)
        .service(get_scene)
        .service(add_object)
        .service(delete_object)
        .service(delete_scene)
        .service(get_dependents)
        .service(get_plot)
        .service(create_scene)
        .service(get_scenes);
}
