use crate::fint::FInt;
use crate::poly::{Poly, PolyOperations};
use crate::x_poly::{XPoly, XYPoly};

pub trait PolyConversion {
    fn as_x_poly(&self, v: u8) -> Result<XPoly, String>;
    fn as_xy_poly(&self, xv: u8, yv: u8) -> Result<XYPoly, String>;
    fn from_poly_expression(s: &str) -> Result<Poly, String>;
}

impl PolyConversion for Poly {
    fn as_x_poly(&self, v: u8) -> Result<XPoly, String> {
        match self {
            Poly::Constant(n) => Ok(XPoly::new(vec![FInt::new(*n as f64)])),
            Poly::Nested(v1, polys) => {
                if *v1 != v {
                    return Err(format!("Variable {} not found in polynomial", v));
                }
                let mut coefficients = Vec::with_capacity(polys.len());
                for poly in polys {
                    match &**poly {
                        Poly::Constant(n) => coefficients.push(FInt::new(*n as f64)),
                        _ => return Err("Non-constant coefficients not supported".to_string()),
                    }
                }
                Ok(XPoly::new(coefficients))
            }
        }
    }

    fn as_xy_poly(&self, xv: u8, yv: u8) -> Result<XYPoly, String> {
        if xv >= yv {
            return Err("x variable must be less than y variable".to_string());
        }

        match self {
            Poly::Constant(_) => Ok(XYPoly::new(vec![self.as_x_poly(yv)?])),
            Poly::Nested(v1, polys) => {
                if *v1 == yv {
                    Ok(XYPoly::new(vec![self.as_x_poly(yv)?]))
                } else if *v1 == xv {
                    let mut coefficients = Vec::with_capacity(polys.len());
                    for poly in polys {
                        coefficients.push(poly.as_x_poly(yv)?);
                    }
                    Ok(XYPoly::new(coefficients))
                } else {
                    Err(format!(
                        "Polynomial must be in terms of variables {} and {}",
                        xv, yv
                    ))
                }
            }
        }
    }

    fn from_poly_expression(s: &str) -> Result<Poly, String> {
        let s = s.trim();
        if s.is_empty() {
            return Ok(Poly::Constant(0));
        }

        let (result, _) = Self::parse_expression(s, 0)?;
        Ok(result)
    }
}

impl std::fmt::Debug for Poly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Poly::Constant(n) => write!(f, "Constant({})", n),
            Poly::Nested(v, polys) => {
                write!(f, "Nested({}, [", v)?;
                for (i, p) in polys.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", p)?;
                }
                write!(f, "])")
            }
        }
    }
}

impl std::fmt::Display for Poly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let terms = self.to_terms();
        if terms.is_empty() {
            return write!(f, "0");
        }
        let mut first = true;
        for term in terms {
            if !first {
                if term.constant < 0 {
                    write!(f, " - ")?;
                } else {
                    write!(f, " + ")?;
                }
            } else if term.constant < 0 {
                write!(f, "-")?;
            }
            first = false;

            // Print constant (absolute value)
            let abs_constant = term.constant.abs();
            if abs_constant != 1 || term.vars.is_empty() {
                write!(f, "{}", abs_constant)?;
            }
            let mut first = true;

            // Print variables
            for (var_idx, degree) in term.vars {
                if abs_constant != 1 || !first {
                    write!(f, "*")?;
                }
                write!(f, "{}", Self::var_to_string(var_idx))?;
                if degree > 1 {
                    write!(f, "^{}", degree)?;
                }
                first = false;
            }
        }
        Ok(())
    }
}

impl Poly {
    fn parse_expression(s: &str, mut pos: usize) -> Result<(Poly, usize), String> {
        let mut result = Poly::Constant(0);
        let mut current_sign = 1;

        while pos < s.len() {
            let ch = s.chars().nth(pos).unwrap();

            if ch == ')' {
                // End of parenthesized expression, return
                return Ok((result, pos + 1));
            } else if ch == '(' {
                // Parse parenthesized expression
                pos += 1;
                let (sub_poly, new_pos) = Self::parse_expression(s, pos)?;
                pos = new_pos;

                // Check if the next symbol is "*"
                let product = if pos < s.len() && s.chars().nth(pos).unwrap() == '*' {
                    pos += 1; // Skip the "*" symbol
                    let (monomial, final_pos) = Self::extract_monomial(s, pos)?;
                    pos = final_pos;
                    sub_poly.multiply(&monomial)
                } else {
                    sub_poly
                };

                result.add_poly_scaled(&product, current_sign);
            } else if ch == '+' || ch == '-' {
                // Handle sign
                current_sign = if ch == '+' { 1 } else { -1 };
                pos += 1;
            } else if ch.is_whitespace() {
                // Skip whitespace
                pos += 1;
            } else {
                // Collect monomial
                let (monomial, new_pos) = Self::extract_monomial(s, pos)?;
                pos = new_pos;
                result.add_poly_scaled(&monomial, current_sign);
            }
        }

        Ok((result, pos))
    }

