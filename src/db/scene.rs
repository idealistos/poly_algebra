use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, Set};

pub const SCENE_DEFAULT_NAME: &str = "New Scene";
pub const SCENE_DEFAULT_VIEW: &str = r#"{"center": {"x": 0.0, "y": 0.0}, "diagonal": 25.0}"#;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scenes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub view: String,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: sea_orm::ActiveValue::NotSet,
            created_at: Set(Utc::now()),
            view: Set(SCENE_DEFAULT_VIEW.to_string()),
            name: Set(SCENE_DEFAULT_NAME.to_string()),
        }
    }
}
