use regex::Regex;
use serde_json::json;
use serde_json::Value;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;
use thiserror::Error;

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
    SlidingPoint(SlidingPoint),
    IntersectionPoint(IntersectionPoint),
    Midpoint(Midpoint),
    LineAB(LineAB),
    Parameter,
    Invariant(Invariant),
    Locus(Locus),
}

impl SceneObject {
    pub fn from_properties(object_type: ObjectType, properties: Value) -> Result<Self, SceneError> {
        match object_type {
            ObjectType::FixedPoint => Ok(SceneObject::FixedPoint(FixedPoint::new(properties)?)),
            ObjectType::FreePoint => Ok(SceneObject::FreePoint(FreePoint::new(properties)?)),
            ObjectType::SlidingPoint => {
                Ok(SceneObject::SlidingPoint(SlidingPoint::new(properties)?))
            }
            ObjectType::IntersectionPoint => Ok(SceneObject::IntersectionPoint(
                IntersectionPoint::new(properties)?,
            )),
            ObjectType::Midpoint => Ok(SceneObject::Midpoint(Midpoint::new(properties)?)),
            ObjectType::LineAB => Ok(SceneObject::LineAB(LineAB::new(properties)?)),
            ObjectType::Parameter => Ok(SceneObject::Parameter),
            ObjectType::Invariant => Ok(SceneObject::Invariant(Invariant::new(properties)?)),
            ObjectType::Locus => Ok(SceneObject::Locus(Locus::new(properties)?)),
        }
    }

    pub fn get_type(&self) -> ObjectType {
        match self {
            SceneObject::FixedPoint(_) => ObjectType::FixedPoint,
            SceneObject::FreePoint(_) => ObjectType::FreePoint,
            SceneObject::SlidingPoint(_) => ObjectType::SlidingPoint,
            SceneObject::IntersectionPoint(_) => ObjectType::IntersectionPoint,
            SceneObject::Midpoint(_) => ObjectType::Midpoint,
            SceneObject::LineAB(_) => ObjectType::LineAB,
            SceneObject::Parameter => ObjectType::Parameter,
            SceneObject::Invariant(_) => ObjectType::Invariant,
            SceneObject::Locus(_) => ObjectType::Locus,
        }
    }

    pub fn get_properties(&self) -> Value {
        match self {
            SceneObject::FixedPoint(p) => p.get_properties(),
            SceneObject::FreePoint(p) => p.get_properties(),
            SceneObject::SlidingPoint(p) => p.get_properties(),
            SceneObject::IntersectionPoint(p) => p.get_properties(),
            SceneObject::Midpoint(m) => m.get_properties(),
            SceneObject::LineAB(l) => l.get_properties(),
            SceneObject::Parameter => Value::Null,
            SceneObject::Invariant(i) => i.get_properties(),
            SceneObject::Locus(p) => p.get_properties(),
        }
    }

