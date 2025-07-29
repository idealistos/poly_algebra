use crate::scene_object::SceneError;
use crate::scene_utils::SceneUtils;
use serde_json::json;
use serde_json::Value;

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
        let prepared_formula = SceneUtils::prepare_expression(&self.formula);
        format!("is_constant({})", prepared_formula)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        let identifiers = SceneUtils::extract_identifiers(&self.formula);
        return identifiers.object_names;
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

    #[test]
    fn test_to_python_basic() {
        let props = json!({
            "formula": "d(A, B)"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(inv.to_python("test"), "is_constant(d(A, B))");
    }

    #[test]
    fn test_to_python_with_exponentiation() {
        let props = json!({
            "formula": "x^2 + y^2"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(inv.to_python("test"), "is_constant(x**i(2) + y**i(2))");
    }

    #[test]
    fn test_to_python_with_coordinates() {
        let props = json!({
            "formula": "(1, 2) + (3, 4)"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(
            inv.to_python("test"),
            "is_constant((i(1), i(2)) + (i(3), i(4)))"
        );
    }

    #[test]
    fn test_to_python_with_coordinates_and_spaces() {
        let props = json!({
            "formula": "( 1 , 2 ) + ( 3 , 4 )"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(
            inv.to_python("test"),
            "is_constant(( i(1) , i(2) ) + ( i(3) , i(4) ))"
        );
    }

    #[test]
    fn test_to_python_with_standalone_integers() {
        let props = json!({
            "formula": "5 + 10 * 2"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(inv.to_python("test"), "is_constant(i(5) + i(10) * i(2))");
    }

    #[test]
    fn test_to_python_with_mixed_content() {
        let props = json!({
            "formula": "d(A, B)^2 + (1, 2) * 3"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(
            inv.to_python("test"),
            "is_constant(d(A, B)**i(2) + (i(1), i(2)) * i(3))"
        );
    }

    #[test]
    fn test_to_python_with_nested_coordinates() {
        let props = json!({
            "formula": "((1, 2), (3, 4))"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(
            inv.to_python("test"),
            "is_constant(((i(1), i(2)), (i(3), i(4))))"
        );
    }

    #[test]
    fn test_to_python_with_complex_expression() {
        let props = json!({
            "formula": "sqrt((x1 - x2)^2 + (y1 - y2)^2) + 5"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(
            inv.to_python("test"),
            "is_constant(sqrt((x1 - x2)**i(2) + (y1 - y2)**i(2)) + i(5))"
        );
    }

    #[test]
    fn test_to_python_with_negative_numbers() {
        let props = json!({
            "formula": "-5 + (-3)"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(inv.to_python("test"), "is_constant(-i(5) + (-i(3)))");
    }

    #[test]
    fn test_to_python_with_zero() {
        let props = json!({
            "formula": "0 + x"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(inv.to_python("test"), "is_constant(i(0) + x)");
    }

    #[test]
    fn test_to_python_with_large_numbers() {
        let props = json!({
            "formula": "1000000 + 999999"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(inv.to_python("test"), "is_constant(i(1000000) + i(999999))");
    }

    #[test]
    fn test_to_python_with_mixed_coordinate_formats() {
        let props = json!({
            "formula": "(1/2) + ( 3 , 4 ) + (5, 6)"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(
            inv.to_python("test"),
            "is_constant(q(1, 2) + ( i(3) , i(4) ) + (i(5), i(6)))"
        );
    }

    #[test]
    fn test_to_python_with_single_number() {
        let props = json!({
            "formula": "42"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(inv.to_python("test"), "is_constant(i(42))");
    }

    #[test]
    fn test_to_python_with_single_coordinate() {
        let props = json!({
            "formula": "(1, 2)"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(inv.to_python("test"), "is_constant((i(1), i(2)))");
    }

    #[test]
    fn test_to_python_with_single_exponent() {
        let props = json!({
            "formula": "x^2"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(inv.to_python("test"), "is_constant(x**i(2))");
    }

    #[test]
    fn test_to_python_with_fractional_degree() {
        let props = json!({
            "formula": "d(A, B)^(1/2)"
        });
        let inv = Invariant::new(props).unwrap();
        assert_eq!(inv.to_python("test"), "is_constant(d(A, B)**q(1, 2))");
    }
}
