use crate::scene_object::SceneError;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    pub point: String,
    pub line: String,
}

impl Projection {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let point = properties
            .get("point")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                SceneError::InvalidProperties("Projection requires 'point' property".to_string())
            })?
            .to_string();

        let line = properties
            .get("line")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                SceneError::InvalidProperties("Projection requires 'line' property".to_string())
            })?
            .to_string();

        Ok(Projection { point, line })
    }

    pub fn get_properties(&self) -> Value {
        serde_json::json!({
            "point": self.point,
            "line": self.line
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        let point = if self.point.contains(',') {
            let coords: Vec<&str> = self.point.split(',').collect();
            format!("FixedPoint({}, {})", coords[0].trim(), coords[1].trim())
        } else {
            self.point.clone()
        };
        format!("{} = Projection({}, {})", name, point, self.line)
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
    use serde_json::json;

    #[test]
    fn test_projection_new_and_properties() {
        let props = json!({
            "point": "P1",
            "line": "L1"
        });
        let proj = Projection::new(props.clone()).unwrap();
        assert_eq!(proj.point, "P1");
        assert_eq!(proj.line, "L1");
        assert_eq!(proj.get_properties(), props);
    }

    #[test]
    fn test_projection_to_python() {
        let proj = Projection {
            point: "A".to_string(),
            line: "l".to_string(),
        };
        assert_eq!(proj.to_python("P"), "P = Projection(A, l)");
    }

    #[test]
    fn test_projection_get_dependencies() {
        let proj = Projection {
            point: "A".to_string(),
            line: "l".to_string(),
        };
        let deps = proj.get_dependencies();
        assert_eq!(deps, vec!["A", "l"]);
    }

    #[test]
    fn test_projection_missing_properties() {
        let props = json!({ "point": "P1" });
        assert!(Projection::new(props).is_err());
        let props = json!({ "line": "L1" });
        assert!(Projection::new(props).is_err());
    }
}
