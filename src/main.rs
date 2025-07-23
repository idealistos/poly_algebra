mod db;
mod elimination;
mod fint;
mod gp_pari_service;
mod modular_poly;
mod poly;
mod poly_draw;
mod scene;
mod scene_object;
mod service;
mod x_poly;

use chrono::Utc;
use clap::{Parser, Subcommand};
use log::info;
use sea_orm::{ActiveModelTrait, ConnectOptions, ConnectionTrait, Set};
use sea_orm::{Database, DatabaseConnection, Statement};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::db::SceneActiveModel;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use env_logger::Env;

// Global variable to store the Pari/GP executable path
static mut PARI_EXECUTABLE_PATH: Option<String> = None;

// Global singleton for GpPariService
static mut GP_PARI_SERVICE: Option<gp_pari_service::GpPariService> = None;

#[derive(Parser)]
#[command(name = "poly_algebra")]
#[command(about = "A program for eliminating variables from multivariate polynomials")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Specify Pari/GP executable path
    #[arg(long, value_name = "PATH")]
    gp_executable: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize database
    Init,
    /// Start web server
    Start,
}

/// Get the Pari/GP executable path, resolving it from command line arguments or system PATH
pub fn get_pari_executable_path() -> Result<String, String> {
    // Check if we have a cached path
    unsafe {
        if let Some(ref path) = PARI_EXECUTABLE_PATH {
            return Ok(path.clone());
        }
    }

    // Get the CLI arguments
    let cli = Cli::parse();

    // Check if --gp-executable was provided
    if let Some(path) = cli.gp_executable {
        // Validate that the executable exists
        if Path::new(&path).exists() {
            unsafe {
                PARI_EXECUTABLE_PATH = Some(path.clone());
            }
            return Ok(path);
        } else {
            return Err(format!("Pari/GP executable not found at: {}", path));
        }
    }

    // If no explicit path provided, try to find gp executable in system PATH
    let gp_names = if cfg!(target_os = "windows") {
        vec!["gp.exe", "gp"]
    } else {
        vec!["gp", "gp.exe"]
    };

    for name in gp_names {
        match Command::new(name).arg("--version").output() {
            Ok(_) => {
                let path = name.to_string();
                unsafe {
                    PARI_EXECUTABLE_PATH = Some(path.clone());
                }
                return Ok(path);
            }
            Err(_) => continue,
        }
    }

    Err("Pari/GP executable not found. Please install Pari/GP or specify the path with --gp-executable".to_string())
}

/// Set the Pari/GP executable path (for testing or manual override)
pub fn set_pari_executable_path(path: String) {
    unsafe {
        PARI_EXECUTABLE_PATH = Some(path);
    }
}

/// Initialize the global GpPariService singleton
pub fn init_gp_pari_service() -> Result<(), String> {
    let executable_path = get_pari_executable_path()?;
    unsafe {
        GP_PARI_SERVICE = Some(gp_pari_service::GpPariService::new(executable_path));
    }
    Ok(())
}

/// Get a mutable reference to the global GpPariService singleton
pub fn get_gp_pari_service() -> Result<&'static mut gp_pari_service::GpPariService, String> {
    unsafe {
        if let Some(ref mut service) = GP_PARI_SERVICE {
            Ok(service)
        } else {
            Err("GpPariService not initialized".to_string())
        }
    }
}

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

    let cli = Cli::parse();

    // Initialize Pari/GP executable path
    match get_pari_executable_path() {
        Ok(path) => info!("Using Pari/GP executable: {}", path),
        Err(e) => {
            eprintln!("Warning: {}", e);
            eprintln!("Pari/GP functionality will be limited");
        }
    }

    // Initialize GpPariService singleton
    if let Err(e) = init_gp_pari_service() {
        eprintln!("Warning: Failed to initialize GpPariService: {}", e);
        eprintln!("Pari/GP functionality will be limited");
    } else {
        info!("GpPariService initialized successfully");
    }

    match cli.command {
        Commands::Init => {
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
        Commands::Start => {
            let mut connect_options = ConnectOptions::new("sqlite://scenes.db?mode=rwc");
            connect_options.sqlx_logging(false);
            let db = Database::connect(connect_options).await.unwrap();
            let app_state = service::AppState::new(db).await;

            HttpServer::new(move || {
                App::new()
                    .wrap(
                        Cors::default()
                            .allowed_origin("http://localhost:5174")
                            .allowed_methods(vec![
                                "GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS",
                            ])
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
    }

    Ok(())
}
