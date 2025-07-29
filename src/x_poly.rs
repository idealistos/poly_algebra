use crate::fint::{FInt, ZERO_FINT};
use rand::Rng;
use std::fmt;
use std::ops::{Add, Index, IndexMut, Mul, Sub};

const NEWTON_MAX_ATTEMPTS: usize = 20;
const NEWTON_MAX_ITERATIONS: usize = 100;

#[derive(Clone)]
pub struct XPoly(pub Vec<FInt>);

impl XPoly {
    pub fn new(coeffs: Vec<FInt>) -> Self {
        let mut p = XPoly(coeffs);
        p.cleanup();
        p
    }

    pub fn get_degree(&self) -> usize {
        self.0.len().max(1) - 1
    }

    pub fn cleanup(&mut self) {
        while let Some(last) = self.0.last() {
            if last.precise() && *last == FInt::new(0.0) {
                self.0.pop();
            } else {
                break;
            }
        }
    }

    pub fn divide(&self, divisor: &XPoly) -> (XPoly, XPoly) {
        if divisor.0.is_empty() {
            panic!("Division by zero polynomial");
        }

        let mut quotient = XPoly::new(vec![]);
        let mut remainder = self.clone();

        while !remainder.0.is_empty() && remainder.get_degree() >= divisor.get_degree() {
            let degree_diff = remainder.get_degree() - divisor.get_degree();
            let leading_coeff =
                remainder[remainder.get_degree()] * divisor[divisor.get_degree()].inverse();

            let mut term = vec![FInt::new(0.0); degree_diff];
            term.push(leading_coeff);
            let term_poly = XPoly::new(term);

            quotient = &quotient + &term_poly;
            remainder = &remainder - &(&term_poly * divisor);
        }

        (quotient, remainder)
    }

    pub fn evaluate(&self, x: FInt) -> FInt {
        let mut result = FInt::new(0.0);
        let mut power = FInt::new(1.0);
        for &coef in &self.0 {
            result = result + coef * power;
            power = power * x;
        }
        result
    }

    // self = result.0 * (x - a) + result.1
    pub fn divide_by_monomial(&self, a: FInt) -> (XPoly, FInt) {
        if self.0.is_empty() {
            return (XPoly::new(vec![]), FInt::new(0.0));
        }
        let degree = self.get_degree();
        let mut quotient = vec![FInt::new(0.0); degree];
        let mut remainder = self[degree];

        for i in (0..degree).rev() {
            quotient[i] = remainder;
            remainder = self[i] + remainder * a;
        }

        (XPoly::new(quotient), remainder)
    }

    pub fn to_string(&self, var: &str) -> String {
        if self.0.is_empty() {
            return "0".to_string();
        }

        let mut result = String::new();
        let mut first = true;

        for (i, &coef) in self.0.iter().enumerate() {
            if coef == FInt::new(0.0) {
                continue;
            }

            if !first {
                if coef.midpoint() < 0.0 {
                    result.push_str(" - ");
                } else {
                    result.push_str(" + ");
                }
            } else if coef.midpoint() < 0.0 {
                result.push('-');
            }
            first = false;

            // Print coefficient (absolute value)
            let abs_coef = if coef.midpoint() < 0.0 {
                FInt::new(0.0) - coef
            } else {
                coef
            };
            if abs_coef != FInt::new(1.0) || i == 0 {
                result.push_str(&abs_coef.to_string());
            }

            // Print variable and exponent
            if i > 0 {
                result.push_str(var);
                if i > 1 {
                    result.push_str(&format!("^{}", i));
                }
            }
        }

        if result.is_empty() {
            "0".to_string()
        } else {
            result
        }
    }

    pub fn gcd(&self, other: &XPoly) -> XPoly {
        if self.0.is_empty() || other.0.is_empty() {
            panic!("Cannot compute GCD of empty polynomials");
        }

        let mut a = self.clone();
        let mut b = other.clone();

        while !b.0.is_empty() {
            let (_, remainder) = a.divide(&b);
            a = b;
            b = remainder;
        }

        // Make the leading coefficient positive
        if !a.0.is_empty() && a[a.get_degree()].midpoint() < 0.0 {
            a = &XPoly::new(vec![FInt::new(-1.0)]) * &a;
        }

        a
    }

    pub fn get_derivative(&self) -> XPoly {
        if self.0.len() <= 1 {
            return XPoly::new(vec![]);
        }

        let mut result = Vec::with_capacity(self.0.len() - 1);
        for (i, &coeff) in self.0.iter().skip(1).enumerate() {
            result.push(FInt::new((i + 1) as f64) * coeff);
        }
        XPoly::new(result)
    }

    fn sturm_sequence(&self) -> Vec<XPoly> {
        let mut sequence = vec![self.clone()];
        if self.0.is_empty() {
            return sequence;
        }

        sequence.push(self.get_derivative());
        while sequence.last().unwrap().get_degree() > 0 {
            let (_, remainder) = sequence[sequence.len() - 2].divide(sequence.last().unwrap());
            sequence.push(&XPoly::new(vec![FInt::new(-1.0)]) * &remainder);
        }
        sequence
    }

