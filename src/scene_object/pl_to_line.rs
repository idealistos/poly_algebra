use crate::scene_object::SceneError;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct PlToLine {
    pub point: String,
    pub line: String,
}

impl PlToLine {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let point = properties["point"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'point' field".to_string()))?
            .to_string();
        let line = properties["line"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'line' field".to_string()))?
            .to_string();

        Ok(PlToLine { point, line })
    }

    pub fn get_properties(&self) -> Value {
        json!({
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

        format!("{} = PlToLine({}, {})", name, point, self.line)
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
    fn test_pl_to_line() {
        let props = json!({
            "point": "P1",
            "line": "L1"
        });
        let pl_to_line = PlToLine::new(props).unwrap();
        assert_eq!(pl_to_line.point, "P1");
        assert_eq!(pl_to_line.line, "L1");
        assert_eq!(
            pl_to_line.get_properties(),
            json!({
                "point": "P1",
                "line": "L1"
            })
        );
        assert_eq!(pl_to_line.get_dependencies(), vec!["P1", "L1"]);
    }

    #[test]
    fn test_pl_to_line_with_coordinate_point() {
        let props = json!({
            "point": "10, 20",
            "line": "L1"
        });
        let pl_to_line = PlToLine::new(props).unwrap();
        assert_eq!(pl_to_line.point, "10, 20");
        assert_eq!(pl_to_line.line, "L1");
        assert_eq!(
            pl_to_line.get_properties(),
            json!({
                "point": "10, 20",
                "line": "L1"
            })
        );
        assert_eq!(pl_to_line.get_dependencies(), vec!["L1"]);
    }

    #[test]
    fn test_pl_to_line_missing_point() {
        let props = json!({
            "line": "L1"
        });
        let result = PlToLine::new(props);
        assert!(result.is_err());
    }

    #[test]
    fn test_pl_to_line_missing_line() {
        let props = json!({
            "point": "P1"
        });
        let result = PlToLine::new(props);
        assert!(result.is_err());
    }

    #[test]
    fn test_pl_to_line_to_python() {
        let pl_to_line = PlToLine {
            point: "P1".to_string(),
            line: "L1".to_string(),
        };
        assert_eq!(pl_to_line.to_python("PT1"), "PT1 = PlToLine(P1, L1)");
    }

    #[test]
    fn test_pl_to_line_to_python_with_coordinates() {
        let pl_to_line = PlToLine {
            point: "10, 20".to_string(),
            line: "L1".to_string(),
        };
        assert_eq!(
            pl_to_line.to_python("PT1"),
            "PT1 = PlToLine(FixedPoint(10, 20), L1)"
        );
    }
}
