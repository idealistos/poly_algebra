use std::str::FromStr;

use crate::scene_object::{ObjectType, SceneError, SceneObject};
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scene_objects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub scene_id: i32,
    pub object_type: String,
    pub object_name: String,
    pub properties: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::scene::Entity",
        from = "Column::SceneId",
        to = "super::scene::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Scene,
}

impl Related<super::scene::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Scene.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn get_scene_object(&self) -> Result<SceneObject, SceneError> {
        let properties: Value = serde_json::from_str(&self.properties)
            .map_err(|e| SceneError::InvalidProperties(e.to_string()))?;

        let object_type = ObjectType::from_str(&self.object_type)?;
        SceneObject::from_properties(object_type, properties)
    }

    pub async fn save_object(
        db: &DatabaseConnection,
        scene_id: i32,
        name: &str,
        object_type: ObjectType,
        properties: Value,
    ) -> Result<(), SceneError> {
        let model = ActiveModel {
            id: NotSet,
            scene_id: Set(scene_id),
            object_type: Set(object_type.to_string()),
            object_name: Set(name.to_string()),
            properties: Set(properties.to_string()),
        };

        model
            .insert(db)
            .await
            .map_err(|e| SceneError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_object(
        db: &DatabaseConnection,
        scene_id: i32,
        name: &str,
    ) -> Result<(), SceneError> {
        Entity::delete_many()
            .filter(Column::SceneId.eq(scene_id))
            .filter(Column::ObjectName.eq(name))
            .exec(db)
            .await
            .map_err(|e| SceneError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_objects(
        db: &DatabaseConnection,
        scene_id: i32,
        names: &[String],
    ) -> Result<(), SceneError> {
        if names.is_empty() {
            return Ok(());
        }

        Entity::delete_many()
            .filter(Column::SceneId.eq(scene_id))
            .filter(Column::ObjectName.is_in(names))
            .exec(db)
            .await
            .map_err(|e| SceneError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{SceneEntity, SceneModel};
    use chrono::Utc;
    use sea_orm::ActiveValue::Set;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Schema};
    use serde_json::json;

    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();

        // Create tables
        let schema = Schema::new(db.get_database_backend());
        let stmt = schema.create_table_from_entity(crate::db::scene::Entity);
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .unwrap();
        let stmt = schema.create_table_from_entity(Entity);
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .unwrap();

        db
    }

    #[tokio::test]
    async fn test_scene_object_operations() {
        let db = setup_test_db().await;

        // Create a scene
        let scene = crate::db::SceneActiveModel {
            id: Set(1),
            created_at: Set(Utc::now()),
            view: Set("{}".to_string()),
            name: Set("Scene 1".to_string()),
        };
        let scene = scene.insert(&db).await.unwrap();

        // Test saving and retrieving a fixed point
        let properties = json!({
            "value": "10, 20"
        });
        Model::save_object(
            &db,
            scene.id,
            "P1",
            ObjectType::FixedPoint,
            properties.clone(),
        )
        .await
        .unwrap();

        let saved = Entity::find()
            .filter(Column::SceneId.eq(scene.id))
            .filter(Column::ObjectName.eq("P1"))
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        let scene_object = saved.get_scene_object().unwrap();
        match scene_object {
            SceneObject::FixedPoint(p) => {
                assert_eq!(p.x, 10);
                assert_eq!(p.y, 20);
            }
            _ => panic!("Expected FixedPoint"),
        }

        // Test deleting the object
        Model::delete_object(&db, scene.id, "P1").await.unwrap();
        let deleted = Entity::find()
            .filter(Column::SceneId.eq(scene.id))
            .filter(Column::ObjectName.eq("P1"))
            .one(&db)
            .await
            .unwrap();
        assert!(deleted.is_none());
    }
}