    fn sign_changes_at(&self, x: f64) -> i32 {
        let mut changes = 0;
        let mut prev_sign = None;

        for poly in self.sturm_sequence() {
            let value = poly.evaluate(FInt::new(x));
            let sign = if value == FInt::new(0.0) {
                None
            } else if value.midpoint() > 0.0 {
                Some(1)
            } else {
                Some(-1)
            };

            if let Some(s) = sign {
                if let Some(p) = prev_sign {
                    if s != p {
                        changes += 1;
                    }
                }
                prev_sign = Some(s);
            }
        }
        changes
    }

    fn count_roots_between(&self, low: f64, high: f64) -> i32 {
        self.sign_changes_at(low) - self.sign_changes_at(high)
    }

    fn find_root_newton(&self, low: f64, high: f64) -> Option<FInt> {
        let mut rng = rand::rng();
        let derivative = self.get_derivative();

        for _ in 0..NEWTON_MAX_ATTEMPTS {
            let mut x = FInt::new(rng.random_range(low..high));
            let mut min_abs_value = f64::INFINITY;
            let mut iterations_without_improvement = 0;

            for _ in 0..NEWTON_MAX_ITERATIONS {
                let p = self.evaluate(x);
                if p == FInt::new(0.0) {
                    return Some(x);
                }
                let p_prime = derivative.evaluate(x);

                if p_prime == FInt::new(0.0) {
                    break;
                }

                let next_x = x - p / p_prime;
                if next_x.midpoint() < low || next_x.midpoint() > high {
                    break;
                }

                let abs_value = p.abs_bound();
                if abs_value < min_abs_value {
                    min_abs_value = abs_value;
                    iterations_without_improvement = 0;
                } else {
                    iterations_without_improvement += 1;
                }

                if iterations_without_improvement >= 5 {
                    break;
                }

                x = FInt::new(next_x.midpoint());
            }
        }
        None
    }

    fn find_root_binary_search(&self, low: f64, high: f64) -> Option<FInt> {
        let mut a = FInt::new(low);
        let mut b = FInt::new(high);
        let p_a = self.evaluate(a);
        let p_b = self.evaluate(b);

        if p_a.midpoint() * p_b.midpoint() >= 0.0 {
            return None;
        }

        for _ in 0..100 {
            let mid = FInt::new((a.midpoint() + b.midpoint()) / 2.0);
            let p_mid = self.evaluate(mid);

            if p_mid.midpoint() * p_a.midpoint() < 0.0 {
                b = mid;
            } else {
                a = mid;
            }

            if b.midpoint() - a.midpoint() < 1e-10 {
                return Some(mid);
            }
        }
        None
    }

    fn find_root(&self, low: f64, high: f64) -> Option<FInt> {
        self.find_root_newton(low, high)
            .or_else(|| self.find_root_binary_search(low, high))
    }

    pub fn get_roots(&self, low: f64, high: f64) -> Vec<FInt> {
        if self.0.is_empty() {
            return vec![];
        }

        // Get square-free polynomial
        let derivative = self.get_derivative();
        let gcd = self.gcd(&derivative);
        let (square_free, _) = self.divide(&gcd);

        let mut roots = Vec::new();
        let mut current_poly = square_free;

        loop {
            let root_count = current_poly.count_roots_between(low, high);
            if root_count <= 0 {
                break;
            }

            if let Some(root) = current_poly.find_root(low, high) {
                roots.push(root);
                let (quotient, _) = current_poly.divide_by_monomial(root);
                current_poly = quotient;
            } else {
                break;
            }
        }

        roots
    }
}

impl fmt::Display for XPoly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string("x"))
    }
}

impl fmt::Debug for XPoly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string("x"))
    }
}

impl Add for &XPoly {
    type Output = XPoly;

    fn add(self, other: &XPoly) -> XPoly {
        let mut result = self.clone();
        for (i, &coeff) in other.0.iter().enumerate() {
            if i < result.0.len() {
                result.0[i] = result[i] + coeff;
            } else {
                result.0.push(coeff);
            }
        }
        result.cleanup();
        result
    }
}

impl Sub for &XPoly {
    type Output = XPoly;

    fn sub(self, other: &XPoly) -> XPoly {
        let mut result = self.clone();
        for (i, &coeff) in other.0.iter().enumerate() {
            if i <= result.get_degree() {
                result[i] = result[i] - coeff;
            } else {
                result.0.push(FInt::new(0.0) - coeff);
            }
        }
        result.cleanup();
        result
    }
}

impl Mul for &XPoly {
    type Output = XPoly;