    fn extract_monomial(s: &str, mut pos: usize) -> Result<(Poly, usize), String> {
        // Skip leading whitespace
        while pos < s.len() && s.chars().nth(pos).unwrap().is_whitespace() {
            pos += 1;
        }

        if pos >= s.len() {
            return Ok((Poly::Constant(1), pos));
        }

        let start_pos = pos;

        // Keep increasing the index until +, -, ), or the end of the line is encountered
        while pos < s.len() {
            let ch = s.chars().nth(pos).unwrap();
            if ch == '+' || ch == '-' || ch == ')' {
                break;
            }
            pos += 1;
        }

        // Extract the monomial string
        let monomial_str = &s[start_pos..pos].trim();

        // Convert the whole part to Poly via Poly::new()
        if monomial_str.is_empty() {
            Ok((Poly::Constant(1), pos))
        } else {
            let poly = Poly::new(monomial_str)
                .map_err(|e| format!("Invalid monomial '{}': {:?}", monomial_str, e))?;
            Ok((poly, pos))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::relative_eq;

    #[test]
    fn test_as_x_poly() {
        // Test constant polynomial
        let poly = Poly::new("5").unwrap();
        let result = poly.as_x_poly(0).unwrap();
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].midpoint(), 5.0);

        // Test linear polynomial
        let poly = Poly::new("2*a + 3").unwrap();
        let result = poly.as_x_poly(0).unwrap();
        assert_eq!(result.0.len(), 2);
        assert_eq!(result.0[0].midpoint(), 3.0);
        assert_eq!(result.0[1].midpoint(), 2.0);

        // Test quadratic polynomial
        let poly = Poly::new("a^2 + 2*a + 1").unwrap();
        let result = poly.as_x_poly(0).unwrap();
        assert_eq!(result.0.len(), 3);
        assert_eq!(result.0[0].midpoint(), 1.0);
        assert_eq!(result.0[1].midpoint(), 2.0);
        assert_eq!(result.0[2].midpoint(), 1.0);

        // Test error cases
        let poly = Poly::new("a*b").unwrap();
        assert!(poly.as_x_poly(0).is_err());
    }

