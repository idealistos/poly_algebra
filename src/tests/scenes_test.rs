use crate::scene::{ActiveModel as SceneActiveModel, Entity as Scene};
use crate::service::{config, AppState};
use actix_web::{test, web, App};
use sea_orm::{DatabaseConnection, EntityTrait, Set};

#[actix_web::test]
async fn test_get_scenes() {
    // Setup test database
    let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
    let app_state = AppState::new(db.clone()).await;

    // Insert test scenes
    let scene1 = SceneActiveModel {
        name: Set("Scene 1".to_string()),
        created_at: Set(chrono::Utc::now().naive_utc()),
        ..Default::default()
    };
    let scene2 = SceneActiveModel {
        name: Set("Scene 2".to_string()),
        created_at: Set(chrono::Utc::now().naive_utc()),
        ..Default::default()
    };
    scene1.insert(&db).await.unwrap();
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

    let body: Vec<i32> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 2);
    assert!(body[0] < body[1]); // Check order by created_at
}