    fn mul(self, other: &XPoly) -> XPoly {
        let mut result = vec![FInt::new(0.0); self.get_degree() + other.get_degree() + 1];
        for (i, &a) in self.0.iter().enumerate() {
            for (j, &b) in other.0.iter().enumerate() {
                result[i + j] = result[i + j] + a * b;
            }
        }
        let mut poly = XPoly::new(result);
        poly.cleanup();
        poly
    }
}

impl Index<usize> for XPoly {
    type Output = FInt;
    fn index(&self, i: usize) -> &FInt {
        if i > 0 || !self.0.is_empty() {
            &self.0[i]
        } else {
            &ZERO_FINT
        }
    }
}

impl IndexMut<usize> for XPoly {
    fn index_mut(&mut self, i: usize) -> &mut FInt {
        if i > 0 || !self.0.is_empty() {
            &mut self.0[i]
        } else {
            panic!("Attempt to mutate xpoly[0] for an empty xpoly")
        }
    }
}

// Definition of XYPoly(polys): p(x, y) = Sum polys[i](y) * x^i
#[derive(Clone)]
pub struct XYPoly(pub Vec<XPoly>);

impl XYPoly {
    pub fn new(coefficients: Vec<XPoly>) -> Self {
        XYPoly(coefficients)
    }

    /// Flips the polynomial by swapping the roles of x and y variables.
    /// This transforms a polynomial f(x,y) into f(y,x).
    pub fn flip(&self) -> XYPoly {
        // For a polynomial f(x,y) = Σᵢ cᵢ(y) * xⁱ, flipping gives f(y,x) = Σᵢ cᵢ(x) * yⁱ
        // This means we need to transpose the coefficient matrix

        if self.0.is_empty() {
            return XYPoly::new(vec![]);
        }

        // Find the maximum degree of any coefficient polynomial
        let max_degree = self
            .0
            .iter()
            .map(|poly| poly.get_degree())
            .max()
            .unwrap_or(0);

        // Create the flipped polynomial with coefficients for each power of y
        let mut flipped_coeffs = Vec::new();

        for y_power in 0..=max_degree {
            let mut coeff_poly = Vec::new();

            // For each power of y, collect the coefficient of x^y_power from each original coefficient
            for original_coeff in &self.0 {
                if y_power <= original_coeff.get_degree() {
                    coeff_poly.push(original_coeff[y_power]);
                } else {
                    coeff_poly.push(FInt::new(0.0));
                }
            }

            flipped_coeffs.push(XPoly::new(coeff_poly));
        }

        XYPoly::new(flipped_coeffs)
    }

    pub fn evaluate(&self, x: FInt, y: FInt) -> FInt {
        let mut result = FInt::new(0.0);
        let mut x_power = FInt::new(1.0);

        for poly in &self.0 {
            result = result + poly.evaluate(y) * x_power;
            x_power = x_power * x;
        }

        result
    }

    pub fn likely_contains_zero_check_corners_and_center(
        &self,
        x_region: FInt,
        y_region: FInt,
    ) -> bool {
        // Check all four corners of the rectangle plus its center: if all of then are positive
        // (or all are negative), it isn't likely that there is any point in the region for which
        // the polynomial is zero.
        let corners = vec![
            (x_region.lower_bound(), y_region.lower_bound()),
            (x_region.lower_bound(), y_region.upper_bound()),
            (x_region.upper_bound(), y_region.lower_bound()),
            (x_region.upper_bound(), y_region.upper_bound()),
            (x_region.midpoint(), y_region.midpoint()),
        ];
        let mut all_positive = true;
        let mut all_negative = true;
        for (x, y) in corners {
            let value = self.evaluate(FInt::new(x), FInt::new(y));
            if !value.always_positive() {
                all_positive = false;
            }
            if !value.negate().always_positive() {
                all_negative = false;
            }
        }
        !(all_positive || all_negative)
    }

    fn compute_determinant(matrix: &mut [Vec<XPoly>]) -> XPoly {
        let n = matrix.len();
        let mut sign = 1;

        for k in 0..n {
            // Find pivot
            let mut pivot_row = k;
            while pivot_row < n && matrix[pivot_row][k].0.is_empty() {
                pivot_row += 1;
            }
            if pivot_row == n {
                return XPoly::new(vec![]); // Zero determinant
            }

            // Swap rows if needed
            if pivot_row != k {
                matrix.swap(k, pivot_row);
                sign = -sign;
            }

            // Eliminate column k
            for i in k + 1..n {
                for j in k + 1..n {
                    let temp = &(&matrix[i][j] * &matrix[k][k]) - &(&matrix[i][k] * &matrix[k][j]);
                    if k > 0 {
                        if matrix[k - 1][k - 1].get_degree() == 0 {
                            matrix[i][j] =
                                &temp * &XPoly::new(vec![FInt::new(1.0) / matrix[k - 1][k - 1][0]]);
                        } else {
                            let (quotient, _) = temp.divide(&matrix[k - 1][k - 1]);
                            matrix[i][j] = quotient;
                        }
                    } else {
                        matrix[i][j] = temp;
                    }
                }
            }
        }

        // Return the last element with proper sign
        if sign < 0 {
            &XPoly::new(vec![FInt::new(-1.0)]) * &matrix[n - 1][n - 1]
        } else {
            matrix[n - 1][n - 1].clone()
        }
    }