    #[test]
    fn test_as_xy_poly() {
        // Test constant polynomial
        let poly = Poly::new("5").unwrap();
        let result = poly.as_xy_poly(0, 1).unwrap();
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].0.len(), 1);
        assert_eq!(result.0[0].0[0].midpoint(), 5.0);

        // Test polynomial in x only
        let poly = Poly::new("2*a + 3").unwrap();
        let result = poly.as_xy_poly(0, 1).unwrap();
        assert_eq!(result.0.len(), 2);
        assert_eq!(result.0[0].0.len(), 1);
        assert_eq!(result.0[1].0.len(), 1);
        assert_eq!(result.0[0].0[0].midpoint(), 3.0);
        assert_eq!(result.0[1].0[0].midpoint(), 2.0);

        // Test polynomial in y only
        let poly = Poly::new("2*b + 3").unwrap();
        let result = poly.as_xy_poly(0, 1).unwrap();
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].0.len(), 2);
        assert_eq!(result.0[0].0[0].midpoint(), 3.0);
        assert_eq!(result.0[0].0[1].midpoint(), 2.0);

        // Test polynomial in both x and y
        let poly = Poly::new("a*b + 2*a + 3*b + 4").unwrap();
        let result = poly.as_xy_poly(0, 1).unwrap();
        assert_eq!(result.0.len(), 2);
        assert_eq!(result.0[0].0.len(), 2);
        assert_eq!(result.0[1].0.len(), 2);
        assert_eq!(result.0[0].0[0].midpoint(), 4.0);
        assert_eq!(result.0[0].0[1].midpoint(), 3.0);
        assert_eq!(result.0[1].0[0].midpoint(), 2.0);
        assert_eq!(result.0[1].0[1].midpoint(), 1.0);

        // Test error cases
        let poly = Poly::new("a*b*c").unwrap();
        assert!(poly.as_xy_poly(0, 1).is_err());
        assert!(poly.as_xy_poly(1, 0).is_err());
    }

    #[test]
    fn test_from_poly_expression() {
        // Test case 1: Simple expression "(b + 1)*a - b"
        let result = Poly::from_poly_expression("(b + 1)*a - b").unwrap();
        assert_eq!(format!("{}", result), "-b + a + b*a");

        // Test case 2: Simple constant
        let result = Poly::from_poly_expression("5").unwrap();
        assert_eq!(format!("{}", result), "5");

        // Test case 3: Simple variable
        let result = Poly::from_poly_expression("a").unwrap();
        assert_eq!(format!("{}", result), "a");

        // Test case 4: Simple addition
        let result = Poly::from_poly_expression("a + b").unwrap();
        assert_eq!(format!("{}", result), "b + a");

        // Test case 5: Simple subtraction
        let result = Poly::from_poly_expression("a - b").unwrap();
        assert_eq!(format!("{}", result), "-b + a");

        // Test case 6: Parenthesized expression
        let result = Poly::from_poly_expression("(a + b)").unwrap();
        assert_eq!(format!("{}", result), "b + a");

        // Test case 7: Parenthesized expression with multiplication
        let result = Poly::from_poly_expression("(a + b)*c").unwrap();
        assert_eq!(format!("{}", result), "c*b + c*a");

        // Test case 8: Complex expression with multiple parentheses
        let result = Poly::from_poly_expression("(a + b)*c + d").unwrap();
        assert_eq!(format!("{}", result), "d + c*b + c*a");

        // Test case 9: Expression with coefficients
        let result = Poly::from_poly_expression("2*a + 3*b").unwrap();
        assert_eq!(format!("{}", result), "3*b + 2*a");

        // Test case 10: Expression with exponents
        let result = Poly::from_poly_expression("(-a^2 + b)*c^2 + b*c + b^3").unwrap();
        assert_eq!(format!("{}", result), "c*b + c^2*b + b^3 - c^2*a^2");

        // Test case 11: Expression with mixed terms
        let result = Poly::from_poly_expression("2*a^2 + 3*b + 1").unwrap();
        assert_eq!(format!("{}", result), "1 + 3*b + 2*a^2");

        // Test case 12: Expression with negative coefficients
        let result = Poly::from_poly_expression("-2*a + 3*b").unwrap();
        assert_eq!(format!("{}", result), "3*b - 2*a");

        // Test case 13: Expression with whitespace
        let result = Poly::from_poly_expression("  a  +  b  ").unwrap();
        assert_eq!(format!("{}", result), "b + a");

        // Test case 14: Expression with explicit multiplication
        let result = Poly::from_poly_expression("a * b + c").unwrap();
        assert_eq!(format!("{}", result), "c + b*a");

        // Test case 15: Empty expression
        let result = Poly::from_poly_expression("").unwrap();
        assert_eq!(format!("{}", result), "0");
    }

    #[test]
    fn test_from_poly_expression_edge_cases() {
        // Test case 1: Single negative term
        let result = Poly::from_poly_expression("-a").unwrap();
        assert_eq!(format!("{}", result), "-a");

        // Test case 2: Single positive term with plus
        let result = Poly::from_poly_expression("+a").unwrap();
        assert_eq!(format!("{}", result), "a");

        // Test case 3: Multiple parentheses
        let result = Poly::from_poly_expression("((a + b))").unwrap();
        assert_eq!(format!("{}", result), "b + a");

        // Test case 4: Nested parentheses
        let result = Poly::from_poly_expression("(a + (b + c))").unwrap();
        assert_eq!(format!("{}", result), "c + b + a");

        // Test case 5: Expression with multiple variables and exponents
        let result = Poly::from_poly_expression("a^2*b^3 + c*d^2").unwrap();
        assert_eq!(format!("{}", result), "d^2*c + b^3*a^2");

        // Test case 6: Expression with coefficient 1
        let result = Poly::from_poly_expression("1*a + 1*b").unwrap();
        assert_eq!(format!("{}", result), "b + a");

        // Test case 7: Expression with zero coefficient
        let result = Poly::from_poly_expression("0*a + b").unwrap();
        assert_eq!(format!("{}", result), "b");
    }

    #[test]
    fn test_from_poly_expression_complex_case() {
        let input = "((3*y1^2 - 3*y1)*b*a + (3*y1^3 - 3*y1 - 2))*x^2 + ((3*y1^2 - 3*y1 - 2)*b*a + (d + (3*y1^3 - 2*y1 - 1)))*x + ((d + (y1 + 1))*b*a + (d + (y1 + 1)))";
        let result = Poly::from_poly_expression(input).unwrap();
        let poly1 = Poly::new("1 + a*b + x").unwrap();
        let poly2 = Poly::new("1 + y1 - 2*x + d").unwrap();
        let product1 = poly1.multiply(&poly2);
        let poly3 = Poly::new("3*x*y1^2 - 3*x*y1").unwrap();
        let poly4 = Poly::new("1 + a*b + y1").unwrap();
        let poly5 = Poly::new("1 + x").unwrap();
        let product2 = poly3.multiply(&poly4).multiply(&poly5);
        let mut sum = product1;
        sum.add_poly_scaled(&product2, 1);
        assert_eq!(format!("{}", result), format!("{}", sum));
    }
}
