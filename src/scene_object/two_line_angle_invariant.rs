use crate::scene_object::SceneError;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct TwoLineAngleInvariant {
    pub line1: String,
    pub line2: String,
}

impl TwoLineAngleInvariant {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let line1 = properties["line1"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'line1' field".to_string()))?
            .to_string();
        let line2 = properties["line2"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'line2' field".to_string()))?
            .to_string();

        Ok(TwoLineAngleInvariant { line1, line2 })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "line1": self.line1,
            "line2": self.line2
        })
    }

    pub fn to_python(&self, _name: &str) -> String {
        format!("is_constant(cot({}.n, {}.n).abs())", self.line1, self.line2)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        vec![self.line1.clone(), self.line2.clone()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_line_angle_invariant() {
        let props = json!({
            "line1": "L1",
            "line2": "L2"
        });
        let inv = TwoLineAngleInvariant::new(props).unwrap();
        assert_eq!(inv.line1, "L1");
        assert_eq!(inv.line2, "L2");
        assert_eq!(
            inv.get_properties(),
            json!({
                "line1": "L1",
                "line2": "L2"
            })
        );
        assert_eq!(inv.to_python("I1"), "is_constant(cot(L1.n, L2.n).abs())");
        assert_eq!(inv.get_dependencies(), vec!["L1", "L2"]);
    }
}