    pub fn resultant(&self, other: &XYPoly) -> XPoly {
        let d1 = self.0.len() - 1;
        let d2 = other.0.len() - 1;
        let n = d1 + d2;

        // Initialize Sylvester matrix with polynomial entries
        let mut matrix = vec![vec![XPoly::new(vec![]); n]; n];

        // Fill first d2 rows with coefficients of self, with proper offsets
        for i in 0..d2 {
            for j in 0..=d1 {
                matrix[i][i + j] = self.0[j].clone();
            }
        }

        // Fill last d1 rows with coefficients of other, with proper offsets
        for i in 0..d1 {
            for j in 0..=d2 {
                matrix[d2 + i][i + j] = other.0[j].clone();
            }
        }
        println!("matrix: {:?}", matrix);
        Self::compute_determinant(&mut matrix)
    }

    pub fn to_string(&self, var_x: &str, var_y: &str) -> String {
        if self.0.is_empty() {
            return "0".to_string();
        }

        let mut result = String::new();
        let mut first = true;

        for (i, poly) in self.0.iter().enumerate() {
            if poly.0.iter().all(|&coef| coef == FInt::new(0.0)) {
                continue;
            }

            if !first {
                result.push_str(" + ");
            }
            first = false;

            // Print coefficient polynomial in y
            let coef_str = poly.to_string(var_y);
            if coef_str != "1" || i == 0 {
                if coef_str.contains('+') || coef_str.contains('-') {
                    result.push_str(&format!("({})", coef_str));
                } else {
                    result.push_str(&coef_str);
                }
            }

            // Print x and its exponent
            if i > 0 {
                result.push_str(var_x);
                if i > 1 {
                    result.push_str(&format!("^{}", i));
                }
            }
        }

        if result.is_empty() {
            "0".to_string()
        } else {
            result
        }
    }

    pub fn points_at_fixed_x(&self, x: f64, y_low: f64, y_high: f64) -> Vec<FInt> {
        // Evaluate the polynomial at fixed x to get a polynomial in y
        let mut poly_y = XPoly::new(vec![]);
        for (i, term_poly_in_y) in self.0.iter().enumerate() {
            let x_power = FInt::new(x.powi(i as i32));
            let term = term_poly_in_y * &XPoly::new(vec![x_power]);
            poly_y = &poly_y + &term;
        }
        poly_y.get_roots(y_low, y_high)
    }

    pub fn points_at_fixed_y(&self, y: f64, x_low: f64, x_high: f64) -> Vec<FInt> {
        // Evaluate the polynomial at fixed y to get a polynomial in x
        if self.0.is_empty() {
            return vec![];
        }
        let mut poly_x = XPoly::new(vec![FInt::new(1.0); self.0.len()]);
        for (i, term_poly_in_y) in self.0.iter().enumerate() {
            poly_x.0[i] = term_poly_in_y.evaluate(FInt::new(y));
        }
        poly_x.cleanup();
        poly_x.get_roots(x_low, x_high)
    }
}

impl fmt::Display for XYPoly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string("x", "y"))
    }
}

impl fmt::Debug for XYPoly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string("x", "y"))
    }
}

#[cfg(test)]
mod tests {
    use approx::relative_eq;

    use super::*;

    #[test]
    fn test_evaluate() {
        // Test constant polynomial
        let poly = XPoly::new(vec![FInt::new(5.0)]);
        let result = poly.evaluate(FInt::new(2.0));
        relative_eq!(result.midpoint(), 5.0);

        // Test linear polynomial
        let poly = XPoly::new(vec![FInt::new(2.0), FInt::new(3.0)]);
        let result = poly.evaluate(FInt::new(2.0));
        relative_eq!(result.midpoint(), 8.0);

        // Test quadratic polynomial
        let poly = XPoly::new(vec![FInt::new(1.0), FInt::new(2.0), FInt::new(3.0)]);
        let result = poly.evaluate(FInt::new(2.0));
        relative_eq!(result.midpoint(), 17.0);

        // Test with interval input
        let poly = XPoly::new(vec![FInt::new(1.0), FInt::new(2.0), FInt::new(3.0)]);
        let result = poly.evaluate(FInt::new_with_bounds(1.0, 3.0));
        assert!(result.almost_equals(FInt::new_with_bounds(6.0, 34.0)));
    }

