use crate::scene_object::SceneError;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Midpoint {
    pub point1: String,
    pub point2: String,
}

impl Midpoint {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let point1 = properties["point1"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'point1' field".to_string()))?
            .to_string();
        let point2 = properties["point2"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'point2' field".to_string()))?
            .to_string();

        Ok(Midpoint { point1, point2 })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "point1": self.point1,
            "point2": self.point2
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        let point1 = if self.point1.contains(',') {
            let coords: Vec<&str> = self.point1.split(',').collect();
            format!("FixedPoint({}, {})", coords[0].trim(), coords[1].trim())
        } else {
            self.point1.clone()
        };

        let point2 = if self.point2.contains(',') {
            let coords: Vec<&str> = self.point2.split(',').collect();
            format!("FixedPoint({}, {})", coords[0].trim(), coords[1].trim())
        } else {
            self.point2.clone()
        };

        let new_value = "Value(next_var(), initial=Value(next_var()))";
        let line1 = format!("{} = Point({}, {})", name, new_value, new_value);
        let line2 = format!(
            "is_zero_vector(({} - {}) + ({} - {}))",
            point1, name, point2, name
        );
        format!("{}\n{}", line1, line2)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Add point1 if it's a named point (not coordinates)
        if !self.point1.contains(',') {
            dependencies.push(self.point1.clone());
        }

        // Add point2 if it's a named point (not coordinates)
        if !self.point2.contains(',') {
            dependencies.push(self.point2.clone());
        }

        dependencies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midpoint() {
        let props = json!({
            "point1": "P1",
            "point2": "P2"
        });
        let midpoint = Midpoint::new(props).unwrap();
        assert_eq!(midpoint.point1, "P1");
        assert_eq!(midpoint.point2, "P2");
        assert_eq!(
            midpoint.get_properties(),
            json!({
                "point1": "P1",
                "point2": "P2"
            })
        );
    }
}
