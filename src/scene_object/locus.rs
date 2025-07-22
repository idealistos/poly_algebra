use crate::scene_object::SceneError;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Locus {
    pub point: String,
}

impl Locus {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let point = properties
            .get("point")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'point' property".to_string()))?;

        Ok(Self {
            point: point.to_string(),
        })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "point": self.point
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        format!("plot(\"{}\", {})", name, self.point)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        vec![self.point.clone()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locus() {
        let props = json!({
            "point": "P1"
        });
        let locus = Locus::new(props).unwrap();
        assert_eq!(locus.point, "P1");
        assert_eq!(
            locus.get_properties(),
            json!({
                "point": "P1"
            })
        );
    }
}
