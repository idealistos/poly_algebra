use gcd::Gcd;
pub use poly_conversion::PolyConversion;
pub use poly_operations::PolyOperations;
pub use poly_operations::SingleOutResult;
use std::collections::HashMap;
use std::{fmt, mem, rc::Rc};

use crate::modular_poly::ModularPoly;

mod poly_conversion;
mod poly_operations;

#[derive(Debug, Clone)]
pub struct Term {
    pub constant: i64,
    pub vars: Vec<(u8, u32)>, // (variable index, degree)
}

#[derive(Debug)]
pub enum ParseError {
    InvalidVariable(String),
    InvalidTerm(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidVariable(s) => write!(f, "Invalid variable name: {}", s),
            ParseError::InvalidTerm(s) => write!(f, "Invalid term: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
struct EliminationStep {
    pub v: u8,
    pub poly1: Rc<Poly>,
    pub poly2: Rc<Poly>,
    pub p_factor_1a: Rc<Poly>,
    pub p_factor_2a: Rc<Poly>,
    pub p_factor_1b: Rc<Poly>,
    pub p_factor_2b: Rc<Poly>,
    pub poly_a: Rc<Poly>, // poly1 * p_factor_1a + poly2 * p_factor_2a
    pub poly_b: Rc<Poly>, // poly1 * p_factor_1b + poly2 * p_factor_2b
    pub degree_a: u32,
    pub degree_b: u32,
}

impl EliminationStep {
    pub fn new(v: u8, poly1: Rc<Poly>, poly2: Rc<Poly>) -> Self {
        let degree1 = poly1.get_degree(v);
        let degree2 = poly2.get_degree(v);
        let (x_poly_1, x_poly_2, x_degree_1, x_degree_2) = if degree1 >= degree2 {
            (poly1, poly2, degree1, degree2)
        } else {
            (poly2, poly1, degree2, degree1)
        };
        Self {
            v,
            poly1: x_poly_1.clone(),
            poly2: x_poly_2.clone(),
            p_factor_1a: Rc::new(Poly::Constant(1)),
            p_factor_2a: Rc::new(Poly::Constant(0)),
            p_factor_1b: Rc::new(Poly::Constant(0)),
            p_factor_2b: Rc::new(Poly::Constant(1)),
            poly_a: x_poly_1.clone(),
            poly_b: x_poly_2.clone(),
            degree_a: x_degree_1,
            degree_b: x_degree_2,
        }
    }

    pub fn get_next_step(&self) -> Option<Self> {
        if self.degree_b == 0 {
            return None;
        }

        // Extract factors and remainders
        println!("poly_a: {} {:?}", self.poly_a, self.poly_a);
        let (pa1, pa2) = self
            .poly_a
            .extract_factor_and_remainder(self.v, self.degree_b);
        let (pb1, pb2) = self
            .poly_b
            .extract_factor_and_remainder(self.v, self.degree_b);

        // Compute new poly_b = pb1 * pa2 - pb2 * pa1

        println!("pa1: {} {:?}", pa1, pa1);
        println!("pa2: {} {:?}", pa2, pa2);
        println!("pb1: {} {:?}", pb1, pb1);
        let mut new_poly_b = pa2.multiply(&pb1);
        println!("pa2 * pb1: {} {:?}", new_poly_b, new_poly_b);
        let temp = pa1.multiply(&pb2);
        // println!(
        //     "new_poly_b before subtracting temp: {} {:?}",
        //     new_poly_b, new_poly_b
        // );
        // println!("temp: {} {:?}", temp, temp);
        new_poly_b.add_poly_scaled(&temp, -1);

        // Compute new factors
        let mut p_factor_1b = self.p_factor_1a.multiply(&pb1);
        let temp = self.p_factor_1b.multiply(&pa1);
        p_factor_1b.add_poly_scaled(&temp, -1);

        let mut p_factor_2b = self.p_factor_2a.multiply(&pb1);
        let temp = self.p_factor_2b.multiply(&pa1);
        p_factor_2b.add_poly_scaled(&temp, -1);

        println!("new_poly_b: {} {:?}", new_poly_b, new_poly_b);
        let degree_b = new_poly_b.get_degree(self.v);

        Some(Self {
            v: self.v,
            poly1: self.poly1.clone(),
            poly2: self.poly2.clone(),
            p_factor_1a: self.p_factor_1b.clone(),
            p_factor_2a: self.p_factor_2b.clone(),
            p_factor_1b: Rc::new(p_factor_1b),
            p_factor_2b: Rc::new(p_factor_2b),
            poly_a: self.poly_b.clone(),
            poly_b: Rc::new(new_poly_b),
            degree_a: self.degree_b,
            degree_b,
        })
    }
}

#[derive(Clone)]
pub enum Poly {
    Constant(i64),
    Nested(u8, Vec<Rc<Poly>>),
}

impl PartialEq for Poly {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Poly::Constant(n1), Poly::Constant(n2)) => n1 == n2,
            (Poly::Nested(v1, polys1), Poly::Nested(v2, polys2)) => v1 == v2 && polys1 == polys2,
            _ => false,
        }
    }
}

impl Eq for Poly {}

impl Poly {
    pub fn parse_var(s: &str) -> Result<u8, ParseError> {
        if s.is_empty() {
            return Err(ParseError::InvalidVariable(s.to_string()));
        }
        let first_char = s.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() {
            return Err(ParseError::InvalidVariable(s.to_string()));
        }
        let base = (first_char.to_ascii_lowercase() as u8) - b'a';
        if s.len() == 1 {
            return Ok(base);
        }
        if let Ok(num) = s[1..].parse::<u8>() {
            Ok(base + num * 26)
        } else {
            Err(ParseError::InvalidVariable(s.to_string()))
        }
    }

    fn parse_term(term_str: &str) -> Result<Term, ParseError> {
        let mut constant = 1;
        let mut degrees = HashMap::new();
        let term_str_to_use = if term_str.len() > 1
            && term_str.starts_with('-')
            && term_str.chars().nth(1).unwrap().is_alphabetic()
        {
            format!("-1*{}", &term_str[1..])
        } else {
            term_str.to_string()
        };
        let parts: Vec<&str> = term_str_to_use.split('*').collect();

        for part in parts {
            if part.is_empty() {
                return Err(ParseError::InvalidTerm("Empty part in term".to_string()));
            }

            if part.chars().next().unwrap().is_alphabetic() {
                // Handle variable part (e.g., "a^2" or "b")
                let var_degree: Vec<&str> = part.split('^').collect();
                if var_degree.len() > 2 {
                    return Err(ParseError::InvalidTerm(format!(
                        "Invalid variable format in '{}'",
                        part
                    )));
                }

                let degree = if var_degree.len() == 2 {
                    var_degree[1].parse::<u32>().map_err(|_| {
                        ParseError::InvalidTerm(format!("Invalid degree in '{}'", part))
                    })?
                } else {
                    1
                };
                if degree < 1 {
                    return Err(ParseError::InvalidTerm(format!(
                        "Invalid degree in '{}'",
                        part
                    )));
                }

                let var_index = Self::parse_var(var_degree[0])?;
                *degrees.entry(var_index).or_insert(0) += degree;
            } else {
                // Handle constant part
                let num_str = if let Some(without_plus) = part.strip_prefix('+') {
                    without_plus
                } else if let Some(without_minus) = part.strip_prefix('-') {
                    without_minus
                } else {
                    part
                };

                if num_str.is_empty() {
                    return Err(ParseError::InvalidTerm(format!(
                        "Invalid constant part '{}'",
                        part
                    )));
                }

                let num = num_str.parse::<i64>().map_err(|_| {
                    ParseError::InvalidTerm(format!("Invalid constant '{}'", num_str))
                })?;

                constant *= if part.starts_with('-') { -num } else { num };
            }
        }

        Ok(Term {
            constant,
            vars: degrees.into_iter().collect(),
        })
    }

