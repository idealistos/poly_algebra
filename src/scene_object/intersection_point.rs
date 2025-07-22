use crate::scene_object::SceneError;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct IntersectionPoint {
    pub object_name_1: String,
    pub object_name_2: String,
}

impl IntersectionPoint {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let object_name_1 = properties["object_name_1"]
            .as_str()
            .ok_or_else(|| {
                SceneError::InvalidProperties("Missing 'object_name_1' field".to_string())
            })?
            .to_string();
        let object_name_2 = properties["object_name_2"]
            .as_str()
            .ok_or_else(|| {
                SceneError::InvalidProperties("Missing 'object_name_2' field".to_string())
            })?
            .to_string();

        Ok(IntersectionPoint {
            object_name_1,
            object_name_2,
        })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "object_name_1": self.object_name_1,
            "object_name_2": self.object_name_2
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        let new_value = "Value(next_var(), initial=Value(next_var()))";
        let line1 = format!("{} = Point({}, {})", name, new_value, new_value);
        let line2 = format!("{}.contains({})", self.object_name_1, name);
        let line3 = format!("{}.contains({})", self.object_name_2, name);
        format!("{}\n{}\n{}", line1, line2, line3)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        vec![self.object_name_1.clone(), self.object_name_2.clone()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection_point() {
        let props = json!({
            "object_name_1": "L1",
            "object_name_2": "C1"
        });
        let point = IntersectionPoint::new(props).unwrap();
        assert_eq!(point.object_name_1, "L1");
        assert_eq!(point.object_name_2, "C1");
        assert_eq!(
            point.get_properties(),
            json!({
                "object_name_1": "L1",
                "object_name_2": "C1"
            })
        );
    }
}