    pub fn to_python(&self, name: &str) -> String {
        match self {
            SceneObject::FixedPoint(p) => p.to_python(name),
            SceneObject::FreePoint(p) => p.to_python(name),
            SceneObject::SlidingPoint(p) => p.to_python(name),
            SceneObject::IntersectionPoint(p) => p.to_python(name),
            SceneObject::Midpoint(m) => m.to_python(name),
            SceneObject::LineAB(l) => l.to_python(name),
            SceneObject::Parameter => format!("{} = Value(next_var(), 0)", name),
            SceneObject::Invariant(i) => i.to_python(name),
            SceneObject::Locus(p) => p.to_python(name),
        }
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        match self {
            SceneObject::FixedPoint(p) => p.get_dependencies(),
            SceneObject::FreePoint(p) => p.get_dependencies(),
            SceneObject::SlidingPoint(p) => p.get_dependencies(),
            SceneObject::IntersectionPoint(p) => p.get_dependencies(),
            SceneObject::Midpoint(m) => m.get_dependencies(),
            SceneObject::LineAB(l) => l.get_dependencies(),
            SceneObject::Parameter => Vec::new(),
            SceneObject::Invariant(i) => i.get_dependencies(),
            SceneObject::Locus(p) => p.get_dependencies(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixedPoint {
    pub x: i64,
    pub y: i64,
}

impl FixedPoint {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let value = properties["value"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'value' field".to_string()))?;

        let coords: Vec<&str> = value.split(',').collect();
        if coords.len() != 2 {
            return Err(SceneError::InvalidPointFormat(value.to_string()));
        }

        let x = coords[0]
            .trim()
            .parse::<i64>()
            .map_err(|_| SceneError::InvalidPointFormat(coords[0].to_string()))?;
        let y = coords[1]
            .trim()
            .parse::<i64>()
            .map_err(|_| SceneError::InvalidPointFormat(coords[1].to_string()))?;

        Ok(FixedPoint { x, y })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "value": format!("{}, {}", self.x, self.y)
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        format!("{} = FixedPoint({}, {})", name, self.x, self.y)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreePoint {
    pub x: i64,
    pub y: i64,
}

impl FreePoint {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let value = properties["value"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'value' field".to_string()))?;

        let coords: Vec<&str> = value.split(',').collect();
        if coords.len() != 2 {
            return Err(SceneError::InvalidPointFormat(value.to_string()));
        }

        let x = coords[0]
            .trim()
            .parse::<i64>()
            .map_err(|_| SceneError::InvalidPointFormat(coords[0].to_string()))?;
        let y = coords[1]
            .trim()
            .parse::<i64>()
            .map_err(|_| SceneError::InvalidPointFormat(coords[1].to_string()))?;

        Ok(FreePoint { x, y })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "value": format!("{}, {}", self.x, self.y)
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        format!("{} = FreePoint({}, {})", name, self.x, self.y)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlidingPoint {
    pub x: i64,
    pub y: i64,
    pub constraining_object_name: String,
}

impl SlidingPoint {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let value = properties["value"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'value' field".to_string()))?;

        let coords: Vec<&str> = value.split(',').collect();
        if coords.len() != 2 {
            return Err(SceneError::InvalidPointFormat(value.to_string()));
        }

        let x = coords[0]
            .trim()
            .parse::<i64>()
            .map_err(|_| SceneError::InvalidPointFormat(coords[0].to_string()))?;
        let y = coords[1]
            .trim()
            .parse::<i64>()
            .map_err(|_| SceneError::InvalidPointFormat(coords[1].to_string()))?;

        let constraining_object_name = properties["constraining_object_name"]
            .as_str()
            .ok_or_else(|| {
                SceneError::InvalidProperties(
                    "Missing 'constraining_object_name' field".to_string(),
                )
            })?
            .to_string();

        Ok(SlidingPoint {
            x,
            y,
            constraining_object_name,
        })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "value": format!("{}, {}", self.x, self.y),
            "constraining_object_name": self.constraining_object_name,
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        format!(
            "{} = FreePoint({}, {})\n{}.contains({})",
            name, self.x, self.y, self.constraining_object_name, name
        )
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        vec![self.constraining_object_name.clone()]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntersectionPoint {
    pub object_name_1: String,
    pub object_name_2: String,
}

impl IntersectionPoint {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let object_name_1 = properties["object_name_1"]
            .as_str()
            .ok_or_else(|| {
                SceneError::InvalidProperties("Missing 'object_name_1' field".to_string())
            })?
            .to_string();
        let object_name_2 = properties["object_name_2"]
            .as_str()
            .ok_or_else(|| {
                SceneError::InvalidProperties("Missing 'object_name_2' field".to_string())
            })?
            .to_string();

        Ok(IntersectionPoint {
            object_name_1,
            object_name_2,
        })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "object_name_1": self.object_name_1,
            "object_name_2": self.object_name_2
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        let new_value = "Value(next_var(), initial=Value(next_var()))";
        let line1 = format!("{} = Point({}, {})", name, new_value, new_value);
        let line2 = format!("{}.contains({})", self.object_name_1, name);
        let line3 = format!("{}.contains({})", self.object_name_2, name);
        format!("{}\n{}\n{}", line1, line2, line3)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        vec![self.object_name_1.clone(), self.object_name_2.clone()]
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct LineAB {
    pub point1: String,
    pub point2: String,
}

impl LineAB {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let point1 = properties["point1"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'point1' field".to_string()))?
            .to_string();
        let point2 = properties["point2"]
            .as_str()
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'point2' field".to_string()))?
            .to_string();

        Ok(LineAB { point1, point2 })
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

        format!("{} = LineAB({}, {})", name, point1, point2)
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

#[derive(Debug, Clone, PartialEq)]
pub struct Locus {
    pub point: String,
}

impl Locus {
    pub fn new(properties: Value) -> Result<Self, SceneError> {
        let point = properties
            .get("point")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SceneError::InvalidProperties("Missing 'point' property".to_string()))?;

        Ok(Self {
            point: point.to_string(),
        })
    }

    pub fn get_properties(&self) -> Value {
        json!({
            "point": self.point
        })
    }

    pub fn to_python(&self, name: &str) -> String {
        format!("plot(\"{}\", {})", name, self.point)
    }

    pub fn get_dependencies(&self) -> Vec<String> {
        vec![self.point.clone()]
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ObjectType {
    FixedPoint,
    FreePoint,
    SlidingPoint,
    Midpoint,
    IntersectionPoint,
    LineAB,
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
            "SlidingPoint" => Ok(ObjectType::SlidingPoint),
            "IntersectionPoint" => Ok(ObjectType::IntersectionPoint),
            "Midpoint" => Ok(ObjectType::Midpoint),
            "LineAB" => Ok(ObjectType::LineAB),
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
            ObjectType::SlidingPoint => "SlidingPoint".to_string(),
            ObjectType::IntersectionPoint => "IntersectionPoint".to_string(),
            ObjectType::Midpoint => "Midpoint".to_string(),
            ObjectType::LineAB => "LineAB".to_string(),
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
    fn test_fixed_point() {
        let props = json!({
            "value": "10, 20"
        });
        let point = FixedPoint::new(props).unwrap();
        assert_eq!(point.x, 10);
        assert_eq!(point.y, 20);
        assert_eq!(
            point.get_properties(),
            json!({
                "value": "10, 20"
            })
        );
    }

    #[test]
    fn test_free_point() {
        let props = json!({
            "value": "30, 40"
        });
        let point = FreePoint::new(props).unwrap();
        assert_eq!(point.x, 30);
        assert_eq!(point.y, 40);
        assert_eq!(
            point.get_properties(),
            json!({
                "value": "30, 40"
            })
        );
    }

    #[test]
    fn test_sliding_point() {
        let props = json!({
            "value": "5,0",
            "constraining_object_name": "L1",
        });
        let point = SlidingPoint::new(props).unwrap();
        assert_eq!(point.x, 5);
        assert_eq!(point.y, 0);
        assert_eq!(
            point.get_properties(),
            json!({
                "value": "5, 0",
                "constraining_object_name": "L1",
            })
        );
    }

    #[test]
    fn test_intersection_point() {
        let props = json!({
            "object_name_1": "L1",
            "object_name_2": "C1"
        });
        let point = IntersectionPoint::new(props).unwrap();
        assert_eq!(point.object_name_1, "L1");
        assert_eq!(point.object_name_2, "C1");
        assert_eq!(
            point.get_properties(),
            json!({
                "object_name_1": "L1",
                "object_name_2": "C1"
            })
        );
    }

    #[test]
    fn test_line_ab() {
        let props = json!({
            "point1": "P1",
            "point2": "P2"
        });
        let line = LineAB::new(props).unwrap();
        assert_eq!(line.point1, "P1");
        assert_eq!(line.point2, "P2");
        assert_eq!(
            line.get_properties(),
            json!({
                "point1": "P1",
                "point2": "P2"
            })
        );
    }

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
    }
}
