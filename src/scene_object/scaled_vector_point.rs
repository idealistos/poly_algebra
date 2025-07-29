use crate::scene_object::SceneError;
use crate::scene_utils::SceneUtils;
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct ScaledVectorPoint {
    pub k: String,
    pub point1: String,
    pub point2: String,
    pub k_value: f64,
}

impl ScaledVectorPoint {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let k = properties["k"]
            .as_str()
            .ok_or_else(|| {
                SceneError::InvalidProperties("Missing or invalid 'k' property".to_string())
            })?
            .to_string();

        let point1 = properties["point1"]
            .as_str()
            .ok_or_else(|| {
                SceneError::InvalidProperties("Missing or invalid 'point1' property".to_string())
            })?
            .to_string();

        let point2 = properties["point2"]
            .as_str()
            .ok_or_else(|| {
                SceneError::InvalidProperties("Missing or invalid 'point2' property".to_string())
            })?
            .to_string();

        let k_value = properties["k_value"].as_f64().ok_or_else(|| {
            SceneError::InvalidProperties("Missing or invalid 'k_value' property".to_string())
        })?;

        Ok(ScaledVectorPoint {
            k,
            point1,
            point2,
            k_value,
        })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "k": self.k,
            "point1": self.point1,
            "point2": self.point2,
            "k_value": self.k_value,
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        let prepared_k = SceneUtils::prepare_expression(&self.k);
        let point1 = if self.point1.contains(',') {
            format!(
                "FixedPoint({}, {})",
                self.point1.split(',').next().unwrap(),
                self.point1.split(',').nth(1).unwrap()
            )
        } else {
            self.point1.clone()
        };
        let point2 = if self.point2.contains(',') {
            format!(
                "FixedPoint({}, {})",
                self.point2.split(',').next().unwrap(),
                self.point2.split(',').nth(1).unwrap()
            )
        } else {
            self.point2.clone()
        };
        format!(
            "{} = ScaledVectorPoint({}, {}, {})",
            name, prepared_k, point1, point2
        )
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        let mut deps = SceneUtils::extract_identifiers(&self.k).object_names;
        if !self.point1.contains(',') {
            deps.push(self.point1.clone());
        }
        if !self.point2.contains(',') {
            deps.push(self.point2.clone());
        }
        deps.sort();
        deps.dedup();
        deps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaled_vector_point_basic() {
        let props = json!({
            "k": "2",
            "point1": "A",
            "point2": "B",
            "k_value": 2.0
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        assert_eq!(svp.k, "2");
        assert_eq!(svp.point1, "A");
        assert_eq!(svp.point2, "B");
        assert_eq!(svp.k_value, 2.0);
    }

    #[test]
    fn test_scaled_vector_point_with_sqrt() {
        let props = json!({
            "k": "sqrt(2)",
            "point1": "P1",
            "point2": "P2",
            "k_value": 1.4142135623730951
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        assert_eq!(svp.k, "sqrt(2)");
        assert_eq!(svp.point1, "P1");
        assert_eq!(svp.point2, "P2");
        assert_eq!(svp.k_value, 1.4142135623730951);
    }

    #[test]
    fn test_scaled_vector_point_with_variable() {
        let props = json!({
            "k": "t",
            "point1": "X",
            "point2": "Y",
            "k_value": 0.0
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        assert_eq!(svp.k, "t");
        assert_eq!(svp.point1, "X");
        assert_eq!(svp.point2, "Y");
        assert_eq!(svp.k_value, 0.0);
    }

    #[test]
    fn test_scaled_vector_point_with_coordinates() {
        let props = json!({
            "k": "2",
            "point1": "1,2",
            "point2": "3,4",
            "k_value": 2.0
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        assert_eq!(svp.k, "2");
        assert_eq!(svp.point1, "1,2");
        assert_eq!(svp.point2, "3,4");
        assert_eq!(svp.k_value, 2.0);
    }

    #[test]
    fn test_scaled_vector_point_mixed_coordinates_and_names() {
        let props = json!({
            "k": "2*t",
            "point1": "A",
            "point2": "1,0",
            "k_value": 0.0
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        assert_eq!(svp.k, "2*t");
        assert_eq!(svp.point1, "A");
        assert_eq!(svp.point2, "1,0");
        assert_eq!(svp.k_value, 0.0);
    }

    #[test]
    fn test_get_properties() {
        let props = json!({
            "k": "sqrt(2)",
            "point1": "P",
            "point2": "Q",
            "k_value": 1.4142135623730951
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        let expected = json!({
            "k": "sqrt(2)",
            "point1": "P",
            "point2": "Q",
            "k_value": 1.4142135623730951
        });
        assert_eq!(svp.get_properties(), expected);
    }

    #[test]
    fn test_to_python_basic() {
        let props = json!({
            "k": "2",
            "point1": "A",
            "point2": "B",
            "k_value": 2.0
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        assert_eq!(
            svp.to_python("test"),
            "test = ScaledVectorPoint(i(2), A, B)"
        );
    }

    #[test]
    fn test_to_python_with_sqrt() {
        let props = json!({
            "k": "sqrt(2)",
            "point1": "P1",
            "point2": "P2",
            "k_value": 1.4142135623730951
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        assert_eq!(
            svp.to_python("svp"),
            "svp = ScaledVectorPoint(sqrt(i(2)), P1, P2)"
        );
    }

    #[test]
    fn test_to_python_with_variable() {
        let props = json!({
            "k": "t",
            "point1": "X",
            "point2": "Y",
            "k_value": 0.0
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        assert_eq!(svp.to_python("point"), "point = ScaledVectorPoint(t, X, Y)");
    }

    #[test]
    fn test_to_python_with_coordinates() {
        let props = json!({
            "k": "2",
            "point1": "1,2",
            "point2": "3,4",
            "k_value": 2.0
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        assert_eq!(
            svp.to_python("test"),
            "test = ScaledVectorPoint(i(2), FixedPoint(1, 2), FixedPoint(3, 4))"
        );
    }

    #[test]
    fn test_get_dependencies_constant_k() {
        let props = json!({
            "k": "2",
            "point1": "A",
            "point2": "B",
            "k_value": 2.0
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        let deps = svp.get_dependencies();
        assert_eq!(deps, vec!["A", "B"]);
    }

    #[test]
    fn test_get_dependencies_sqrt_k() {
        let props = json!({
            "k": "sqrt(2)",
            "point1": "P1",
            "point2": "P2",
            "k_value": 1.4142135623730951
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        let deps = svp.get_dependencies();
        assert_eq!(deps, vec!["P1", "P2"]);
    }

    #[test]
    fn test_get_dependencies_variable_k() {
        let props = json!({
            "k": "t",
            "point1": "X",
            "point2": "Y",
            "k_value": 0.0
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        let deps = svp.get_dependencies();
        assert_eq!(deps, vec!["X", "Y", "t"]);
    }

    #[test]
    fn test_get_dependencies_complex_k() {
        let props = json!({
            "k": "2*t + sqrt(2)",
            "point1": "A",
            "point2": "B",
            "k_value": 1.4142135623730951
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        let deps = svp.get_dependencies();
        assert_eq!(deps, vec!["A", "B", "t"]);
    }

    #[test]
    fn test_get_dependencies_with_coordinates() {
        let props = json!({
            "k": "2",
            "point1": "1,2",
            "point2": "A",
            "k_value": 2.0
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        let deps = svp.get_dependencies();
        assert_eq!(deps, vec!["A"]);
    }

    #[test]
    fn test_get_dependencies_both_coordinates() {
        let props = json!({
            "k": "t",
            "point1": "0,1",
            "point2": "1,0",
            "k_value": 0.0
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        let deps = svp.get_dependencies();
        assert_eq!(deps, vec!["t"]);
    }

    #[test]
    fn test_get_dependencies_mixed() {
        let props = json!({
            "k": "d(A,B) * t",
            "point1": "P",
            "point2": "1,2",
            "k_value": 2.0
        });
        let svp = ScaledVectorPoint::new(props).unwrap();
        let deps = svp.get_dependencies();
        assert_eq!(deps, vec!["A", "B", "P", "t"]);
    }

    #[test]
    fn test_missing_k_property() {
        let props = json!({
            "point1": "A",
            "point2": "B"
        });
        let result = ScaledVectorPoint::new(props);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing or invalid 'k' property"));
    }

    #[test]
    fn test_missing_point1_property() {
        let props = json!({
            "k": "2",
            "point2": "B"
        });
        let result = ScaledVectorPoint::new(props);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing or invalid 'point1' property"));
    }

    #[test]
    fn test_missing_point2_property() {
        let props = json!({
            "k": "2",
            "point1": "A"
        });
        let result = ScaledVectorPoint::new(props);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing or invalid 'point2' property"));
    }

    #[test]
    fn test_invalid_k_type() {
        let props = json!({
            "k": 123,
            "point1": "A",
            "point2": "B",
            "k_value": 2.0
        });
        let result = ScaledVectorPoint::new(props);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing or invalid 'k' property"));
    }

    #[test]
    fn test_invalid_point1_type() {
        let props = json!({
            "k": "2",
            "point1": 456,
            "point2": "B",
            "k_value": 2.0
        });
        let result = ScaledVectorPoint::new(props);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing or invalid 'point1' property"));
    }

    #[test]
    fn test_invalid_point2_type() {
        let props = json!({
            "k": "2",
            "point1": "A",
            "point2": 789,
            "k_value": 2.0
        });
        let result = ScaledVectorPoint::new(props);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing or invalid 'point2' property"));
    }
}
