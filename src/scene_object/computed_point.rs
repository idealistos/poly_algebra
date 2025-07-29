use crate::scene_object::SceneError;
use crate::scene_utils::SceneUtils;
use serde_json::json;
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedPoint {
    pub x_expr: String,
    pub y_expr: String,
    pub value: String,
}

impl ComputedPoint {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let x_expr = properties["x_expr"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'x_expr' field".to_string()))?
            .to_string();

        let y_expr = properties["y_expr"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'y_expr' field".to_string()))?
            .to_string();

        let value = properties["value"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'value' field".to_string()))?
            .to_string();

        Ok(ComputedPoint {
            x_expr,
            y_expr,
            value,
        })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "x_expr": self.x_expr,
            "y_expr": self.y_expr,
            "value": self.value
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        let x_expr = SceneUtils::prepare_expression(&self.x_expr);
        let y_expr = SceneUtils::prepare_expression(&self.y_expr);
        format!("{} = Point({}, {})", name, x_expr, y_expr)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        // Extract identifiers from x_expr and y_expr
        let x_identifiers = SceneUtils::extract_identifiers(&self.x_expr);
        let y_identifiers = SceneUtils::extract_identifiers(&self.y_expr);

        // Merge object names from both expressions using HashSet to avoid duplicates
        let mut dependencies = HashSet::new();
        dependencies.extend(x_identifiers.object_names);
        dependencies.extend(y_identifiers.object_names);

        // Convert to sorted Vec
        let mut result: Vec<String> = dependencies.into_iter().collect();
        result.sort();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_computed_point() {
        let properties = json!({
            "x_expr": "A.x + 1",
            "y_expr": "B.y * 2",
            "value": "3, 4"
        });
        let computed_point = ComputedPoint::new(properties).unwrap();
        assert_eq!(computed_point.x_expr, "A.x + 1");
        assert_eq!(computed_point.y_expr, "B.y * 2");
        assert_eq!(computed_point.value, "3, 4");

        let properties = ComputedPoint::get_properties(&computed_point);
        assert_eq!(properties["x_expr"], "A.x + 1");
        assert_eq!(properties["y_expr"], "B.y * 2");
        assert_eq!(properties["value"], "3, 4");

        let python = computed_point.to_python("C");
        assert_eq!(python, "C = Point(A.x + i(1), B.y * i(2))");

        let dependencies = computed_point.get_dependencies();
        assert_eq!(dependencies, vec!["A", "B"]);
    }

    #[test]
    fn test_computed_point_missing_properties() {
        // Test missing x_expr
        let properties = json!({
            "y_expr": "B.y * 2",
            "value": "3, 4"
        });
        let result = ComputedPoint::new(properties);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SceneError::InvalidProperties(_)
        ));

        // Test missing y_expr
        let properties = json!({
            "x_expr": "A.x + 1",
            "value": "3, 4"
        });
        let result = ComputedPoint::new(properties);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SceneError::InvalidProperties(_)
        ));

        // Test missing value
        let properties = json!({
            "x_expr": "A.x + 1",
            "y_expr": "B.y * 2"
        });
        let result = ComputedPoint::new(properties);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SceneError::InvalidProperties(_)
        ));
    }

    #[test]
    fn test_get_dependencies() {
        // Test with simple expressions
        let properties = json!({
            "x_expr": "A.x",
            "y_expr": "B.y",
            "value": "1, 2"
        });
        let computed_point = ComputedPoint::new(properties).unwrap();
        let dependencies = computed_point.get_dependencies();
        assert_eq!(dependencies, vec!["A", "B"]);

        // Test with complex expressions
        let properties = json!({
            "x_expr": "A.x + B.x + C.x",
            "y_expr": "B.y + D.y",
            "value": "1, 2"
        });
        let computed_point = ComputedPoint::new(properties).unwrap();
        let dependencies = computed_point.get_dependencies();
        assert_eq!(dependencies, vec!["A", "B", "C", "D"]);

        // Test with duplicate objects
        let properties = json!({
            "x_expr": "A.x + B.x",
            "y_expr": "B.y + A.y",
            "value": "1, 2"
        });
        let computed_point = ComputedPoint::new(properties).unwrap();
        let dependencies = computed_point.get_dependencies();
        assert_eq!(dependencies, vec!["A", "B"]);

        // Test with function calls
        let properties = json!({
            "x_expr": "d(A, B)",
            "y_expr": "Point(C, D).x",
            "value": "1, 2"
        });
        let computed_point = ComputedPoint::new(properties).unwrap();
        let dependencies = computed_point.get_dependencies();
        assert_eq!(dependencies, vec!["A", "B", "C", "D"]);

        // Test with method calls
        let properties = json!({
            "x_expr": "A.abs()",
            "y_expr": "B.length()",
            "value": "1, 2"
        });
        let computed_point = ComputedPoint::new(properties).unwrap();
        let dependencies = computed_point.get_dependencies();
        assert_eq!(dependencies, vec!["A", "B"]);
    }
}
