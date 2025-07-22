use crate::scene_object::SceneError;
use regex::Regex;
use serde_json::json;
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Invariant {
    pub formula: String,
}

impl Invariant {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let formula = properties["formula"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'formula' field".to_string()))?
            .to_string();

        Ok(Invariant { formula })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "formula": self.formula
        })
    }

    pub fn to_python(&self, _name: &str) -> String {
        let formula = self.formula.replace("^", "**");
        // Use regex to find standalone integers and wrap them with i()
        let re = Regex::new(r"\b\d+\b").unwrap();
        let formula = re.replace_all(&formula, "i($0)").to_string();
        format!("is_constant({})", formula)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        // Built-in identifiers that should be excluded
        let built_ins: HashSet<&str> = ["d", "d_sqr"].iter().cloned().collect();

        let re = Regex::new(r"[a-zA-Z][a-zA-Z0-9_.]*").unwrap();

        // Extract all identifiers from the formula
        let mut dependencies = HashSet::new();

        for capture in re.find_iter(&self.formula) {
            let identifier = capture.as_str();

            // Remove the part after the first "." symbol
            let base_identifier = if let Some(dot_pos) = identifier.find('.') {
                &identifier[..dot_pos]
            } else {
                identifier
            };

            // Exclude built-in identifiers
            if !built_ins.contains(base_identifier) {
                dependencies.insert(base_identifier.to_string());
            }
        }

        dependencies.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariant() {
        let props = json!({
            "formula": "d(A, B)"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(inv.formula, "d(A, B)");
        assert_eq!(
            inv.get_properties(),
            json!({
                "formula": "d(A, B)"
            })
        );
    }
}