    fn from_terms(terms: &[Term], used_vars: &[bool], var_index: u8) -> Self {
        // Find the next used variable
        let next_var = used_vars
            .iter()
            .enumerate()
            .skip(var_index as usize)
            .find(|(_, &used)| used)
            .map(|(i, _)| i as u8);

        match next_var {
            None => {
                // No more variables, this is a constant
                let sum: i64 = terms
                    .iter()
                    .filter(|term| term.vars.is_empty())
                    .map(|term| term.constant)
                    .sum();
                Poly::Constant(sum)
            }
            Some(v) => {
                // Group terms by degree of current variable
                let mut max_degree = 0;
                let mut terms_by_degree: Vec<Vec<Term>> = Vec::new();

                for term in terms {
                    let degree = term
                        .vars
                        .iter()
                        .find(|(var, _)| *var == v)
                        .map(|(_, d)| *d)
                        .unwrap_or(0);

                    max_degree = max_degree.max(degree);
                    while terms_by_degree.len() <= degree as usize {
                        terms_by_degree.push(Vec::new());
                    }

                    let mut new_term = term.clone();
                    new_term.vars.retain(|(var, _)| *var != v);
                    terms_by_degree[degree as usize].push(new_term);
                }

                // Convert each group to a polynomial
                let mut polys = Vec::new();
                for degree in 0..=max_degree {
                    let terms = &terms_by_degree[degree as usize];
                    if !terms.is_empty() {
                        polys.push(Rc::new(Self::from_terms(terms, used_vars, v + 1)));
                    } else {
                        polys.push(Rc::new(Poly::Constant(0)));
                    }
                }

                let mut poly = Poly::Nested(v, polys);
                poly.cleanup();
                poly
            }
        }
    }

    pub fn new(poly_str: &str) -> Result<Self, ParseError> {
        let mut terms = Vec::new();
        let mut current_term = String::new();
        let mut sign = 1i64;

        for c in poly_str.chars() {
            match c {
                '+' | '-' if !current_term.is_empty() && !current_term.ends_with('*') => {
                    let mut term = Self::parse_term(&current_term)?;
                    term.constant *= sign;
                    terms.push(term);
                    current_term.clear();
                    sign = if c == '+' { 1 } else { -1 };
                }
                ' ' => continue,
                _ => current_term.push(c),
            }
        }

        if !current_term.is_empty() {
            let mut term = Self::parse_term(&current_term)?;
            term.constant *= sign;
            terms.push(term);
        }

        // Step 2: Find used variables
        let mut used_vars = [false; 256];
        for term in &terms {
            for (var, _) in &term.vars {
                used_vars[*var as usize] = true;
            }
        }

        // Step 3: Convert terms to polynomial
        Ok(Self::from_terms(&terms, &used_vars, 0))
    }

    pub fn cleanup(&mut self) {
        match self {
            Poly::Constant(_) => {}
            Poly::Nested(_, polys) => {
                // First, cleanup all nested polynomials
                for poly in polys.iter_mut() {
                    let poly_mut = Rc::make_mut(poly);
                    poly_mut.cleanup();
                }

                // Remove trailing zero terms
                while let Some(p) = polys.last() {
                    if let Poly::Constant(0) = p.as_ref() {
                        polys.pop();
                    } else {
                        break;
                    }
                }

                // If we have only one term left, replace the entire Nested with it
                if polys.len() == 1 {
                    let poly = Rc::make_mut(&mut polys[0]);
                    let mut new_poly = Poly::Constant(0);
                    mem::swap(poly, &mut new_poly);
                    *self = new_poly;
                } else if polys.is_empty() {
                    *self = Poly::Constant(0);
                }
            }
        }
    }

    pub fn to_terms(&self) -> Vec<Term> {
        match self {
            Poly::Constant(n) => {
                if *n == 0 {
                    vec![]
                } else {
                    vec![Term {
                        constant: *n,
                        vars: vec![],
                    }]
                }
            }
            Poly::Nested(v, polys) => {
                let mut terms = Vec::new();
                for (i, poly) in polys.iter().enumerate() {
                    let sub_terms = poly.to_terms();
                    for mut term in sub_terms {
                        if i > 0 {
                            term.vars.push((*v, i as u32));
                        }
                        terms.push(term);
                    }
                }
                terms
            }
        }
    }

    fn var_to_string(var_idx: u8) -> String {
        let base = var_idx / 26;
        let offset = var_idx % 26;
        let c = (b'a' + offset) as char;
        if base == 0 {
            c.to_string()
        } else {
            format!("{}{}", c, base)
        }
    }

    pub fn get_degree(&self, v: u8) -> u32 {
        match self {
            Poly::Constant(_) => 0,
            Poly::Nested(v1, polys) => {
                if *v1 > v {
                    0
                } else if *v1 == v {
                    polys.len() as u32 - 1
                } else {
                    polys.iter().map(|p| p.get_degree(v)).max().unwrap_or(0)
                }
            }
        }
    }

    pub fn fill_in_variables(&self, vars: &mut [bool; 256]) {
        match self {
            Poly::Constant(_) => {}
            Poly::Nested(v, polys) => {
                vars[*v as usize] = true;
                for poly in polys {
                    poly.fill_in_variables(vars);
                }
            }
        }
    }

    /// Observes all coefficients in the polynomial by calling a function for each coefficient
    /// The polynomial itself is not mutated by this method
    pub fn observe_coefficients<F>(&self, mut f: F)
    where
        F: FnMut(i64),
    {
        self.observe_coefficients_with(&mut f);
    }

    /// Internal method that takes a mutable reference to avoid recursive type issues
    fn observe_coefficients_with<F>(&self, f: &mut F)
    where
        F: FnMut(i64),
    {
        match self {
            Poly::Constant(n) => {
                f(*n);
            }
            Poly::Nested(_, polys) => {
                for poly in polys {
                    poly.observe_coefficients_with(f);
                }
            }
        }
    }

    /// Applies a function to all coefficients in the polynomial, mutating it
    pub fn apply_to_coefficients<F>(&mut self, mut f: F)
    where
        F: FnMut(i64) -> i64,
    {
        self.apply_to_coefficients_with(&mut f);
        self.cleanup();
    }

    /// Internal method that takes a mutable reference to avoid recursive type issues
    fn apply_to_coefficients_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(i64) -> i64,
    {
        match self {
            Poly::Constant(n) => {
                *n = f(*n);
            }
            Poly::Nested(_, polys) => {
                for poly in polys {
                    let poly_mut = Rc::make_mut(poly);
                    poly_mut.apply_to_coefficients_with(f);
                }
            }
        }
    }

