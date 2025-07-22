use crate::scene_object::SceneError;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct PointToLineDistanceInvariant {
    pub point: String,
    pub line: String,
}

impl PointToLineDistanceInvariant {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let point = properties["point"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'point' field".to_string()))?
            .to_string();
        let line = properties["line"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'line' field".to_string()))?
            .to_string();

        Ok(PointToLineDistanceInvariant { point, line })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "point": self.point,
            "line": self.line
        })
    }

    pub fn to_python(&self, _name: &str) -> String {
        let point = if self.point.contains(',') {
            let coords: Vec<&str> = self.point.split(',').collect();
            format!("FixedPoint({}, {})", coords[0].trim(), coords[1].trim())
        } else {
            self.point.clone()
        };

        format!("is_constant(d({}, {}))", point, self.line)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Add point if it's a named point (not coordinates)
        if !self.point.contains(',') {
            dependencies.push(self.point.clone());
        }

        // Add line dependency
        dependencies.push(self.line.clone());

        dependencies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_to_line_distance_invariant() {
        let props = json!({
            "point": "P1",
            "line": "L1"
        });
        let inv = PointToLineDistanceInvariant::new(props).unwrap();
        assert_eq!(inv.point, "P1");
        assert_eq!(inv.line, "L1");
        assert_eq!(
            inv.get_properties(),
            json!({
                "point": "P1",
                "line": "L1"
            })
        );
        assert_eq!(inv.to_python("I1"), "is_constant(d(P1, L1))");
        assert_eq!(inv.get_dependencies(), vec!["P1", "L1"]);
    }
}