    #[test]
    fn test_divide_by_monomial() {
        // Test division of constant by x - a
        let poly = XPoly::new(vec![FInt::new(5.0)]);
        let (quotient, remainder) = poly.divide_by_monomial(FInt::new(2.0));
        assert_eq!(quotient.0.len(), 0);
        assert_eq!(remainder.midpoint(), 5.0);

        // Test division of linear polynomial by x - a
        let poly = XPoly::new(vec![FInt::new(2.0), FInt::new(3.0)]);
        let (quotient, remainder) = poly.divide_by_monomial(FInt::new(2.0));
        assert_eq!(format!("{:?}", quotient), "3");
        relative_eq!(remainder.midpoint(), 8.0);

        // Test division of quadratic polynomial by x - a: 3x^2 + 2x + 1 = (x - 2)(3x + 8) + 17
        let poly = XPoly::new(vec![FInt::new(1.0), FInt::new(2.0), FInt::new(3.0)]);
        let (quotient, remainder) = poly.divide_by_monomial(FInt::new(2.0));
        assert_eq!(format!("{:?}", quotient), "8 + 3x");
        relative_eq!(remainder.midpoint(), 17.0);
    }

    #[test]
    fn test_xy_poly_evaluate() {
        // Test constant polynomial
        let poly = XYPoly::new(vec![XPoly::new(vec![FInt::new(5.0)])]);
        let result = poly.evaluate(FInt::new(2.0), FInt::new(3.0));
        relative_eq!(result.midpoint(), 5.0);

        // Test linear in x, constant in y
        let poly = XYPoly::new(vec![XPoly::new(vec![FInt::new(2.0), FInt::new(3.0)])]);
        let result = poly.evaluate(FInt::new(2.0), FInt::new(3.0));
        relative_eq!(result.midpoint(), 8.0);

        // Test constant in x, linear in y
        let poly = XYPoly::new(vec![
            XPoly::new(vec![FInt::new(2.0)]),
            XPoly::new(vec![FInt::new(3.0)]),
        ]);
        let result = poly.evaluate(FInt::new(2.0), FInt::new(3.0));
        relative_eq!(result.midpoint(), 11.0);

        // Test linear in both x and y
        let poly = XYPoly::new(vec![
            XPoly::new(vec![FInt::new(2.0), FInt::new(3.0)]),
            XPoly::new(vec![FInt::new(4.0), FInt::new(5.0)]),
        ]);
        let result = poly.evaluate(FInt::new(2.0), FInt::new(3.0));
        relative_eq!(result.midpoint(), 8.0 + 3.0 * 14.0);

        // Test quadratic in x, linear in y
        let poly = XYPoly::new(vec![
            XPoly::new(vec![FInt::new(1.0), FInt::new(2.0), FInt::new(3.0)]),
            XPoly::new(vec![FInt::new(4.0), FInt::new(5.0), FInt::new(6.0)]),
        ]);
        let result = poly.evaluate(FInt::new(2.0), FInt::new(3.0));
        relative_eq!(result.midpoint(), 17.0 + 3.0 * 38.0);
    }

    #[test]
    fn test_xpoly_ops() {
        let p1 = XPoly::new(vec![FInt::new(1.0), FInt::new(2.0)]); // 1 + 2x
        let p2 = XPoly::new(vec![FInt::new(3.0), FInt::new(4.0)]); // 3 + 4x

        // Test multiplication
        let result = &p1 * &p2;
        assert_eq!(format!("{:?}", result), "3 + 10x + 8x^2");

        // Test subtraction
        let result = &p1 - &p2;
        assert_eq!(format!("{:?}", result), "-2 - 2x");

        // Test subtraction with different degrees
        let p3 = XPoly::new(vec![FInt::new(1.0), FInt::new(2.0), FInt::new(3.0)]); // 1 + 2x + 3x^2
        let result = &p3 - &p1;
        assert_eq!(format!("{:?}", result), "3x^2");
    }

    #[test]
    fn test_resultant_linear() {
        // Test case 1: Simple polynomials
        let p = XYPoly::new(vec![
            XPoly::new(vec![FInt::new(1.0), FInt::new(2.0)]), // 1 + 2y
            XPoly::new(vec![FInt::new(3.0)]),                 // 3x
        ]);
        let q = XYPoly::new(vec![
            XPoly::new(vec![FInt::new(4.0), FInt::new(5.0)]), // 4 + 5y
            XPoly::new(vec![FInt::new(6.0)]),                 // 6x
        ]);
        let result = p.resultant(&q);

        assert_eq!(format!("{}", result), "-6 - 3x");
    }

    #[test]
    fn test_resultant_nonlinear() {
        // Test case 2: Higher degree polynomials
        // x^2 + 2y^2 = 3
        let p = XYPoly::new(vec![
            XPoly::new(vec![FInt::new(-3.0), FInt::new(0.0), FInt::new(2.0)]), // 2y^2 - 3
            XPoly::new(vec![]),
            XPoly::new(vec![FInt::new(1.0)]),
        ]);
        // 2x^2 + y^2 = 3
        let q = XYPoly::new(vec![
            XPoly::new(vec![FInt::new(-3.0), FInt::new(0.0), FInt::new(1.0)]), // y^2 - 3
            XPoly::new(vec![]),
            XPoly::new(vec![FInt::new(2.0)]),
        ]);
        let result = p.resultant(&q);
        assert_eq!(format!("{}", result), "9 - 18x^2 + 9x^4");
    }

