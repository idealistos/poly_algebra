use regex::Regex;
use serde_json::json;
use serde_json::Value;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;
use thiserror::Error;

// Module declarations for split files
pub mod fixed_point;
pub mod free_point;
pub mod intersection_point;
pub mod invariant;
pub mod line_ab;
pub mod locus;
pub mod midpoint;
pub mod pl_to_line;
pub mod point_to_line_distance_invariant;
pub mod pp_bisector;
pub mod pp_to_line;
pub mod projection;
pub mod reflection;
pub mod sliding_point;
pub mod two_line_angle_invariant;
pub mod two_point_distance_invariant;

// Re-export the structs from the modules
use fixed_point::FixedPoint;
use free_point::FreePoint;
use intersection_point::IntersectionPoint;
use invariant::Invariant;
use line_ab::LineAB;
use locus::Locus;
use midpoint::Midpoint;
use pl_to_line::PlToLine;
use point_to_line_distance_invariant::PointToLineDistanceInvariant;
use pp_bisector::PpBisector;
use pp_to_line::PpToLine;
use projection::Projection;
use reflection::Reflection;
use sliding_point::SlidingPoint;
use two_line_angle_invariant::TwoLineAngleInvariant;
use two_point_distance_invariant::TwoPointDistanceInvariant;

