use crate::fint::FInt;
use crate::poly::{Poly, PolyOperations};
use crate::x_poly::{XPoly, XYPoly};

pub trait PolyConversion {
    fn as_x_poly(&self, v: u8) -> Result<XPoly, String>;
    fn as_xy_poly(&self, xv: u8, yv: u8) -> Result<XYPoly, String>;
    fn from_poly_expression(s: &str) -> Result<Poly, String>;
    fn as_formatted_equation(&self, x_var: u8, y_var: u8) -> String;
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
        if xv == yv {
            return Err("x and y variables must be different".to_string());
        }

        let result = if xv < yv {
            // Original case: xv < yv
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
        } else {
            // New case: xv > yv, so we call as_xy_poly(yv, xv) and flip the result
            self.as_xy_poly(yv, xv)
        }?;

        // If xv > yv, flip the result to get the correct variable ordering
        if xv > yv {
            Ok(result.flip())
        } else {
            Ok(result)
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

    fn as_formatted_equation(&self, x_var: u8, y_var: u8) -> String {
        let terms = self.to_terms();

        // Separate constant term from variable terms
        let mut constant_term: Option<i64> = None;
        let mut variable_terms = Vec::new();

        for term in terms {
            let mut x_degree = 0;
            let mut y_degree = 0;

            for (var_idx, degree) in &term.vars {
                if *var_idx == x_var {
                    x_degree = *degree;
                } else if *var_idx == y_var {
                    y_degree = *degree;
                } else {
                    panic!(
                        "Polynomial contains variable {} which is not x or y",
                        Poly::var_to_string(*var_idx)
                    );
                }
            }

            if x_degree == 0 && y_degree == 0 {
                // This is a constant term
                constant_term = Some(term.constant);
            } else {
                // This is a variable term
                variable_terms.push((term.constant, x_degree, y_degree));
            }
        }

        // Sort variable terms by total degree (descending), then by x degree (descending)
        variable_terms.sort_by(|(_, x1, y1), (_, x2, y2)| {
            let total1 = x1 + y1;
            let total2 = x2 + y2;
            total2.cmp(&total1).then_with(|| x2.cmp(x1))
        });

        // Build the polynomial part
        let mut poly_parts = Vec::new();
        for (coeff, x_deg, y_deg) in variable_terms {
            let mut monomial = String::new();

            // Add coefficient if not 1
            if coeff.abs() != 1 {
                monomial.push_str(&coeff.abs().to_string());
            }

            // Add x part
            if x_deg > 0 {
                monomial.push('x');
                if x_deg > 1 {
                    monomial.push_str(&Self::degree_to_superscript(x_deg));
                }
            }

            // Add y part
            if y_deg > 0 {
                monomial.push('y');
                if y_deg > 1 {
                    monomial.push_str(&Self::degree_to_superscript(y_deg));
                }
            }

            // Handle coefficient of 1 with no variables
            if x_deg == 0 && y_deg == 0 {
                monomial = coeff.abs().to_string();
            }

            poly_parts.push((coeff, monomial));
        }

        // Build the final equation
        let mut equation = String::new();

        match constant_term {
            Some(c) if c > 0 => {
                // Format: (-p) = c
                if !poly_parts.is_empty() {
                    let poly_parts: Vec<(i64, String)> = poly_parts
                        .iter()
                        .map(|(coeff, monomial)| (-coeff, monomial.clone()))
                        .collect();
                    equation.push_str(&Self::format_polynomial_parts(&poly_parts));
                    equation.push_str(" = ");
                    equation.push_str(&c.to_string());
                } else {
                    equation.push_str("0 = ");
                    equation.push_str(&c.to_string());
                }
            }
            Some(c) if c < 0 => {
                // Format: p = -c
                if !poly_parts.is_empty() {
                    equation.push_str(&Self::format_polynomial_parts(&poly_parts));
                    equation.push_str(" = ");
                    equation.push_str(&(-c).to_string());
                } else {
                    equation.push_str("0 = ");
                    equation.push_str(&(-c).to_string());
                }
            }
            Some(_) | None => {
                // Format: p = 0
                if !poly_parts.is_empty() {
                    equation.push_str(&Self::format_polynomial_parts(&poly_parts));
                    equation.push_str(" = 0");
                } else {
                    equation.push_str("0 = 0");
                }
            }
        }

        equation
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

impl Poly {
    /// Convert a degree to Unicode superscript
    fn degree_to_superscript(degree: u32) -> String {
        let superscript_chars = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
        let mut result = String::new();
        let mut n = degree;

        if n == 0 {
            return "⁰".to_string();
        }

        while n > 0 {
            result.insert(0, superscript_chars[(n % 10) as usize]);
            n /= 10;
        }

        result
    }

    /// Format polynomial parts with proper signs
    fn format_polynomial_parts(parts: &[(i64, String)]) -> String {
        if parts.is_empty() {
            return "0".to_string();
        }

        let mut result = String::new();
        let mut first = true;

        for (coeff, monomial) in parts {
            if !first {
                if *coeff > 0 {
                    result.push_str(" + ");
                } else {
                    result.push_str(" - ");
                }
            } else if *coeff < 0 {
                result.push('-');
            }
            first = false;

            result.push_str(monomial);
        }

        result
    }
}

impl std::fmt::Display for Poly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let terms = self.to_terms();
        if terms.is_empty() {
            return write!(f, "0");
        }

        // Check if full formatting is requested
        let use_full = f.alternate(); // This enables {:#} formatting

        let last_term = terms.last().unwrap().clone();
        let mut first = true;
        let mut lexemes_written = 0;
        for (i, term) in terms.iter().enumerate() {
            let mut last_iteration = false;
            let term = if !use_full && lexemes_written > 100 {
                write!(f, "... ({} terms omitted)", terms.len() - i - 1)?;
                last_iteration = true;
                last_term.clone()
            } else {
                term.clone()
            };

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
                lexemes_written += 1;
            }
            let mut first = true;

            // Print variables
            for (var_idx, degree) in term.vars {
                if abs_constant != 1 || !first {
                    write!(f, "*")?;
                }
                write!(f, "{}", Self::var_to_string(var_idx))?;
                lexemes_written += 1;
                if degree > 1 {
                    write!(f, "^{}", degree)?;
                    lexemes_written += 1;
                }
                first = false;
            }
            if last_iteration {
                break;
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
        assert_eq!(result.0[0].0.len(), 2); // 4 + 3b
        assert_eq!(result.0[0].0[0].midpoint(), 4.0);
        assert_eq!(result.0[0].0[1].midpoint(), 3.0);
        assert_eq!(result.0[1].0.len(), 2); // 2 + b
        assert_eq!(result.0[1].0[0].midpoint(), 2.0);
        assert_eq!(result.0[1].0[1].midpoint(), 1.0);

        // Test reversed variables
        let poly = Poly::new("a*b + 2*a + 3*b + 4").unwrap();
        let result = poly.as_xy_poly(1, 0).unwrap();
        assert_eq!(result.0.len(), 2);
        assert_eq!(result.0[0].0.len(), 2); // 4 + 2a
        assert_eq!(result.0[0].0[0].midpoint(), 4.0);
        assert_eq!(result.0[0].0[1].midpoint(), 2.0);
        assert_eq!(result.0[1].0.len(), 2); // 3 + a
        assert_eq!(result.0[1].0[0].midpoint(), 3.0);
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

    #[test]
    fn test_as_formatted_equation() {
        // Test simple linear equation: x + y = 0
        let poly = Poly::new("a + b").unwrap();
        let result = poly.as_formatted_equation(0, 1);
        assert_eq!(result, "x + y = 0");

        // Test equation with constant: x² + y² = 25
        let poly = Poly::new("a^2 + b^2 - 25").unwrap();
        let result = poly.as_formatted_equation(0, 1);
        assert_eq!(result, "x² + y² = 25");

        // Test equation with negative constant: x² + y² = -16
        let poly = Poly::new("a^2 + b^2 + 16").unwrap();
        let result = poly.as_formatted_equation(0, 1);
        assert_eq!(result, "-x² - y² = 16");

        // Test equation with mixed terms: 2x²y + 3xy² = 0
        let poly = Poly::new("2*a^2*b + 3*a*b^2").unwrap();
        let result = poly.as_formatted_equation(0, 1);
        assert_eq!(result, "2x²y + 3xy² = 0");

        // Test equation with high degrees: x¹⁰y²⁰ = 0
        let poly = Poly::new("a^10*b^20").unwrap();
        let result = poly.as_formatted_equation(0, 1);
        assert_eq!(result, "x¹⁰y²⁰ = 0");

        // Test equation with negative coefficients: -x² + y² = 0
        let poly = Poly::new("-a^2 + b^2").unwrap();
        let result = poly.as_formatted_equation(0, 1);
        assert_eq!(result, "-x² + y² = 0");

        // Test equation with mixed signs: x² - y² = 1
        let poly = Poly::new("a^2 - b^2 - 1").unwrap();
        let result = poly.as_formatted_equation(0, 1);
        assert_eq!(result, "x² - y² = 1");

        // Test constant equation: 5 = 0
        let poly = Poly::new("5").unwrap();
        let result = poly.as_formatted_equation(0, 1);
        assert_eq!(result, "0 = 5");

        // Test zero equation: 0 = 0
        let poly = Poly::new("0").unwrap();
        let result = poly.as_formatted_equation(0, 1);
        assert_eq!(result, "0 = 0");

        // Test equation with only x terms: x³ = 0
        let poly = Poly::new("a^3").unwrap();
        let result = poly.as_formatted_equation(0, 1);
        assert_eq!(result, "x³ = 0");

        // Test equation with only y terms: y⁴ = 0
        let poly = Poly::new("b^4").unwrap();
        let result = poly.as_formatted_equation(0, 1);
        assert_eq!(result, "y⁴ = 0");
    }

    #[test]
    #[should_panic(expected = "variable z which is not x or y")]
    fn test_as_formatted_equation_error_cases() {
        // Test with third variable (should panic)
        let poly = Poly::new("x + y + z").unwrap();
        let _result = poly.as_formatted_equation(0, 1);
    }

    #[test]
    fn test_degree_to_superscript() {
        assert_eq!(Poly::degree_to_superscript(0), "⁰");
        assert_eq!(Poly::degree_to_superscript(1), "¹");
        assert_eq!(Poly::degree_to_superscript(2), "²");
        assert_eq!(Poly::degree_to_superscript(3), "³");
        assert_eq!(Poly::degree_to_superscript(10), "¹⁰");
        assert_eq!(Poly::degree_to_superscript(25), "²⁵");
        assert_eq!(Poly::degree_to_superscript(100), "¹⁰⁰");
    }
}