    #[test]
    fn test_xpoly_to_string() {
        let p = XPoly::new(vec![
            FInt::new(1.0),  // 1
            FInt::new(2.0),  // 2x
            FInt::new(3.0),  // 3x^2
            FInt::new(-4.0), // -4x^3
        ]);
        assert_eq!(p.to_string("x"), "1 + 2x + 3x^2 - 4x^3");
        assert_eq!(p.to_string("y"), "1 + 2y + 3y^2 - 4y^3");

        let p = XPoly::new(vec![FInt::new(0.0), FInt::new(1.0)]); // x
        assert_eq!(p.to_string("x"), "x");

        let p = XPoly::new(vec![FInt::new(0.0)]); // 0
        assert_eq!(p.to_string("x"), "0");
    }

    #[test]
    fn test_xypoly_to_string() {
        let p = XYPoly::new(vec![
            XPoly::new(vec![FInt::new(1.0), FInt::new(2.0)]), // 1 + 2y
            XPoly::new(vec![FInt::new(3.0)]),                 // 3x
            XPoly::new(vec![FInt::new(4.0), FInt::new(5.0)]), // (4 + 5y)x^2
        ]);
        assert_eq!(p.to_string("x", "y"), "(1 + 2y) + 3x + (4 + 5y)x^2");
        assert_eq!(p.to_string("a", "b"), "(1 + 2b) + 3a + (4 + 5b)a^2");

        let result = p.evaluate(FInt::new(2.0), FInt::new(3.0));
        assert!(result == FInt::new(7.0 + 3.0 * 2.0 + 19.0 * 4.0));

        let p = XYPoly::new(vec![
            XPoly::new(vec![FInt::new(1.0)]), // 1
            XPoly::new(vec![FInt::new(0.0)]), // 0x
        ]);
        assert_eq!(p.to_string("x", "y"), "1");

        let p = XYPoly::new(vec![]); // 0
        assert_eq!(p.to_string("x", "y"), "0");
    }

    #[test]
    fn test_debug_formatting() {
        let p = XPoly::new(vec![
            FInt::new(1.0),  // 1
            FInt::new(2.0),  // 2x
            FInt::new(3.0),  // 3x^2
            FInt::new(-4.0), // -4x^3
        ]);
        assert_eq!(format!("{:?}", p), "1 + 2x + 3x^2 - 4x^3");

        let p = XYPoly::new(vec![
            XPoly::new(vec![FInt::new(1.0), FInt::new(2.0)]), // 1 + 2y
            XPoly::new(vec![FInt::new(3.0)]),                 // 3x
            XPoly::new(vec![FInt::new(4.0), FInt::new(5.0)]), // (4 + 5y)x^2
        ]);
        assert_eq!(format!("{:?}", p), "(1 + 2y) + 3x + (4 + 5y)x^2");
    }

    #[test]
    fn test_polynomial_division() {
        // Test case 1: x^2 + 2x + 1 divided by x + 1
        let p1 = XPoly::new(vec![
            FInt::new(1.0), // 1
            FInt::new(2.0), // 2x
            FInt::new(1.0), // x^2
        ]);
        let p2 = XPoly::new(vec![
            FInt::new(1.0), // 1
            FInt::new(1.0), // x
        ]);
        let (quotient, remainder) = p1.divide(&p2);
        assert_eq!(quotient.to_string("x"), "1 + x");
        assert_eq!(remainder.to_string("x"), "0");

        // Test case 2: x^3 + 2x^2 + 3x + 4 divided by x^2 + 1
        let p1 = XPoly::new(vec![
            FInt::new(4.0), // 4
            FInt::new(3.0), // 3x
            FInt::new(2.0), // 2x^2
            FInt::new(1.0), // x^3
        ]);
        let p2 = XPoly::new(vec![
            FInt::new(1.0), // 1
            FInt::new(0.0), // 0x
            FInt::new(1.0), // x^2
        ]);
        let (quotient, remainder) = p1.divide(&p2);
        assert_eq!(quotient.to_string("x"), "2 + x");
        assert_eq!(remainder.to_string("x"), "2 + 2x");
    }

    #[test]
    fn test_cleanup() {
        // Test cleanup of trailing zeros
        let mut p = XPoly::new(vec![
            FInt::new(1.0),
            FInt::new(2.0),
            FInt::new(0.0),
            FInt::new(0.0),
        ]);
        p.cleanup();
        assert_eq!(p.to_string("x"), "1 + 2x");

        // Test cleanup after operations
        let p1 = XPoly::new(vec![FInt::new(1.0), FInt::new(2.0)]);
        let p2 = XPoly::new(vec![FInt::new(1.0), FInt::new(-2.0)]);
        let result = &p1 + &p2;
        assert_eq!(result.to_string("x"), "2");
    }