    /// Reduces coefficients by dividing by their GCD if the largest coefficient is above 10000
    pub fn reduce_coefficients_if_large(&mut self) {
        // Find the largest absolute value using observe_coefficients
        let mut max_abs_coeff = 0;
        self.observe_coefficients(|x| {
            max_abs_coeff = max_abs_coeff.max(x.abs());
        });

        // Only proceed if the largest coefficient is above 10000
        if max_abs_coeff <= 10000 {
            return;
        }

        // Find GCD of all coefficients using observe_coefficients
        let mut gcd_value = 1u64;
        let mut first_coeff = true;
        self.observe_coefficients(|x| {
            if x == 0 {
                return;
            }
            if first_coeff {
                gcd_value = x.unsigned_abs();
                first_coeff = false;
            } else {
                gcd_value = gcd_value.gcd(x.unsigned_abs());
            }
        });

        // If GCD is 1, no reduction needed
        if gcd_value == 1 {
            return;
        }

        // Divide all coefficients by GCD using apply_to_coefficients
        self.apply_to_coefficients(|x| x / (gcd_value as i64));
    }

    /// Retains only the polynomials that are needed for finding the equation F(x, y) = 0
    pub fn retain_relevant_polys(polys: Vec<Poly>, x_var: u8, y_var: u8) -> Vec<Poly> {
        // Find variables used in each polynomial
        let mut vars_used_in_poly: Vec<[bool; 256]> = Vec::new();
        for poly in &polys {
            let mut vars = [false; 256];
            poly.fill_in_variables(&mut vars);
            vars_used_in_poly.push(vars);
        }

        // Initialize poly_needed to false for each poly
        let mut poly_needed = vec![false; polys.len()];

        // Initialize vars_needed to true for x_var and y_var, false otherwise
        let mut vars_needed = [false; 256];
        vars_needed[x_var as usize] = true;
        vars_needed[y_var as usize] = true;

        // Iteratively find relevant polynomials
        loop {
            // Find the first index i for which poly_needed[i] is false and
            // vars_used_in_poly[i] intersects vars_needed
            let mut found_index = None;
            for (i, &needed) in poly_needed.iter().enumerate() {
                if !needed {
                    // Check if vars_used_in_poly[i] intersects vars_needed
                    let mut has_intersection = false;
                    for (j, is_needed) in vars_needed.iter().enumerate() {
                        if vars_used_in_poly[i][j] && *is_needed {
                            has_intersection = true;
                            break;
                        }
                    }
                    if has_intersection {
                        found_index = Some(i);
                        break;
                    }
                }
            }

            // If no intersection found, break the loop
            if found_index.is_none() {
                break;
            }

            let i = found_index.unwrap();

            // Set poly_needed[i] to true
            poly_needed[i] = true;

            // Set vars_needed to the union of vars_needed and vars_used_in_poly[i]
            for (j, is_needed) in vars_needed.iter_mut().enumerate() {
                *is_needed = *is_needed || vars_used_in_poly[i][j];
            }
        }

        // Return just the polys for which poly_needed[i] is true
        polys
            .into_iter()
            .enumerate()
            .filter(|(i, _)| poly_needed[*i])
            .map(|(_, poly)| poly)
            .collect()
    }

