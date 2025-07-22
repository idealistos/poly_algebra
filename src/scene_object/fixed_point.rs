use crate::scene_object::SceneError;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct FixedPoint {
    pub x: i64,
    pub y: i64,
}

impl FixedPoint {
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

        Ok(FixedPoint { x, y })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "value": format!("{}, {}", self.x, self.y)
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        format!("{} = FixedPoint({}, {})", name, self.x, self.y)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_point() {
        let properties = json!({
            "value": "3, 4"
        });
        let fixed_point = FixedPoint::new(properties).unwrap();
        assert_eq!(fixed_point.x, 3);
        assert_eq!(fixed_point.y, 4);

        let properties = FixedPoint::get_properties(&fixed_point);
        assert_eq!(properties["value"], "3, 4");

        let python = fixed_point.to_python("A");
        assert_eq!(python, "A = FixedPoint(3, 4)");

        let dependencies = fixed_point.get_dependencies();
        assert_eq!(dependencies, Vec::<String>::new());
    }
}