#[derive(Debug, Error)]
pub enum SceneError {
    #[error("Invalid object type: {0}")]
    InvalidObjectType(String),
    #[error("Invalid properties: {0}")]
    InvalidProperties(String),
    #[error("Invalid point format: {0}")]
    InvalidPointFormat(String),
    #[error("Object not found: {0}")]
    ObjectNotFound(String),
    #[error("Referenced object not found: {0}")]
    DependencyNotFound(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Invalid equation: {0}")]
    InvalidEquation(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneObject {
    FixedPoint(FixedPoint),
    FreePoint(FreePoint),
    Midpoint(Midpoint),
    IntersectionPoint(IntersectionPoint),
    SlidingPoint(SlidingPoint),
    Projection(Projection),
    Reflection(Reflection),
    LineAB(LineAB),
    PpBisector(PpBisector),
    PpToLine(PpToLine),
    PlToLine(PlToLine),
    Parameter,
    TwoPointDistanceInvariant(TwoPointDistanceInvariant),
    PointToLineDistanceInvariant(PointToLineDistanceInvariant),
    TwoLineAngleInvariant(TwoLineAngleInvariant),
    Invariant(Invariant),
    Locus(Locus),
}

impl SceneObject {
    pub fn from_properties(object_type: ObjectType, properties: Value) -> Result<Self, SceneError> {
        match object_type {
            ObjectType::FixedPoint => Ok(SceneObject::FixedPoint(FixedPoint::new(properties)?)),
            ObjectType::FreePoint => Ok(SceneObject::FreePoint(FreePoint::new(properties)?)),
            ObjectType::Midpoint => Ok(SceneObject::Midpoint(Midpoint::new(properties)?)),
            ObjectType::IntersectionPoint => Ok(SceneObject::IntersectionPoint(
                IntersectionPoint::new(properties)?,
            )),
            ObjectType::SlidingPoint => {
                Ok(SceneObject::SlidingPoint(SlidingPoint::new(properties)?))
            }
            ObjectType::LineAB => Ok(SceneObject::LineAB(LineAB::new(properties)?)),
            ObjectType::PpBisector => Ok(SceneObject::PpBisector(PpBisector::new(properties)?)),
            ObjectType::PpToLine => Ok(SceneObject::PpToLine(PpToLine::new(properties)?)),
            ObjectType::PlToLine => Ok(SceneObject::PlToLine(PlToLine::new(properties)?)),
            ObjectType::Projection => Ok(SceneObject::Projection(Projection::new(properties)?)),
            ObjectType::Reflection => Ok(SceneObject::Reflection(Reflection::new(properties)?)),
            ObjectType::Parameter => Ok(SceneObject::Parameter),
            ObjectType::TwoPointDistanceInvariant => Ok(SceneObject::TwoPointDistanceInvariant(
                TwoPointDistanceInvariant::new(properties)?,
            )),
            ObjectType::PointToLineDistanceInvariant => {
                Ok(SceneObject::PointToLineDistanceInvariant(
                    PointToLineDistanceInvariant::new(properties)?,
                ))
            }
            ObjectType::TwoLineAngleInvariant => Ok(SceneObject::TwoLineAngleInvariant(
                TwoLineAngleInvariant::new(properties)?,
            )),
            ObjectType::Invariant => Ok(SceneObject::Invariant(Invariant::new(properties)?)),
            ObjectType::Locus => Ok(SceneObject::Locus(Locus::new(properties)?)),
        }
    }

    pub fn get_type(&self) -> ObjectType {
        match self {
            SceneObject::FixedPoint(_) => ObjectType::FixedPoint,
            SceneObject::FreePoint(_) => ObjectType::FreePoint,
            SceneObject::Midpoint(_) => ObjectType::Midpoint,
            SceneObject::IntersectionPoint(_) => ObjectType::IntersectionPoint,
            SceneObject::SlidingPoint(_) => ObjectType::SlidingPoint,
            SceneObject::LineAB(_) => ObjectType::LineAB,
            SceneObject::PpBisector(_) => ObjectType::PpBisector,
            SceneObject::PpToLine(_) => ObjectType::PpToLine,
            SceneObject::PlToLine(_) => ObjectType::PlToLine,
            SceneObject::Projection(_) => ObjectType::Projection,
            SceneObject::Reflection(_) => ObjectType::Reflection,
            SceneObject::Parameter => ObjectType::Parameter,
            SceneObject::TwoPointDistanceInvariant(_) => ObjectType::TwoPointDistanceInvariant,
            SceneObject::PointToLineDistanceInvariant(_) => {
                ObjectType::PointToLineDistanceInvariant
            }
            SceneObject::TwoLineAngleInvariant(_) => ObjectType::TwoLineAngleInvariant,
            SceneObject::Invariant(_) => ObjectType::Invariant,
            SceneObject::Locus(_) => ObjectType::Locus,
        }
    }

    pub fn get_properties(&self) -> Value {
        match self {
            SceneObject::FixedPoint(p) => p.get_properties(),
            SceneObject::FreePoint(p) => p.get_properties(),
            SceneObject::Midpoint(m) => m.get_properties(),
            SceneObject::IntersectionPoint(p) => p.get_properties(),
            SceneObject::SlidingPoint(p) => p.get_properties(),
            SceneObject::LineAB(l) => l.get_properties(),
            SceneObject::PpBisector(p) => p.get_properties(),
            SceneObject::PpToLine(p) => p.get_properties(),
            SceneObject::PlToLine(p) => p.get_properties(),
            SceneObject::Projection(p) => p.get_properties(),
            SceneObject::Reflection(p) => p.get_properties(),
            SceneObject::Parameter => Value::Null,
            SceneObject::TwoPointDistanceInvariant(t) => t.get_properties(),
            SceneObject::PointToLineDistanceInvariant(p) => p.get_properties(),
            SceneObject::TwoLineAngleInvariant(t) => t.get_properties(),
            SceneObject::Invariant(i) => i.get_properties(),
            SceneObject::Locus(p) => p.get_properties(),
        }
    }

    pub fn to_python(&self, name: &str) -> String {
        match self {
            SceneObject::FixedPoint(p) => p.to_python(name),
            SceneObject::FreePoint(p) => p.to_python(name),
            SceneObject::Midpoint(m) => m.to_python(name),
            SceneObject::IntersectionPoint(p) => p.to_python(name),
            SceneObject::SlidingPoint(p) => p.to_python(name),
            SceneObject::LineAB(l) => l.to_python(name),
            SceneObject::PpBisector(p) => p.to_python(name),
            SceneObject::PpToLine(p) => p.to_python(name),
            SceneObject::PlToLine(p) => p.to_python(name),
            SceneObject::Projection(p) => p.to_python(name),
            SceneObject::Reflection(p) => p.to_python(name),
            SceneObject::Parameter => format!("{} = Value(next_var(), 0)", name),
            SceneObject::TwoPointDistanceInvariant(t) => t.to_python(name),
            SceneObject::PointToLineDistanceInvariant(p) => p.to_python(name),
            SceneObject::TwoLineAngleInvariant(t) => t.to_python(name),
            SceneObject::Invariant(i) => i.to_python(name),
            SceneObject::Locus(p) => p.to_python(name),
        }
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        match self {
            SceneObject::FixedPoint(p) => p.get_dependencies(),
            SceneObject::FreePoint(p) => p.get_dependencies(),
            SceneObject::Midpoint(m) => m.get_dependencies(),
            SceneObject::IntersectionPoint(p) => p.get_dependencies(),
            SceneObject::SlidingPoint(p) => p.get_dependencies(),
            SceneObject::LineAB(l) => l.get_dependencies(),
            SceneObject::PpBisector(p) => p.get_dependencies(),
            SceneObject::PpToLine(p) => p.get_dependencies(),
            SceneObject::PlToLine(p) => p.get_dependencies(),
            SceneObject::Projection(p) => p.get_dependencies(),
            SceneObject::Reflection(p) => p.get_dependencies(),
            SceneObject::Parameter => Vec::new(),
            SceneObject::TwoPointDistanceInvariant(t) => t.get_dependencies(),
            SceneObject::PointToLineDistanceInvariant(p) => p.get_dependencies(),
            SceneObject::TwoLineAngleInvariant(t) => t.get_dependencies(),
            SceneObject::Invariant(i) => i.get_dependencies(),
            SceneObject::Locus(p) => p.get_dependencies(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ObjectType {
    FixedPoint,
    FreePoint,
    Midpoint,
    IntersectionPoint,
    SlidingPoint,
    Projection,
    Reflection,
    LineAB,
    PpBisector,
    PpToLine,
    PlToLine,
    TwoPointDistanceInvariant,
    PointToLineDistanceInvariant,
    TwoLineAngleInvariant,
    Invariant,
    Locus,
    Parameter,
}

impl FromStr for ObjectType {
    type Err = SceneError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FixedPoint" => Ok(ObjectType::FixedPoint),
            "FreePoint" => Ok(ObjectType::FreePoint),
            "Midpoint" => Ok(ObjectType::Midpoint),
            "IntersectionPoint" => Ok(ObjectType::IntersectionPoint),
            "SlidingPoint" => Ok(ObjectType::SlidingPoint),
            "Projection" => Ok(ObjectType::Projection),
            "Reflection" => Ok(ObjectType::Reflection),
            "LineAB" => Ok(ObjectType::LineAB),
            "PpBisector" => Ok(ObjectType::PpBisector),
            "PpToLine" => Ok(ObjectType::PpToLine),
            "PlToLine" => Ok(ObjectType::PlToLine),
            "TwoPointDistanceInvariant" => Ok(ObjectType::TwoPointDistanceInvariant),
            "PointToLineDistanceInvariant" => Ok(ObjectType::PointToLineDistanceInvariant),
            "TwoLineAngleInvariant" => Ok(ObjectType::TwoLineAngleInvariant),
            "Invariant" => Ok(ObjectType::Invariant),
            "Locus" => Ok(ObjectType::Locus),
            "Parameter" => Ok(ObjectType::Parameter),
            _ => Err(SceneError::InvalidObjectType(s.to_string())),
        }
    }
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let string_value = match self {
            ObjectType::FixedPoint => "FixedPoint".to_string(),
            ObjectType::FreePoint => "FreePoint".to_string(),
            ObjectType::Midpoint => "Midpoint".to_string(),
            ObjectType::IntersectionPoint => "IntersectionPoint".to_string(),
            ObjectType::SlidingPoint => "SlidingPoint".to_string(),
            ObjectType::Projection => "Projection".to_string(),
            ObjectType::Reflection => "Reflection".to_string(),
            ObjectType::LineAB => "LineAB".to_string(),
            ObjectType::PpBisector => "PpBisector".to_string(),
            ObjectType::PpToLine => "PpToLine".to_string(),
            ObjectType::PlToLine => "PlToLine".to_string(),
            ObjectType::TwoPointDistanceInvariant => "TwoPointDistanceInvariant".to_string(),
            ObjectType::PointToLineDistanceInvariant => "PointToLineDistanceInvariant".to_string(),
            ObjectType::TwoLineAngleInvariant => "TwoLineAngleInvariant".to_string(),
            ObjectType::Invariant => "Invariant".to_string(),
            ObjectType::Locus => "Locus".to_string(),
            ObjectType::Parameter => "Parameter".to_string(),
        };
        write!(f, "{}", string_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_object_conversion() {
        let props = json!({
            "value": "10, 20"
        });
        let obj = SceneObject::from_properties(ObjectType::FixedPoint, props.clone()).unwrap();
        assert!(matches!(obj, SceneObject::FixedPoint(_)));
        assert_eq!(obj.get_type(), ObjectType::FixedPoint);
        assert_eq!(obj.get_properties(), props);

        let props = json!({
            "value": "30, 40"
        });
        let obj = SceneObject::from_properties(ObjectType::FreePoint, props.clone()).unwrap();
        assert!(matches!(obj, SceneObject::FreePoint(_)));
        assert_eq!(obj.get_type(), ObjectType::FreePoint);
        assert_eq!(obj.get_properties(), props);

        let props = json!({
            "value": "5, 0",
            "constraining_object_name": "L1",
        });
        let obj = SceneObject::from_properties(ObjectType::SlidingPoint, props.clone()).unwrap();
        assert!(matches!(obj, SceneObject::SlidingPoint(_)));
        assert_eq!(obj.get_type(), ObjectType::SlidingPoint);
        assert_eq!(obj.get_properties(), props);

        let props = json!({
            "point1": "P1",
            "point2": "P2"
        });
        let obj = SceneObject::from_properties(ObjectType::LineAB, props.clone()).unwrap();
        assert!(matches!(obj, SceneObject::LineAB(_)));
        assert_eq!(obj.get_type(), ObjectType::LineAB);
        assert_eq!(obj.get_properties(), props);

        let props = json!({
            "formula": "d(A, B)"
        });
        let obj = SceneObject::from_properties(ObjectType::Invariant, props.clone()).unwrap();
        assert!(matches!(obj, SceneObject::Invariant(_)));
        assert_eq!(obj.get_type(), ObjectType::Invariant);
        assert_eq!(obj.get_properties(), props);

        let props = json!({
            "point1": "P1",
            "point2": "P2"
        });
        let obj =
            SceneObject::from_properties(ObjectType::TwoPointDistanceInvariant, props.clone())
                .unwrap();
        assert!(matches!(obj, SceneObject::TwoPointDistanceInvariant(_)));
        assert_eq!(obj.get_type(), ObjectType::TwoPointDistanceInvariant);
        assert_eq!(obj.get_properties(), props);

        let props = json!({
            "point": "P1",
            "line": "L1"
        });
        let obj =
            SceneObject::from_properties(ObjectType::PointToLineDistanceInvariant, props.clone())
                .unwrap();
        assert!(matches!(obj, SceneObject::PointToLineDistanceInvariant(_)));
        assert_eq!(obj.get_type(), ObjectType::PointToLineDistanceInvariant);
        assert_eq!(obj.get_properties(), props);

        let props = json!({
            "object_name_1": "L1",
            "object_name_2": "C1"
        });
        let obj =
            SceneObject::from_properties(ObjectType::IntersectionPoint, props.clone()).unwrap();
        assert!(matches!(obj, SceneObject::IntersectionPoint(_)));
        assert_eq!(obj.get_type(), ObjectType::IntersectionPoint);
        assert_eq!(obj.get_properties(), props);

        let locus = Locus {
            point: "P1".to_string(),
        };
        assert_eq!(locus.get_properties(), json!({ "point": "P1" }));

        let midpoint = Midpoint {
            point1: "P1".to_string(),
            point2: "P2".to_string(),
        };
        assert_eq!(
            midpoint.get_properties(),
            json!({ "point1": "P1", "point2": "P2" })
        );
    }

    #[test]
    fn test_python_generation() {
        let fixed = FixedPoint { x: 10, y: 20 };
        assert_eq!(fixed.to_python("P1"), "P1 = FixedPoint(10, 20)");

        let free = FreePoint { x: 30, y: 40 };
        assert_eq!(free.to_python("P2"), "P2 = FreePoint(30, 40)");

        let line = LineAB {
            point1: "P1".to_string(),
            point2: "P2".to_string(),
        };
        assert_eq!(line.to_python("L1"), "L1 = LineAB(P1, P2)");

        let sliding = SlidingPoint {
            x: 20,
            y: 30,
            constraining_object_name: "L1".to_string(),
        };
        assert_eq!(
            sliding.to_python("P3"),
            "P3 = FreePoint(20, 30)\nL1.contains(P3)"
        );

        let inv = Invariant {
            formula: "d(A, B)".to_string(),
        };
        assert_eq!(inv.to_python("I1"), "is_constant(d(A, B))");

        let two_point_inv = TwoPointDistanceInvariant {
            point1: "P1".to_string(),
            point2: "P2".to_string(),
        };
        assert_eq!(two_point_inv.to_python("I2"), "is_constant(d(P1, P2))");

        let point_to_line_inv = PointToLineDistanceInvariant {
            point: "P1".to_string(),
            line: "L1".to_string(),
        };
        assert_eq!(point_to_line_inv.to_python("I3"), "is_constant(d(P1, L1))");

        let two_line_angle_inv = TwoLineAngleInvariant {
            line1: "L1".to_string(),
            line2: "L2".to_string(),
        };
        assert_eq!(
            two_line_angle_inv.to_python("I4"),
            "is_constant(cot(L1.n, L2.n).abs())"
        );

        // Test through SceneObject
        let obj = SceneObject::FixedPoint(fixed);
        assert_eq!(obj.to_python("P1"), "P1 = FixedPoint(10, 20)");

        let obj = SceneObject::FreePoint(free);
        assert_eq!(obj.to_python("P2"), "P2 = FreePoint(30, 40)");

        let obj = SceneObject::SlidingPoint(sliding);
        assert_eq!(
            obj.to_python("P3"),
            "P3 = FreePoint(20, 30)\nL1.contains(P3)"
        );

        let obj = SceneObject::LineAB(line);
        assert_eq!(obj.to_python("L1"), "L1 = LineAB(P1, P2)");

        let obj = SceneObject::Invariant(inv);
        assert_eq!(obj.to_python("I1"), "is_constant(d(A, B))");

        let midpoint = Midpoint {
            point1: "P1".to_string(),
            point2: "P2".to_string(),
        };
        assert_eq!(
            midpoint.to_python("M1"),
            "M1 = Point(Value(next_var(), initial=Value(next_var())), Value(next_var(), initial=Value(next_var())))\nis_zero_vector((P1 - M1) + (P2 - M1))"
        );

        let obj = SceneObject::Midpoint(midpoint);
        assert_eq!(
            obj.to_python("M1"),
            "M1 = Point(Value(next_var(), initial=Value(next_var())), Value(next_var(), initial=Value(next_var())))\nis_zero_vector((P1 - M1) + (P2 - M1))"
        );
    }

    #[test]
    fn test_get_dependencies() {
        // Test FixedPoint - should return empty list
        let fixed = FixedPoint { x: 10, y: 20 };
        assert_eq!(fixed.get_dependencies(), Vec::<String>::new());

        let obj = SceneObject::FixedPoint(fixed);
        assert_eq!(obj.get_dependencies(), Vec::<String>::new());

        // Test FreePoint - should return empty list
        let free = FreePoint { x: 30, y: 40 };
        assert_eq!(free.get_dependencies(), Vec::<String>::new());

        let obj = SceneObject::FreePoint(free);
        assert_eq!(obj.get_dependencies(), Vec::<String>::new());

        // Test SlidingPoint - should return empty list
        let sliding = SlidingPoint {
            x: 50,
            y: 60,
            constraining_object_name: "L1".to_string(),
        };
        assert_eq!(sliding.get_dependencies(), vec!["L1"]);

        let obj = SceneObject::SlidingPoint(sliding);
        assert_eq!(obj.get_dependencies(), vec!["L1"]);

        // Test IntersectionPoint - should return both object names
        let intersection = IntersectionPoint {
            object_name_1: "L1".to_string(),
            object_name_2: "C1".to_string(),
        };
        let mut expected = vec!["L1".to_string(), "C1".to_string()];
        expected.sort();
        let mut actual = intersection.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        let obj = SceneObject::IntersectionPoint(intersection);
        let mut actual = obj.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        // Test LineAB with named points
        let line = LineAB {
            point1: "P1".to_string(),
            point2: "P2".to_string(),
        };
        let mut expected = vec!["P1".to_string(), "P2".to_string()];
        expected.sort();
        let mut actual = line.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        let obj = SceneObject::LineAB(line);
        let mut actual = obj.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        // Test LineAB with mixed named and coordinate points
        let line = LineAB {
            point1: "P1".to_string(),
            point2: "10, 20".to_string(),
        };
        assert_eq!(line.get_dependencies(), vec!["P1".to_string()]);

        let obj = SceneObject::LineAB(line);
        assert_eq!(obj.get_dependencies(), vec!["P1".to_string()]);

        // Test LineAB with coordinate points only
        let line = LineAB {
            point1: "10, 20".to_string(),
            point2: "30, 40".to_string(),
        };
        assert_eq!(line.get_dependencies(), Vec::<String>::new());

        let obj = SceneObject::LineAB(line);
        assert_eq!(obj.get_dependencies(), Vec::<String>::new());

        // Test Invariant with simple identifiers
        let inv = Invariant {
            formula: "d(A, B)".to_string(),
        };
        let mut expected = vec!["A".to_string(), "B".to_string()];
        expected.sort();
        let mut actual = inv.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        let obj = SceneObject::Invariant(inv);
        let mut actual = obj.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        // Test Invariant with field access
        let inv = Invariant {
            formula: "d(A.x, B.y)".to_string(),
        };
        let mut expected = vec!["A".to_string(), "B".to_string()];
        expected.sort();
        let mut actual = inv.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        let obj = SceneObject::Invariant(inv);
        let mut actual = obj.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        // Test Invariant with complex formula
        let inv = Invariant {
            formula: "d(A, B) + d(C, D)".to_string(),
        };
        let mut expected = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ];
        expected.sort();
        let mut actual = inv.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        let obj = SceneObject::Invariant(inv);
        let mut actual = obj.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        // Test Invariant with built-in function (should exclude 'd')
        let inv = Invariant {
            formula: "d(A, B) + e(C, D)".to_string(),
        };
        let mut expected = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "e".to_string(),
        ];
        expected.sort();
        let mut actual = inv.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        let obj = SceneObject::Invariant(inv);
        let mut actual = obj.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        // Test Locus
        let locus = Locus {
            point: "P1".to_string(),
        };
        assert_eq!(locus.get_dependencies(), vec!["P1".to_string()]);

        let obj = SceneObject::Locus(locus);
        assert_eq!(obj.get_dependencies(), vec!["P1".to_string()]);

        // Test Midpoint with named points
        let midpoint = Midpoint {
            point1: "P1".to_string(),
            point2: "P2".to_string(),
        };
        let mut expected = vec!["P1".to_string(), "P2".to_string()];
        expected.sort();
        let mut actual = midpoint.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        let obj = SceneObject::Midpoint(midpoint);
        let mut actual = obj.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        // Test Midpoint with mixed named and coordinate points
        let midpoint = Midpoint {
            point1: "P1".to_string(),
            point2: "10, 20".to_string(),
        };
        assert_eq!(midpoint.get_dependencies(), vec!["P1".to_string()]);

        let obj = SceneObject::Midpoint(midpoint);
        assert_eq!(obj.get_dependencies(), vec!["P1".to_string()]);

        // Test Midpoint with coordinate points only
        let midpoint = Midpoint {
            point1: "10, 20".to_string(),
            point2: "30, 40".to_string(),
        };
        assert_eq!(midpoint.get_dependencies(), Vec::<String>::new());

        let obj = SceneObject::Midpoint(midpoint);
        assert_eq!(obj.get_dependencies(), Vec::<String>::new());

        // Test TwoPointDistanceInvariant with named points
        let two_point_inv = TwoPointDistanceInvariant {
            point1: "P1".to_string(),
            point2: "P2".to_string(),
        };
        let mut expected = vec!["P1".to_string(), "P2".to_string()];
        expected.sort();
        let mut actual = two_point_inv.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        let obj = SceneObject::TwoPointDistanceInvariant(two_point_inv);
        let mut actual = obj.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        // Test TwoPointDistanceInvariant with mixed named and coordinate points
        let two_point_inv = TwoPointDistanceInvariant {
            point1: "P1".to_string(),
            point2: "10, 20".to_string(),
        };
        assert_eq!(two_point_inv.get_dependencies(), vec!["P1".to_string()]);

        let obj = SceneObject::TwoPointDistanceInvariant(two_point_inv);
        assert_eq!(obj.get_dependencies(), vec!["P1".to_string()]);

        // Test TwoPointDistanceInvariant with coordinate points only
        let two_point_inv = TwoPointDistanceInvariant {
            point1: "10, 20".to_string(),
            point2: "30, 40".to_string(),
        };
        assert_eq!(two_point_inv.get_dependencies(), Vec::<String>::new());

        let obj = SceneObject::TwoPointDistanceInvariant(two_point_inv);
        assert_eq!(obj.get_dependencies(), Vec::<String>::new());

        // Test PointToLineDistanceInvariant with named points
        let point_to_line_inv = PointToLineDistanceInvariant {
            point: "P1".to_string(),
            line: "L1".to_string(),
        };
        let mut expected = vec!["P1".to_string(), "L1".to_string()];
        expected.sort();
        let mut actual = point_to_line_inv.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        let obj = SceneObject::PointToLineDistanceInvariant(point_to_line_inv);
        let mut actual = obj.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        // Test PointToLineDistanceInvariant with coordinate points only
        let point_to_line_inv = PointToLineDistanceInvariant {
            point: "10, 20".to_string(),
            line: "L1".to_string(),
        };
        assert_eq!(point_to_line_inv.get_dependencies(), vec!["L1".to_string()]);

        let obj = SceneObject::PointToLineDistanceInvariant(point_to_line_inv);
        assert_eq!(obj.get_dependencies(), vec!["L1".to_string()]);

        // Test TwoLineAngleInvariant with named lines
        let two_line_angle_inv = TwoLineAngleInvariant {
            line1: "L1".to_string(),
            line2: "L2".to_string(),
        };
        let mut expected = vec!["L1".to_string(), "L2".to_string()];
        expected.sort();
        let mut actual = two_line_angle_inv.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);

        let obj = SceneObject::TwoLineAngleInvariant(two_line_angle_inv);
        let mut actual = obj.get_dependencies();
        actual.sort();
        assert_eq!(actual, expected);
    }
}