    #[test]
    fn test_gcd() {
        // Test case 1: GCD of x^2 - 1 and x - 1 is x - 1
        let p1 = XPoly::new(vec![
            FInt::new(-1.0), // -1
            FInt::new(0.0),  // 0x
            FInt::new(1.0),  // x^2
        ]);
        let p2 = XPoly::new(vec![
            FInt::new(-1.0), // -1
            FInt::new(1.0),  // x
        ]);
        let gcd = p1.gcd(&p2);
        assert_eq!(gcd.to_string("x"), "-1 + x");

        // Test case 2: GCD of x^3 - 2x^2 - x + 2 and x^2 - 1 is x^2 - 1
        let p1 = XPoly::new(vec![
            FInt::new(2.0),  // 2
            FInt::new(-1.0), // -x
            FInt::new(-2.0), // -2x^2
            FInt::new(1.0),  // x^3
        ]);
        let p2 = XPoly::new(vec![
            FInt::new(-1.0), // -1
            FInt::new(0.0),  // 0x
            FInt::new(1.0),  // x^2
        ]);
        let gcd = p1.gcd(&p2);
        assert_eq!(gcd.to_string("x"), "-1 + x^2");

        // Test case 3: GCD of x^2 + 2x + 1 and x + 1 is x + 1
        let p1 = XPoly::new(vec![
            FInt::new(1.0), // 1
            FInt::new(2.0), // 2x
            FInt::new(1.0), // x^2
        ]);
        let p2 = XPoly::new(vec![
            FInt::new(1.0), // 1
            FInt::new(1.0), // x
        ]);
        let gcd = p1.gcd(&p2);
        assert_eq!(gcd.to_string("x"), "1 + x");

        // Test case 4: GCD of x^4 - 1 and x^2 - 1 is x^2 - 1
        let p1 = XPoly::new(vec![
            FInt::new(-1.0), // -1
            FInt::new(0.0),  // 0x
            FInt::new(0.0),  // 0x^2
            FInt::new(0.0),  // 0x^3
            FInt::new(1.0),  // x^4
        ]);
        let p2 = XPoly::new(vec![
            FInt::new(-1.0), // -1
            FInt::new(0.0),  // 0x
            FInt::new(1.0),  // x^2
        ]);
        let gcd = p1.gcd(&p2);
        assert_eq!(gcd.to_string("x"), "-1 + x^2");
    }

    #[test]
    #[should_panic(expected = "Cannot compute GCD of empty polynomials")]
    fn test_gcd_empty_polynomial() {
        let p1 = XPoly::new(vec![]);
        let p2 = XPoly::new(vec![FInt::new(1.0)]);
        let _ = p1.gcd(&p2);
    }

    #[test]
    fn test_derivative() {
        // Test derivative of x^2 + 2x + 1
        let p = XPoly::new(vec![
            FInt::new(1.0), // 1
            FInt::new(2.0), // 2x
            FInt::new(1.0), // x^2
        ]);
        let deriv = p.get_derivative();
        assert_eq!(deriv.to_string("x"), "2 + 2x");

        // Test derivative of constant
        let p = XPoly::new(vec![FInt::new(1.0)]);
        let deriv = p.get_derivative();
        assert_eq!(deriv.to_string("x"), "0");

        // Test derivative of empty polynomial
        let p = XPoly::new(vec![]);
        let deriv = p.get_derivative();
        assert_eq!(deriv.to_string("x"), "0");
    }

    #[test]
    fn test_root_finding() {
        // Test roots of x^2 - 1
        let p = XPoly::new(vec![
            FInt::new(-1.0), // -1
            FInt::new(0.0),  // 0x
            FInt::new(1.0),  // x^2
        ]);
        let roots = p.get_roots(-2.0, 2.0);
        println!("roots: {:?}", roots);
        assert_eq!(roots.len(), 2);
        assert!(roots
            .iter()
            .any(|r| (*r - FInt::new(1.0)).abs_bound() < 1e-12));
        assert!(roots
            .iter()
            .any(|r| (*r + FInt::new(-1.0)).abs_bound() < 1e-12));

        // Test roots of x^3 + x
        let p = XPoly::new(vec![
            FInt::new(0.0), // 0
            FInt::new(1.0), // x
            FInt::new(0.0), // 0x^2
            FInt::new(1.0), // x^3
        ]);
        let roots = p.get_roots(-2.0, 2.0);
        println!("roots: {:?}", roots);
        assert_eq!(roots.len(), 1);
        assert!(roots.iter().any(|r| r.abs_bound() < 1e-12));

        // Test roots of x^3 - x
        let p = XPoly::new(vec![
            FInt::new(0.0),  // 0
            FInt::new(-1.0), // -x
            FInt::new(0.0),  // 0x^2
            FInt::new(1.0),  // x^3
        ]);
        let roots = p.get_roots(-2.0, 2.0);
        assert_eq!(roots.len(), 3);
        println!("roots: {:?}", roots);
        assert!(roots
            .iter()
            .any(|r| (*r - FInt::new(1.0)).abs_bound() < 1e-12));
        assert!(roots
            .iter()
            .any(|r| (*r + FInt::new(-1.0)).abs_bound() < 1e-12));
        assert!(roots.iter().any(|r| r.abs_bound() < 1e-12));
    }

