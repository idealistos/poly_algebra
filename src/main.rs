mod db;
mod elimination;
mod fint;
mod modular_poly;
mod poly;
mod poly_draw;
mod scene;
mod scene_object;
mod service;
mod x_poly;

use chrono::Utc;
use log::info;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Set};
use sea_orm::{Database, DatabaseConnection, Statement};
use std::env;
use std::fs;
use std::path::Path;

use crate::db::SceneActiveModel;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use env_logger::Env;

async fn init_database() -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
    // Database file path
    let db_path = "scenes.db";

    // Remove existing database if it exists
    if Path::new(db_path).exists() {
        fs::remove_file(db_path)?;
    }

    // Connect to database (this will create the file)
    let db = Database::connect(format!("sqlite://{}?mode=rwc", db_path)).await?;

    // Read all migration files from migrations folder
    let migrations_dir = Path::new("migrations");
    let mut migration_files: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("sql"))
        .collect();

    // Sort migration files by filename to ensure correct order
    migration_files.sort_by_key(|a| a.file_name());

    // Apply each migration in order
    for migration_file in migration_files {
        let migration_content = fs::read_to_string(migration_file.path())?;
        info!("Applying migration: {:?}", migration_file.file_name());

        // Execute migration
        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            migration_content,
        ))
        .await?;
    }

    info!("Database initialized successfully at {:?}", db_path);
    Ok(db)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting server...");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <command>", args[0]);
        println!("Commands:");
        println!("  -i    Initialize database");
        println!("  -s    Start web server");
        return Ok(());
    }

    match args[1].as_str() {
        "-i" => {
            let db = init_database().await.unwrap();
            let scene = SceneActiveModel {
                id: Set(1),
                created_at: Set(Utc::now()),
                view: Set("{\"center\": {\"x\": 0.0, \"y\": 0.0}, \"diagonal\": 25.0}".to_string()),
                name: Set("Scene 1".to_string()),
            };

            match scene.insert(&db).await {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to add an object to the database: {}", e),
            }
            return Ok(());
        }
        "-s" => {
            let db = Database::connect("sqlite://scenes.db?mode=rwc")
                .await
                .unwrap();
            let app_state = service::AppState::new(db).await;

            HttpServer::new(move || {
                App::new()
                    .wrap(
                        Cors::default()
                            .allowed_origin("http://localhost:5174")
                            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                            .allowed_header(actix_web::http::header::CONTENT_TYPE)
                            .supports_credentials(),
                    )
                    .app_data(web::Data::new(app_state.clone()))
                    .configure(service::config)
            })
            .bind(("127.0.0.1", 8080))?
            .run()
            .await?;
        }
        _ => {
            println!("Unknown command: {}", args[1]);
        }
    }

    Ok(())
}
