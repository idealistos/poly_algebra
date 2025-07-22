use crate::scene_object::SceneError;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct SlidingPoint {
    pub x: i64,
    pub y: i64,
    pub constraining_object_name: String,
}

impl SlidingPoint {
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

        let constraining_object_name = properties["constraining_object_name"]
            .as_str()
            .ok_or_else(|| {
                SceneError::InvalidProperties(
                    "Missing 'constraining_object_name' field".to_string(),
                )
            })?
            .to_string();

        Ok(SlidingPoint {
            x,
            y,
            constraining_object_name,
        })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "value": format!("{}, {}", self.x, self.y),
            "constraining_object_name": self.constraining_object_name,
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        format!(
            "{} = FreePoint({}, {})\n{}.contains({})",
            name, self.x, self.y, self.constraining_object_name, name
        )
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        vec![self.constraining_object_name.clone()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_point() {
        let props = json!({
            "value": "5,0",
            "constraining_object_name": "L1",
        });
        let point = SlidingPoint::new(props).unwrap();
        assert_eq!(point.x, 5);
        assert_eq!(point.y, 0);
        assert_eq!(
            point.get_properties(),
            json!({
                "value": "5, 0",
                "constraining_object_name": "L1",
            })
        );
    }
}