    #[test]
    fn test_fixed_points() {
        // Test points on circle x^2 + y^2 - 1 = 0
        // Represented as: (y^2 - 1) + 0x + x^2
        let circle = XYPoly::new(vec![
            XPoly::new(vec![
                FInt::new(-1.0), // -1
                FInt::new(0.0),  // 0y
                FInt::new(1.0),  // y^2
            ]), // constant term in x
            XPoly::new(vec![FInt::new(0.0)]), // 0x
            XPoly::new(vec![FInt::new(1.0)]), // x^2
        ]);

        // Test points at x = 0.6
        let y_points = circle.points_at_fixed_x(0.6, -2.0, 2.0);
        assert_eq!(y_points.len(), 2);
        // y should be approximately ±0.8 (since 0.6^2 + 0.8^2 = 1)
        assert!(y_points
            .iter()
            .any(|y| (*y - FInt::new(0.8)).abs_bound() < 1e-12));
        assert!(y_points
            .iter()
            .any(|y| (*y + FInt::new(-0.8)).abs_bound() < 1e-12));

        // Test points at y = 0.6
        let x_points = circle.points_at_fixed_y(0.6, -2.0, 2.0);
        assert_eq!(x_points.len(), 2);
        // x should be approximately ±0.8 (since 0.6^2 + 0.8^2 = 1)
        assert!(x_points
            .iter()
            .any(|x| (*x - FInt::new(0.8)).abs_bound() < 1e-12));
        assert!(x_points
            .iter()
            .any(|x| (*x + FInt::new(-0.8)).abs_bound() < 1e-12));
    }

    #[test]
    fn test_flip() {
        // Test flipping a simple polynomial: 1 + 2y + 3x
        let poly = XYPoly::new(vec![
            XPoly::new(vec![FInt::new(1.0), FInt::new(2.0)]), // 1 + 2y
            XPoly::new(vec![FInt::new(3.0)]),                 // 3
        ]);

        let flipped = poly.flip();

        // The flipped polynomial should be: 1 + 2x + 3y
        assert_eq!(flipped.0.len(), 2);

        // First coefficient: 1 + 3x
        assert_eq!(flipped.0[0].0.len(), 2);
        assert_eq!(flipped.0[0].0[0], FInt::new(1.0));
        assert_eq!(flipped.0[0].0[1], FInt::new(3.0));

        // Second coefficient: 2
        assert_eq!(flipped.0[1].0.len(), 1);
        assert_eq!(flipped.0[1].0[0], FInt::new(2.0));

        // Test flipping an empty polynomial
        let empty_poly = XYPoly::new(vec![]);
        let flipped_empty = empty_poly.flip();
        assert_eq!(flipped_empty.0.len(), 0);

        // Test flipping a constant polynomial: 5
        let const_poly = XYPoly::new(vec![XPoly::new(vec![FInt::new(5.0)])]);
        let flipped_const = const_poly.flip();
        assert_eq!(flipped_const.0.len(), 1);
        assert_eq!(flipped_const.0[0].0.len(), 1);
        assert_eq!(flipped_const.0[0].0[0], FInt::new(5.0));

        // Test flipping a more complex polynomial: 1 + 2y + 3y² + 4x + 5xy
        let complex_poly = XYPoly::new(vec![
            XPoly::new(vec![FInt::new(1.0), FInt::new(2.0), FInt::new(3.0)]), // 1 + 2y + 3y²
            XPoly::new(vec![FInt::new(4.0), FInt::new(5.0)]),                 // 4 + 5y
        ]);

        let flipped_complex = complex_poly.flip();

        // The flipped polynomial should be: 1 + 2x + 3x² + 4y + 5xy
        assert_eq!(flipped_complex.0.len(), 3);

        // First coefficient: 1 + 4x
        assert_eq!(flipped_complex.0[0].0.len(), 2);
        assert_eq!(flipped_complex.0[0].0[0], FInt::new(1.0));
        assert_eq!(flipped_complex.0[0].0[1], FInt::new(4.0));

        // Second coefficient: 2 + 5x
        assert_eq!(flipped_complex.0[1].0.len(), 2);
        assert_eq!(flipped_complex.0[1].0[0], FInt::new(2.0));
        assert_eq!(flipped_complex.0[1].0[1], FInt::new(5.0));

        // Third coefficient: 3
        assert_eq!(flipped_complex.0[2].0.len(), 1);
        assert_eq!(flipped_complex.0[2].0[0], FInt::new(3.0));
    }
}