    /// Substitute variables with modular polynomials
    ///
    /// Given a polynomial f(x1, x2, ..., xn) and a map of variable substitutions
    /// where each variable xi is replaced by a modular polynomial pi(t),
    /// returns f(p1(t), p2(t), ..., pn(t)) as a modular polynomial.
    pub fn substitute_modular_polys(&self, var_polys: &HashMap<u8, ModularPoly>) -> ModularPoly {
        let p = var_polys.values().next().map(|poly| poly.p).unwrap();
        match self {
            Poly::Constant(c) => {
                // Convert constant to modular polynomial
                ModularPoly::constant(ModularPoly::from_i64(*c, p), p)
            }
            Poly::Nested(v, coeffs) => {
                // Get the modular polynomial for this variable
                let var_poly = var_polys.get(v).unwrap();

                // Compute the result by evaluating the polynomial at var_poly
                let mut result = ModularPoly::zero(var_poly.p);
                let mut power = ModularPoly::constant(1, var_poly.p);

                for coeff in coeffs {
                    // Substitute the coefficient polynomial
                    let coeff_result = coeff.substitute_modular_polys(var_polys);

                    // Multiply by the current power of the variable polynomial
                    let term = &coeff_result * &power;
                    result = &result + &term;

                    // Update power for next iteration
                    power = &power * var_poly;
                }

                result
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_debug() {
        // Test constant
        let p = Poly::new("5").unwrap();
        assert_eq!(format!("{}", p), "5");
        assert_eq!(format!("{:?}", p), "Constant(5)");

        // Test simple polynomial
        let p = Poly::new("1 + 2*a + 3*a^2").unwrap();
        assert_eq!(format!("{}", p), "1 + 2*a + 3*a^2");
        assert_eq!(
            format!("{:?}", p),
            "Nested(0, [Constant(1), Constant(2), Constant(3)])"
        );

        // Test multivariate polynomial
        let p = Poly::new("1 + 2*a + 3*b + 4*a*b").unwrap();
        assert_eq!(format!("{}", p), "1 + 3*b + 2*a + 4*b*a");
        assert_eq!(
            format!("{:?}", p),
            "Nested(0, [Nested(1, [Constant(1), Constant(3)]), Nested(1, [Constant(2), Constant(4)])])"
        );

        // Test nested polynomial with higher variable
        let p = Poly::new("1 + 2*b + 3*b^2").unwrap();
        assert_eq!(format!("{}", p), "1 + 2*b + 3*b^2");
        assert_eq!(
            format!("{:?}", p),
            "Nested(1, [Constant(1), Constant(2), Constant(3)])"
        );

        // Test complex polynomial
        let p = Poly::new("a^2*b + 2*a*b^2 + 3*b^3").unwrap();
        assert_eq!(format!("{}", p), "3*b^3 + 2*b^2*a + b*a^2");
        assert_eq!(
            format!("{:?}", p),
            "Nested(0, [Nested(1, [Constant(0), Constant(0), Constant(0), Constant(3)]), Nested(1, [Constant(0), Constant(0), Constant(2)]), Nested(1, [Constant(0), Constant(1)])])"
        );
    }

    #[test]
    fn test_constant() {
        let p = Poly::new("5").unwrap();
        assert_eq!(format!("{}", p), "5");

        let p = Poly::new("-5").unwrap();
        assert_eq!(format!("{}", p), "-5");
    }

    #[test]
    fn test_simple_polynomial() {
        // 2*a^2 + 3*b
        let p = Poly::new("2*a^2 + 3*b").unwrap();
        assert_eq!(format!("{}", p), "3*b + 2*a^2");

        // -2*a^2 - 3*b
        let p = Poly::new("-2*a^2 - 3*b").unwrap();
        assert_eq!(format!("{}", p), "-3*b - 2*a^2");
    }

    #[test]
    fn test_multivariate_polynomial() {
        // 2*a^2*b + 1*b^3
        let p = Poly::new("2*a^2*b + b^3").unwrap();
        assert_eq!(format!("{}", p), "b^3 + 2*b*a^2");

        // -2*a^2*b + 1*b^3
        let p = Poly::new("-2*a^2*b + b^3").unwrap();
        assert_eq!(format!("{}", p), "b^3 - 2*b*a^2");

        // 2*a^2*b - 1*b^3
        let p = Poly::new("2*a^2*b - b^3").unwrap();
        assert_eq!(format!("{}", p), "-b^3 + 2*b*a^2");
    }

    #[test]
    fn test_cleanup() {
        // Test removing trailing zeros
        let mut p = Poly::Nested(
            0,
            vec![
                Rc::new(Poly::Constant(1)),
                Rc::new(Poly::Constant(2)),
                Rc::new(Poly::Constant(0)),
                Rc::new(Poly::Constant(0)),
            ],
        );
        p.cleanup();
        assert_eq!(format!("{}", p), "1 + 2*a");

        // Test removing nested polynomial with single constant
        let mut p = Poly::Nested(
            0,
            vec![
                Rc::new(Poly::Nested(1, vec![Rc::new(Poly::Constant(5))])),
                Rc::new(Poly::Constant(0)),
            ],
        );
        p.cleanup();
        assert_eq!(format!("{}", p), "5");

        // Test removing nested polynomial with single constant and trailing zeros
        let mut p = Poly::Nested(
            0,
            vec![
                Rc::new(Poly::Nested(
                    1,
                    vec![
                        Rc::new(Poly::Constant(5)),
                        Rc::new(Poly::Constant(0)),
                        Rc::new(Poly::Constant(0)),
                    ],
                )),
                Rc::new(Poly::Constant(0)),
            ],
        );
        p.cleanup();
        assert_eq!(format!("{}", p), "5");

        // Test complex cleanup
        let mut p = Poly::Nested(
            0,
            vec![
                Rc::new(Poly::Nested(
                    1,
                    vec![
                        Rc::new(Poly::Constant(1)),
                        Rc::new(Poly::Constant(0)),
                        Rc::new(Poly::Constant(0)),
                    ],
                )),
                Rc::new(Poly::Constant(0)),
                Rc::new(Poly::Constant(0)),
            ],
        );
        p.cleanup();
        assert_eq!(format!("{}", p), "1");

        // Test cleanup with multiple levels
        let mut p = Poly::Nested(
            0,
            vec![
                Rc::new(Poly::Nested(
                    1,
                    vec![
                        Rc::new(Poly::Nested(
                            2,
                            vec![Rc::new(Poly::Constant(3)), Rc::new(Poly::Constant(0))],
                        )),
                        Rc::new(Poly::Constant(0)),
                    ],
                )),
                Rc::new(Poly::Constant(0)),
            ],
        );
        p.cleanup();
        assert_eq!(format!("{}", p), "3");

        // Test cleanup in the middle variable
        let mut p = Poly::Nested(
            0,
            vec![
                Rc::new(Poly::Nested(
                    1,
                    vec![
                        Rc::new(Poly::Nested(
                            2,
                            vec![Rc::new(Poly::Constant(0)), Rc::new(Poly::Constant(2))],
                        )),
                        Rc::new(Poly::Constant(0)),
                    ],
                )),
                Rc::new(Poly::Constant(1)),
            ],
        );
        p.cleanup();
        assert_eq!(format!("{}", p), "2*c + a");
    }

    #[test]
    fn test_cleanup_removes_empty_branches() {
        let mut p = Poly::Nested(
            1,
            vec![
                Rc::new(Poly::Nested(2, vec![Rc::new(Poly::Constant(0))])),
                Rc::new(Poly::Constant(-2)),
            ],
        );
        p.cleanup();
        let poly1 = Poly::new("-2*b").unwrap();
        assert_eq!(format!("{:?}", p), format!("{:?}", poly1));
    }

    #[test]
    fn test_eq() {
        // Test constant equality
        let p1 = Poly::new("5").unwrap();
        let p2 = Poly::new("5").unwrap();
        assert_eq!(p1, p2);

        let p1 = Poly::new("5").unwrap();
        let p2 = Poly::new("-5").unwrap();
        assert_ne!(p1, p2);

        // Test simple polynomial equality
        let p1 = Poly::new("1 + 2*a").unwrap();
        let p2 = Poly::new("1 + 2*a").unwrap();
        assert_eq!(p1, p2);

        // Test different polynomials
        let p1 = Poly::new("1 + 2*a").unwrap();
        let p2 = Poly::new("1 + 3*a").unwrap();
        assert_ne!(p1, p2);

        // Test nested polynomial equality
        let p1 = Poly::new("1 + 2*b + 3*a").unwrap();
        let p2 = Poly::new("1 + 2*b + 3*a").unwrap();
        assert_eq!(p1, p2);

        // Test different variable indices
        let p1 = Poly::new("1 + 2*a").unwrap();
        let p2 = Poly::new("1 + 2*b").unwrap();
        assert_ne!(p1, p2);

        // Test cleanup and equality
        let mut p1 = Poly::new("1 + 2*a + 0*a^2").unwrap();
        let p2 = Poly::new("1 + 2*a").unwrap();
        p1.cleanup();
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_parse_simple() {
        // Test constant
        assert_eq!(format!("{}", Poly::new("5").unwrap()), "5");
        assert_eq!(format!("{}", Poly::new("-5").unwrap()), "-5");

        // Test simple polynomial
        assert_eq!(format!("{}", Poly::new("a^2").unwrap()), "a^2");
        assert_eq!(
            format!("{}", Poly::new("2*a^2 + 3*b").unwrap()),
            "3*b + 2*a^2"
        );
        assert_eq!(
            format!("{}", Poly::new("-2*a^2 - 3*b").unwrap()),
            "-3*b - 2*a^2"
        );

        // Test multivariate polynomial
        assert_eq!(format!("{}", Poly::new("a*b + a").unwrap()), "a + b*a");
        assert_eq!(format!("{}", Poly::new("a*b + b").unwrap()), "b + b*a");
    }

    #[test]
    fn test_parse_complex() {
        assert_eq!(
            format!("{}", Poly::new("2*a*b + 3*b^2").unwrap()),
            "3*b^2 + 2*b*a"
        );
        assert_eq!(
            format!("{}", Poly::new("2*a*b + 1*b^2").unwrap()),
            "b^2 + 2*b*a"
        );
        assert_eq!(format!("{}", Poly::new("a*b + b^2").unwrap()), "b^2 + b*a");
        assert_eq!(format!("{}", Poly::new("a*b + a^2").unwrap()), "b*a + a^2");

        // Test variables with numbers
        assert_eq!(format!("{}", Poly::new("a1 + b2").unwrap()), "b2 + a1");
        assert_eq!(
            format!("{}", Poly::new("2*a1^2 + 3*b2").unwrap()),
            "3*b2 + 2*a1^2"
        );

        // Test cleanup during parsing
        assert_eq!(
            format!("{}", Poly::new("1 + 2*a + 0*a^2").unwrap()),
            "1 + 2*a"
        );
        assert_eq!(format!("{}", Poly::new("5 + 0*b").unwrap()), "5");
    }

    #[test]
    fn test_invalid_variable_names() {
        assert_eq!(
            Poly::new("1@a").unwrap_err().to_string(),
            "Invalid term: Invalid constant '1@a'"
        );
        assert_eq!(
            Poly::new("a@1").unwrap_err().to_string(),
            "Invalid variable name: a@1"
        );
        assert_eq!(
            Poly::new("1a").unwrap_err().to_string(),
            "Invalid term: Invalid constant '1a'"
        );
    }

    #[test]
    fn test_invalid_degrees() {
        assert_eq!(
            Poly::new("a^0").unwrap_err().to_string(),
            "Invalid term: Invalid degree in 'a^0'"
        );
        assert_eq!(
            Poly::new("a^-1").unwrap_err().to_string(),
            "Invalid term: Invalid degree in 'a^'"
        );
        assert_eq!(
            Poly::new("a^2.5").unwrap_err().to_string(),
            "Invalid term: Invalid degree in 'a^2.5'"
        );
    }

    #[test]
    fn test_invalid_terms() {
        assert_eq!(
            Poly::new("a*b*").unwrap_err().to_string(),
            "Invalid term: Empty part in term"
        );
        assert_eq!(
            Poly::new("*a*b").unwrap_err().to_string(),
            "Invalid term: Empty part in term"
        );
        assert_eq!(
            Poly::new("a**b").unwrap_err().to_string(),
            "Invalid term: Empty part in term"
        );
    }

    #[test]
    fn test_get_degree() {
        // Test constant
        let p = Poly::new("5").unwrap();
        assert_eq!(p.get_degree(0), 0);
        assert_eq!(p.get_degree(1), 0);

        // Test simple polynomial
        let p = Poly::new("1 + 2*a + 3*a^2").unwrap();
        assert_eq!(p.get_degree(0), 2); // degree of a
        assert_eq!(p.get_degree(1), 0); // degree of b

        // Test multivariate polynomial
        let p = Poly::new("1 + 2*a + 3*b + 4*a*b + 5*b^2").unwrap();
        assert_eq!(p.get_degree(0), 1); // degree of a
        assert_eq!(p.get_degree(1), 2); // degree of b

        // Test nested polynomial with higher variable
        let p = Poly::new("1 + 2*b + 3*b^2").unwrap();
        assert_eq!(p.get_degree(0), 0); // degree of a
        assert_eq!(p.get_degree(1), 2); // degree of b

        // Test complex polynomial
        let p = Poly::new("a^2*b + 2*a*b^2 + 3*b^3").unwrap();
        assert_eq!(p.get_degree(0), 2); // degree of a
        assert_eq!(p.get_degree(1), 3); // degree of b

        // Test polynomial with multiple variables
        let p = Poly::new("a^2*b^3 + 2*a*b^2*c + 3*b*c^2").unwrap();
        assert_eq!(p.get_degree(0), 2); // degree of a
        assert_eq!(p.get_degree(1), 3); // degree of b
        assert_eq!(p.get_degree(2), 2); // degree of c
    }

    #[test]
    fn test_elimination_step() {
        let poly1 = Poly::new("a + a*c^2 - 1 + c^2").unwrap();
        let poly2 = Poly::new("b + b*c^2 - 2*c").unwrap();
        let v = 2; // c

        let step = EliminationStep::new(v, Rc::new(poly1), Rc::new(poly2));
        assert_eq!(step.degree_a, 2); // degree of c in poly1
        assert_eq!(step.degree_b, 2); // degree of c in poly2

        // First step
        let next_step = step.get_next_step().unwrap();
        assert_eq!(next_step.degree_a, 2); // degree of c in poly_b
        assert_eq!(next_step.degree_b, 1); // degree of c in new poly_b

        assert_eq!(format!("{}", next_step.poly_a), "-2*c + b + c^2*b");
        assert_eq!(format!("{}", next_step.poly_b), "2*c - 2*b + 2*c*a");

        assert_eq!(format!("{}", next_step.p_factor_1a), "0");
        assert_eq!(format!("{}", next_step.p_factor_2a), "1");
        assert_eq!(format!("{}", next_step.p_factor_1b), "b");
        assert_eq!(format!("{}", next_step.p_factor_2b), "-1 - a");

        let step3 = next_step.get_next_step().unwrap();
        assert_eq!(step3.degree_a, 1); // degree of c in poly_b
        assert_eq!(step3.degree_b, 1); // degree of c in new poly_b

        assert_eq!(format!("{}", step3.poly_a), "2*c - 2*b + 2*c*a");
        assert_eq!(format!("{}", step3.poly_b), "-2*b + 2*c*b^2 + 2*b*a");

        assert_eq!(format!("{}", step3.p_factor_1a), "b");
        assert_eq!(format!("{}", step3.p_factor_2a), "-1 - a");
        assert_eq!(format!("{}", step3.p_factor_1b), "2*b - c*b^2");
        assert_eq!(format!("{}", step3.p_factor_2b), "c*b + c*b*a");

        let step4 = step3.get_next_step().unwrap();
        assert_eq!(step4.degree_a, 1); // degree of c in poly_b
        assert_eq!(step4.degree_b, 0); // degree of c in new poly_b

        assert_eq!(format!("{}", step4.poly_a), "-2*b + 2*c*b^2 + 2*b*a");
        assert_eq!(format!("{}", step4.poly_b), "4*b - 4*b^3 - 4*b*a^2");
        assert_eq!(
            format!(
                "{} {} {} {}",
                step4.p_factor_1a, step4.p_factor_2a, step4.p_factor_1b, step4.p_factor_2b
            ),
            "2*b - c*b^2 c*b + c*b*a -4*b + 2*c*b^2 + 2*b^3 - 4*b*a + 2*c*b^2*a -2*c*b - 2*b^2 - 4*c*b*a - 2*b^2*a - 2*c*b*a^2"
        );
        let mut p_a = step4.poly1.multiply(&step4.p_factor_1a);
        let p2_f2a = step4.poly2.multiply(&step4.p_factor_2a);
        p_a.add_poly_scaled(&p2_f2a, 1);
        let mut p_b = step4.poly1.multiply(&step4.p_factor_1b);
        let p2_f2b = step4.poly2.multiply(&step4.p_factor_2b);
        p_b.add_poly_scaled(&p2_f2b, 1);
        assert_eq!(
            format!("{} {}", p_a, p_b),
            "-2*b + 2*c*b^2 + 2*b*a 4*b - 4*b^3 - 4*b*a^2"
        );
    }

    #[test]
    fn test_poly_new() {
        // Test single-variable term with minus sign
        let poly = Poly::new("-a").unwrap();
        assert_eq!(format!("{}", poly), "-a");

        // Test negative coefficients
        let poly = Poly::new("2*a*-2*b").unwrap();
        assert_eq!(format!("{}", poly), "-4*b*a");

        // Test multiple terms of the same kind
        let poly = Poly::new("a*b + b*a").unwrap();
        assert_eq!(format!("{}", poly), "2*b*a");

        // Test multiple constant factors in a single term
        let poly = Poly::new("a*b + a*b*-1").unwrap();
        assert_eq!(format!("{}", poly), "0");

        // Test with a negative first factor
        let poly = Poly::new("a*b + -1*a*b").unwrap();
        assert_eq!(format!("{}", poly), "0");

        // Verify cleanup was called by checking the string representation
        // If cleanup wasn't called, we might see terms like "0*a*b" or "1*a*b"
        let poly = Poly::new("0*a*b + 1*a*b + 2*a*b").unwrap();
        assert_eq!(format!("{}", poly), "3*b*a");
    }

    #[test]
    fn test_observe_coefficients() {
        // Test case 1: Simple constant polynomial
        let poly = Poly::new("5").unwrap();
        let mut observed_coeffs = Vec::new();
        poly.observe_coefficients(|x| observed_coeffs.push(x));
        assert_eq!(observed_coeffs, vec![5]);

        // Test case 2: Simple polynomial with multiple terms
        let poly = Poly::new("2*a + 3*b + 4*c").unwrap();
        let mut observed_coeffs = Vec::new();
        poly.observe_coefficients(|x| observed_coeffs.push(x));
        assert_eq!(observed_coeffs, vec![0, 4, 3, 2]);

        // Test case 3: Complex polynomial with nested structure
        let poly = Poly::new("2*a*b + 3*a^2 + 4*b^2").unwrap();
        let mut observed_coeffs = Vec::new();
        poly.observe_coefficients(|x| observed_coeffs.push(x));
        assert_eq!(observed_coeffs, vec![0, 0, 4, 0, 2, 3]);

        // Test case 4: Polynomial with negative coefficients
        let poly = Poly::new("2*a - 3*b + 4*c").unwrap();
        let mut observed_coeffs = Vec::new();
        poly.observe_coefficients(|x| observed_coeffs.push(x));
        assert_eq!(observed_coeffs, vec![0, 4, -3, 2]);

        // Test case 5: Polynomial with zero coefficients
        let poly = Poly::new("2*a + 0*b + 4*c").unwrap();
        let mut observed_coeffs = Vec::new();
        poly.observe_coefficients(|x| observed_coeffs.push(x));
        assert_eq!(observed_coeffs, vec![0, 4, 2]);

        // Test case 6: Constant polynomial
        let poly = Poly::new("42").unwrap();
        let mut observed_coeffs = Vec::new();
        poly.observe_coefficients(|x| observed_coeffs.push(x));
        assert_eq!(observed_coeffs, vec![42]);

        // Test case 7: Zero polynomial
        let poly = Poly::new("0").unwrap();
        let mut observed_coeffs = Vec::new();
        poly.observe_coefficients(|x| observed_coeffs.push(x));
        assert_eq!(observed_coeffs, vec![0]);

        // Test case 8: Verify polynomial is not mutated
        let poly = Poly::new("2*a + 3*b + 4*c").unwrap();
        let original = format!("{}", poly);
        poly.observe_coefficients(|_| {}); // Do nothing
        assert_eq!(format!("{}", poly), original);
    }

    #[test]
    fn test_observe_coefficients_find_largest() {
        // Test case 1: Find largest coefficient using observe_coefficients
        let poly = Poly::new("2*a + 15*b + 7*c").unwrap();
        let mut max_coeff = i64::MIN;
        poly.observe_coefficients(|x| {
            max_coeff = max_coeff.max(x);
        });
        assert_eq!(max_coeff, 15);

        // Test case 2: Find largest absolute coefficient
        let poly = Poly::new("2*a - 15*b + 7*c").unwrap();
        let mut max_abs_coeff = 0;
        poly.observe_coefficients(|x| {
            max_abs_coeff = max_abs_coeff.max(x.abs());
        });
        assert_eq!(max_abs_coeff, 15);

        // Test case 3: Complex polynomial with large coefficients
        let poly = Poly::new("100*a*b + 250*a^2 + 75*b^2").unwrap();
        let mut max_coeff = i64::MIN;
        poly.observe_coefficients(|x| {
            max_coeff = max_coeff.max(x);
        });
        assert_eq!(max_coeff, 250);

        // Test case 4: Polynomial with negative coefficients
        let poly = Poly::new("-5*a + 3*b - 8*c").unwrap();
        let mut max_coeff = i64::MIN;
        poly.observe_coefficients(|x| {
            max_coeff = max_coeff.max(x);
        });
        assert_eq!(max_coeff, 3);

        // Test case 5: Constant polynomial
        let poly = Poly::new("42").unwrap();
        let mut max_coeff = i64::MIN;
        poly.observe_coefficients(|x| {
            max_coeff = max_coeff.max(x);
        });
        assert_eq!(max_coeff, 42);

        // Test case 6: Zero polynomial
        let poly = Poly::new("0").unwrap();
        let mut max_coeff = i64::MIN;
        poly.observe_coefficients(|x| {
            max_coeff = max_coeff.max(x);
        });
        assert_eq!(max_coeff, 0);

        // Test case 7: All negative coefficients
        let poly = Poly::new("-10*a - 5*b - 20*c").unwrap();
        let mut max_coeff = i64::MIN;
        poly.observe_coefficients(|x| {
            max_coeff = max_coeff.max(x);
        });
        assert_eq!(max_coeff, 0);

        // Test case 8: Mixed positive, negative, and zero
        let poly = Poly::new("100*a - 50*b + 0*c - 75*d").unwrap();
        let mut max_coeff = i64::MIN;
        poly.observe_coefficients(|x| {
            max_coeff = max_coeff.max(x);
        });
        assert_eq!(max_coeff, 100);
    }

    #[test]
    fn test_observe_coefficients_statistics() {
        // Test case 1: Count coefficients
        let poly = Poly::new("2*a + 3*b + 4*c + 5*d").unwrap();
        let mut count = 0;
        poly.observe_coefficients(|_| count += 1);
        assert_eq!(count, 5);

        // Test case 2: Sum of coefficients
        let poly = Poly::new("2*a + 3*b + 4*c + 5*d").unwrap();
        let mut sum = 0;
        poly.observe_coefficients(|x| sum += x);
        assert_eq!(sum, 14);

        // Test case 3: Count positive coefficients
        let poly = Poly::new("2*a - 3*b + 4*c - 5*d").unwrap();
        let mut positive_count = 0;
        poly.observe_coefficients(|x| {
            if x > 0 {
                positive_count += 1;
            }
        });
        assert_eq!(positive_count, 2);

        // Test case 4: Find minimum coefficient
        let poly = Poly::new("2*a - 3*b + 4*c - 5*d").unwrap();
        let mut min_coeff = i64::MAX;
        poly.observe_coefficients(|x| {
            min_coeff = min_coeff.min(x);
        });
        assert_eq!(min_coeff, -5);
    }

    #[test]
    fn test_apply_to_coefficients() {
        // Test case 1: Simple constant polynomial - increment by 1
        let mut poly = Poly::new("5").unwrap();
        poly.apply_to_coefficients(|x| x + 1);
        assert_eq!(format!("{}", poly), "6");

        // Test case 2: Simple polynomial with multiple terms - increment by 1
        let mut poly = Poly::new("2*a + 3*b + 4*c").unwrap();
        poly.apply_to_coefficients(|x| x + 1);
        assert_eq!(format!("{}", poly), "1 + 5*c + 4*b + 3*a");

        // Test case 3: Complex polynomial with nested structure - increment by 1
        let mut poly = Poly::new("2*a*b + 3*a^2 + 4*b^2").unwrap();
        poly.apply_to_coefficients(|x| x + 1);
        assert_eq!(format!("{}", poly), "1 + b + 5*b^2 + a + 3*b*a + 4*a^2");

        // Test case 4: Polynomial with negative coefficients - make absolute
        let mut poly = Poly::new("2*a - 3*b + 4*c").unwrap();
        poly.apply_to_coefficients(|x| x.abs());
        assert_eq!(format!("{}", poly), "4*c + 3*b + 2*a");

        // Test case 5: Identity function (should not change anything)
        let mut poly = Poly::new("2*a + 3*b + 4*c").unwrap();
        let original = format!("{}", poly);
        poly.apply_to_coefficients(|x| x);
        assert_eq!(format!("{}", poly), original);

        // Test case 6: Function that makes everything zero
        let mut poly = Poly::new("2*a + 3*b + 4*c").unwrap();
        poly.apply_to_coefficients(|_| 0);
        assert_eq!(format!("{}", poly), "0");

        // Test case 7: Function that negates coefficients
        let mut poly = Poly::new("2*a + 3*b + 4*c").unwrap();
        poly.apply_to_coefficients(|x| -x);
        assert_eq!(format!("{}", poly), "-4*c - 3*b - 2*a");

        // Test case 8: Function that doubles coefficients
        let mut poly = Poly::new("2*a + 3*b + 4*c").unwrap();
        poly.apply_to_coefficients(|x| x * 2);
        assert_eq!(format!("{}", poly), "8*c + 6*b + 4*a");

        // Test case 9: Function that replaces negative coefficients with their absolute value
        let mut poly = Poly::new("2*a - 3*b + 4*c - 5*d").unwrap();
        poly.apply_to_coefficients(|x| if x < 0 { -x } else { x });
        assert_eq!(format!("{}", poly), "5*d + 4*c + 3*b + 2*a");

        // Test case 10: Zero polynomial with increment
        let mut poly = Poly::new("0").unwrap();
        poly.apply_to_coefficients(|x| x + 1);
        assert_eq!(format!("{}", poly), "1");
    }

    #[test]
    fn test_reduce_coefficients_if_large() {
        // Test case 1: Coefficients below threshold (should not change)
        let mut poly = Poly::new("100*a + 200*b + 300*c").unwrap();
        let original = format!("{}", poly);
        poly.reduce_coefficients_if_large();
        assert_eq!(format!("{}", poly), original);

        // Test case 2: Coefficients above threshold with common factor
        let mut poly = Poly::new("20000*a + 30000*b + 40000*c").unwrap();
        poly.reduce_coefficients_if_large();
        assert_eq!(format!("{}", poly), "4*c + 3*b + 2*a");

        // Test case 3: Coefficients above threshold but no common factor (GCD = 1)
        let mut poly = Poly::new("10001*a + 10003*b").unwrap();
        let original = format!("{}", poly);
        poly.reduce_coefficients_if_large();
        assert_eq!(format!("{}", poly), original);

        // Test case 4: Mixed positive and negative coefficients
        let mut poly = Poly::new("20000*a - 30000*b + 40000*c").unwrap();
        poly.reduce_coefficients_if_large();
        assert_eq!(format!("{}", poly), "4*c - 3*b + 2*a");

        // Test case 5: Zero coefficients should be ignored in GCD calculation
        let mut poly = Poly::new("20000*a + 0*b + 40000*c").unwrap();
        poly.reduce_coefficients_if_large();
        assert_eq!(format!("{}", poly), "2*c + a");

        // Test case 6: Complex polynomial with nested structure
        let mut poly = Poly::new("20000*a*b + 30000*a^2 + 40000*b^2").unwrap();
        poly.reduce_coefficients_if_large();
        assert_eq!(format!("{}", poly), "4*b^2 + 2*b*a + 3*a^2");

        // Test case 7: Constant polynomial
        let mut poly = Poly::new("20000").unwrap();
        poly.reduce_coefficients_if_large();
        assert_eq!(format!("{}", poly), "1");

        // Test case 8: Empty polynomial (should not panic)
        let mut poly = Poly::new("0").unwrap();
        poly.reduce_coefficients_if_large();
        assert_eq!(format!("{}", poly), "0");

        // Test case 9: All coefficients are the same large number
        let mut poly = Poly::new("50000*a + 50000*b + 50000*c").unwrap();
        poly.reduce_coefficients_if_large();
        assert_eq!(format!("{}", poly), "c + b + a");

        // Test case 10: Mixed coefficients with some below threshold
        let mut poly = Poly::new("100*a + 20000*b + 300*c").unwrap();
        poly.reduce_coefficients_if_large();
        // Should not change because the largest coefficient (20000) is exactly at threshold
        assert_eq!(format!("{}", poly), "3*c + 200*b + a");
    }

    #[test]
    fn test_retain_relevant_polys() {
        // Test case 1: polys "0", "x + y", "x - y" (should remain just "x + y" and "x - y")
        let polys = vec![
            Poly::new("0").unwrap(),
            Poly::new("x + y").unwrap(),
            Poly::new("x - y").unwrap(),
        ];
        let result = Poly::retain_relevant_polys(polys, 23, 24); // x=23, y=24
        assert_eq!(result.len(), 2);
        assert_eq!(format!("{}", result[0]), "y + x");
        assert_eq!(format!("{}", result[1]), "-y + x");

        // Test case 2: polys "a + a^2", "x - a", "y - a" (all polys should remain)
        let polys = vec![
            Poly::new("a + a^2").unwrap(),
            Poly::new("x - a").unwrap(),
            Poly::new("y - a").unwrap(),
        ];
        let result = Poly::retain_relevant_polys(polys, 23, 24); // x=23, y=24
        assert_eq!(result.len(), 3);
        assert_eq!(format!("{}", result[0]), "a + a^2");
        assert_eq!(format!("{}", result[1]), "x - a");
        assert_eq!(format!("{}", result[2]), "y - a");

        // Test case 3: polys "a^2 + b^2 - 1", "x - a", "y - x", "b - 1" (all polys should remain)
        let polys = vec![
            Poly::new("-1 + b^2 + a^2").unwrap(),
            Poly::new("x - a").unwrap(),
            Poly::new("y - x").unwrap(),
            Poly::new("-1 + b").unwrap(),
        ];
        let result = Poly::retain_relevant_polys(polys, 23, 24); // x=23, y=24
        assert_eq!(result.len(), 4);
        assert_eq!(format!("{}", result[0]), "-1 + b^2 + a^2");
        assert_eq!(format!("{}", result[1]), "x - a");
        assert_eq!(format!("{}", result[2]), "y - x");
        assert_eq!(format!("{}", result[3]), "-1 + b");

        // Test case 4: polys "a + b + c", "x + y", "x - y" (only "x + y" and "x - y" should remain)
        let polys = vec![
            Poly::new("a + b + c").unwrap(),
            Poly::new("x + y").unwrap(),
            Poly::new("x - y").unwrap(),
        ];
        let result = Poly::retain_relevant_polys(polys, 23, 24); // x=23, y=24
        assert_eq!(result.len(), 2);
        assert_eq!(format!("{}", result[0]), "y + x");
        assert_eq!(format!("{}", result[1]), "-y + x");
    }

    #[test]
    fn test_retain_relevant_polys_edge_cases() {
        // Test case 1: Empty input
        let polys: Vec<Poly> = vec![];
        let result = Poly::retain_relevant_polys(polys, 0, 1);
        assert_eq!(result.len(), 0);

        // Test case 2: Single polynomial with x and y
        let polys = vec![Poly::new("x + y").unwrap()];
        let result = Poly::retain_relevant_polys(polys, 23, 24);
        assert_eq!(result.len(), 1);
        assert_eq!(format!("{}", result[0]), "y + x");

        // Test case 3: Single polynomial without x or y
        let polys = vec![Poly::new("a + b").unwrap()];
        let result = Poly::retain_relevant_polys(polys, 23, 24);
        assert_eq!(result.len(), 0);

        // Test case 4: Multiple polynomials, none with x or y
        let polys = vec![
            Poly::new("a + b").unwrap(),
            Poly::new("c + d").unwrap(),
            Poly::new("e + f").unwrap(),
        ];
        let result = Poly::retain_relevant_polys(polys, 23, 24);
        assert_eq!(result.len(), 0);

        // Test case 5: Polynomials with only x (no y)
        let polys = vec![Poly::new("x + a").unwrap(), Poly::new("b + c").unwrap()];
        let result = Poly::retain_relevant_polys(polys, 23, 24);
        assert_eq!(result.len(), 1);
        assert_eq!(format!("{}", result[0]), "x + a");

        // Test case 6: Polynomials with only y (no x)
        let polys = vec![Poly::new("y + a").unwrap(), Poly::new("b + c").unwrap()];
        let result = Poly::retain_relevant_polys(polys, 23, 24);
        assert_eq!(result.len(), 1);
        assert_eq!(format!("{}", result[0]), "y + a");

        // Test case 7: Complex chain of dependencies
        let polys = vec![
            Poly::new("a + b").unwrap(),
            Poly::new("x - a").unwrap(),
            Poly::new("c + d").unwrap(),
            Poly::new("y - c").unwrap(),
            Poly::new("e + f").unwrap(),
        ];
        let result = Poly::retain_relevant_polys(polys, 23, 24);
        assert_eq!(result.len(), 4);
        assert_eq!(format!("{}", result[0]), "b + a");
        assert_eq!(format!("{}", result[1]), "x - a");
        assert_eq!(format!("{}", result[2]), "d + c");
        assert_eq!(format!("{}", result[3]), "y - c");
    }

    #[test]
    fn test_substitute_modular_polys_single_variable() {
        use crate::modular_poly::ModularPoly;
        use std::collections::HashMap;

        // Test case 1: Simple substitution a -> x^2 + 3x (mod 7)
        let poly = Poly::new("a^2 + a").unwrap(); // a^2 + a
        let mut var_polys = HashMap::new();
        var_polys.insert(0, ModularPoly::new(vec![0, 3, 1], 7)); // x^2 + 3x

        let result = poly.substitute_modular_polys(&var_polys);
        // (x^2 + 3x)^2 + (x^2 + 3x) = (x^4 + 6x^3 + 9x^2) + (x^2 + 3x) = x^4 + 6x^3 + 10x^2 + 3x
        // In Z/7Z: x^4 + 6x^3 + 3x^2 + 3x
        assert_eq!(result.coeffs, vec![0, 3, 3, 6, 1]);
        assert_eq!(result.p, 7);
    }

    #[test]
    fn test_substitute_modular_polys_two_variables() {
        use crate::modular_poly::ModularPoly;
        use std::collections::HashMap;

        // Test case: a*b + 2*b + a^2 with a -> x^2 + 3x, b -> 5x + 3 (mod 7)
        let poly = Poly::new("a*b + 2*b + a^2").unwrap();
        let mut var_polys = HashMap::new();
        var_polys.insert(0, ModularPoly::new(vec![0, 3, 1], 7)); // a -> x^2 + 3x
        var_polys.insert(1, ModularPoly::new(vec![3, 5], 7)); // b -> 5x + 3

        let result = poly.substitute_modular_polys(&var_polys);
        // Manual calculation:
        // a^2 = (x^2 + 3x)^2 = x^4 + 6x^3 + 9x^2 = x^4 + 6x^3 + 2x^2 (mod 7)
        // 2*b = 2*(5x + 3) = 10x + 6 = 3x + 6 (mod 7)
        // a*b = (x^2 + 3x)*(5x + 3) = 5x^3 + 3x^2 + 15x^2 + 9x = 5x^3 + 18x^2 + 9x = 5x^3 + 4x^2 + 2x (mod 7)
        // Total: x^4 + 6x^3 + 2x^2 + 3x + 6 + 5x^3 + 4x^2 + 2x = x^4 + 11x^3 + 6x^2 + 5x + 6
        // = x^4 + 4x^3 + 6x^2 + 5x + 6 (mod 7)
        assert_eq!(result.coeffs, vec![6, 5, 6, 4, 1]);
        assert_eq!(result.p, 7);
    }

    #[test]
    fn test_substitute_modular_polys_constant() {
        use crate::modular_poly::ModularPoly;
        use std::collections::HashMap;

        // Test case: Constant polynomial
        let poly = Poly::new("5").unwrap();
        let mut var_polys = HashMap::new();
        var_polys.insert(0, ModularPoly::new(vec![0, 1], 7)); // x

        let result = poly.substitute_modular_polys(&var_polys);
        assert_eq!(result.coeffs, vec![5]);
        assert_eq!(result.p, 7);
    }

    #[test]
    fn test_substitute_modular_polys_complex_nested() {
        use crate::modular_poly::ModularPoly;
        use std::collections::HashMap;

        // Test case: Complex nested polynomial a^2*b + b^2 with substitutions
        let poly = Poly::new("a^2*b + b^2").unwrap();
        let mut var_polys = HashMap::new();
        var_polys.insert(0, ModularPoly::new(vec![1, 2], 7)); // a -> 2x + 1
        var_polys.insert(1, ModularPoly::new(vec![0, 1], 7)); // b -> x

        let result = poly.substitute_modular_polys(&var_polys);
        // Manual calculation:
        // a^2 = (2x + 1)^2 = 4x^2 + 4x + 1
        // a^2*b = (4x^2 + 4x + 1)*x = 4x^3 + 4x^2 + x
        // b^2 = x^2
        // Total: 4x^3 + 4x^2 + x + x^2 = 4x^3 + 5x^2 + x = 4x^3 + 5x^2 + x (mod 7)
        assert_eq!(result.coeffs, vec![0, 1, 5, 4]);
        assert_eq!(result.p, 7);
    }

    #[test]
    fn test_substitute_modular_polys_linear_polynomials() {
        use crate::modular_poly::ModularPoly;
        use std::collections::HashMap;

        // Test case: Linear polynomials that sum to constant
        // (p-10)x + 1 + 10x + 2 = px + 3 = 3 (mod p)
        let poly = Poly::new("a + b").unwrap();
        let mut var_polys = HashMap::new();
        var_polys.insert(0, ModularPoly::new(vec![1, 3], 7)); // a -> 3x + 1
        var_polys.insert(1, ModularPoly::new(vec![2, 4], 7)); // b -> 4x + 2

        let result = poly.substitute_modular_polys(&var_polys);
        // (3x + 1) + (4x + 2) = 7x + 3 = 3 (mod 7)
        assert_eq!(result.coeffs, vec![3]);
        assert_eq!(result.p, 7);
    }

    #[test]
    fn test_substitute_modular_polys_zero_polynomial() {
        use crate::modular_poly::ModularPoly;
        use std::collections::HashMap;

        // Test case: Zero polynomial
        let poly = Poly::new("0").unwrap();
        let mut var_polys = HashMap::new();
        var_polys.insert(0, ModularPoly::new(vec![1, 2], 7));

        let result = poly.substitute_modular_polys(&var_polys);
        assert_eq!(result.coeffs, vec![0]);
        assert_eq!(result.p, 7);
    }

    #[test]
    fn test_substitute_modular_polys_high_degree() {
        use crate::modular_poly::ModularPoly;
        use std::collections::HashMap;

        // Test case: High degree polynomial
        let poly = Poly::new("a^3 + a^2 + a + 1").unwrap();
        let mut var_polys = HashMap::new();
        var_polys.insert(0, ModularPoly::new(vec![0, 1], 7)); // a -> x

        let result = poly.substitute_modular_polys(&var_polys);
        // x^3 + x^2 + x + 1
        assert_eq!(result.coeffs, vec![1, 1, 1, 1]);
        assert_eq!(result.p, 7);
    }
}
