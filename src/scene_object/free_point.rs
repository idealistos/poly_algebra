use crate::scene_object::SceneError;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct FreePoint {
    pub x: i64,
    pub y: i64,
}

impl FreePoint {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let value = properties["value"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'value' field".to_string()))?;

        let coords: Vec<&str> = value.split(',').collect();
        if coords.len() != 2 {
            return Err(SceneError::InvalidPointFormat(value.to_string()));
        }

        let x = coords[0]
            .trim()
            .parse::<i64>()
            .map_err(|_| SceneError::InvalidPointFormat(coords[0].to_string()))?;
        let y = coords[1]
            .trim()
            .parse::<i64>()
            .map_err(|_| SceneError::InvalidPointFormat(coords[1].to_string()))?;

        Ok(FreePoint { x, y })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "value": format!("{}, {}", self.x, self.y)
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        format!("{} = FreePoint({}, {})", name, self.x, self.y)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_point() {
        let properties = json!({
            "value": "5, 6"
        });
        let free_point = FreePoint::new(properties).unwrap();
        assert_eq!(free_point.x, 5);
        assert_eq!(free_point.y, 6);

        let properties = FreePoint::get_properties(&free_point);
        assert_eq!(properties["value"], "5, 6");

        let python = free_point.to_python("B");
        assert_eq!(python, "B = FreePoint(5, 6)");

        let dependencies = free_point.get_dependencies();
        assert_eq!(dependencies, Vec::<String>::new());
    }
}
