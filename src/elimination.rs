use crate::modular_poly::ModularPoly;
use crate::poly::{Poly, PolyOperations, VarSearchResult};
use gcd::Gcd;
use log::info;
use rand::Rng;
use std::{collections::HashMap, fmt, mem, rc::Rc};

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
        info!("\n(A) {}\n(B) {}", self.poly_a, self.poly_b);
        let (pa1, pa2) = self
            .poly_a
            .extract_factor_and_remainder(self.v, self.degree_b);
        let (pb1, pb2) = self
            .poly_b
            .extract_factor_and_remainder(self.v, self.degree_b);

        // Compute new poly_b = pb1 * pa2 - pb2 * pa1

        let mut new_poly_b = pa2.multiply(&pb1);
        let temp = pa1.multiply(&pb2);
        new_poly_b.add_poly_scaled(&temp, -1);
        new_poly_b.reduce_coefficients_if_large();

        // Compute new factors
        let mut p_factor_1b = self.p_factor_1a.multiply(&pb1);
        let temp = self.p_factor_1b.multiply(&pa1);
        p_factor_1b.add_poly_scaled(&temp, -1);

        let mut p_factor_2b = self.p_factor_2a.multiply(&pb1);
        let temp = self.p_factor_2b.multiply(&pa1);
        p_factor_2b.add_poly_scaled(&temp, -1);

        info!("(B'){}", new_poly_b);
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

    /// Express the variable as a modular polynomial based on the current var_replacements and q
    /// This method is left unimplemented for now
    pub fn express_var_as_modular_poly(
        &self,
        var_replacements: &HashMap<u8, (ModularPoly, u8)>,
        q: &ModularPoly,
    ) -> Result<(ModularPoly, u8), String> {
        let degree = self.poly_a.get_degree(self.v);
        info!(
            "Finding {} from {}",
            Poly::var_to_string(self.v),
            self.poly_a
        );
        let (factor, remainder) = self.poly_a.extract_factor_and_remainder(self.v, degree);
        if factor.has_var(self.v) {
            return Err(format!(
                "{}^{} in {} has the factor {} ({:?})",
                Poly::var_to_string(self.v),
                degree,
                self.poly_a,
                factor,
                factor,
            ));
        }
        if remainder.has_var(self.v) {
            return Err(format!(
                "{}^{} in {} has the remainder {}",
                Poly::var_to_string(self.v),
                degree,
                self.poly_a,
                remainder,
            ));
        }
        info!("Substituting factor {} and remainder {}", factor, remainder);
        let modular_factor = factor.substitute_modular_polys(var_replacements)?;
        let modular_factor = modular_factor.remainder(q);
        let modular_remainder = remainder.substitute_modular_polys(var_replacements)?;
        let modular_remainder = modular_remainder.remainder(q);
        if modular_factor.is_zero() && modular_remainder.is_zero() {
            info!(
                "Using a random polynomial for {} because it turns out to be 0/0",
                Poly::var_to_string(self.v)
            );
            return Ok((ModularPoly::random(1, q.p), 1));
        }
        let inv = modular_factor
            .get_inverse(q)
            .ok_or(format!("{} has no inverse modulo {}", modular_factor, q))?;
        let product = (&modular_remainder * &inv).remainder(q);
        let result = &ModularPoly::new(vec![0], q.p) - &product;
        info!("{}^{} = {}", Poly::var_to_string(self.v), degree, result);
        Ok((result, degree as u8))
    }
}

pub struct Elimination<'a> {
    pub initial_polys: &'a Vec<Rc<Poly>>,
    pub polys: Vec<Rc<Poly>>,
    pub resolved_steps: Vec<EliminationStep>,
    pub x_var: u8,
    pub y_var: u8,
}

impl<'a> Elimination<'a> {
    pub fn new(initial_polys: &'a Vec<Rc<Poly>>, x_var: u8, y_var: u8) -> Self {
        let polys = initial_polys.clone();
        Self {
            initial_polys,
            polys,
            resolved_steps: Vec::new(),
            x_var,
            y_var,
        }
    }

    pub fn get_var_to_eliminate(&self) -> Option<VarSearchResult> {
        Poly::get_min_degree_var(&self.polys, self.x_var, self.y_var)
    }

    pub fn eliminate_var(&mut self, var_search_result: VarSearchResult) {
        let mut new_polys = Vec::new();
        let mut final_step = None;
        let mut poly_with_var = self.polys[var_search_result.poly_index].clone();
        for (i, poly) in self.polys.iter().enumerate() {
            if i == var_search_result.poly_index {
                continue;
            }
            if !poly.has_var(var_search_result.var) {
                new_polys.push(poly.clone());
                continue;
            }

            let mut elimination_step =
                EliminationStep::new(var_search_result.var, poly.clone(), poly_with_var.clone());
            while let Some(next_step) = elimination_step.get_next_step() {
                elimination_step = next_step;
            }
            if *elimination_step.poly_b != Poly::Constant(0) {
                new_polys.push(elimination_step.poly_b.clone());
            }
            poly_with_var = elimination_step.poly_a.clone();
            final_step = Some(elimination_step);
        }
        self.resolved_steps.push(final_step.unwrap());
        self.polys = new_polys;
    }

    pub fn check_factor(&self, factor: &Poly) -> Result<bool, String> {
        // Choose modulus p as one of the specified large random numbers
        let modulus_options = [
            u64::MAX - 58,
            u64::MAX - 82,
            u64::MAX - 94,
            u64::MAX - 178,
            u64::MAX - 188,
        ];
        let p = modulus_options[rand::rng().random_range(0..5)];

        // Generate random polynomials x(t) and y(t) with degree 1
        let mut x_poly: ModularPoly;
        let mut y_poly: ModularPoly;

        // Keep trying until we get non-proportional polynomials
        loop {
            x_poly = ModularPoly::random(1, p);
            y_poly = ModularPoly::random(1, p);

            // Check if the polynomials are not proportional (determinant is non-zero)
            let ax = x_poly.coeffs[0];
            let bx = x_poly.coeffs[1];
            let ay = y_poly.coeffs[0];
            let by = y_poly.coeffs[1];

            if ax * by != ay * bx {
                break;
            }
        }
        info!("{} = {}", Poly::var_to_string(self.x_var), x_poly);
        info!("{} = {}", Poly::var_to_string(self.y_var), y_poly);

        // Initialize var_replacements with x_var and y_var
        let mut var_replacements: HashMap<u8, (ModularPoly, u8)> = HashMap::new();
        var_replacements.insert(self.x_var, (x_poly, 1));
        var_replacements.insert(self.y_var, (y_poly, 1));

        // Substitute x(t) and y(t) into the factor polynomial
        let q = factor.substitute_modular_polys(&var_replacements)?;
        info!("q: {}", q);
        if q.is_zero() {
            info!("q is zero - test is inconclusive! Returning false");
            return Ok(false);
        }
        let x_poly = var_replacements.get(&self.x_var).unwrap().0.remainder(&q);
        let y_poly = var_replacements.get(&self.y_var).unwrap().0.remainder(&q);
        info!("{} = {}", Poly::var_to_string(self.x_var), x_poly);
        info!("{} = {}", Poly::var_to_string(self.y_var), y_poly);
        var_replacements.insert(self.x_var, (x_poly, 1));
        var_replacements.insert(self.y_var, (y_poly, 1));

        // Iterate over resolved_steps in reversed order
        for step in self.resolved_steps.iter().rev() {
            let (var_poly, var_degree) = step.express_var_as_modular_poly(&var_replacements, &q)?;
            var_replacements.insert(step.v, (var_poly, var_degree));
        }

        // Verify that equations hold
        Ok(self.verify_equations_hold(&var_replacements, &q))
    }

    /// Verify that the equations hold with the given variable replacements and q
    /// For each polynomial from self.initial_polys, substitute the variables with modular polynomials (mod q)
    /// and verify that the result is always 0.
    fn verify_equations_hold(
        &self,
        var_replacements: &HashMap<u8, (ModularPoly, u8)>,
        q: &ModularPoly,
    ) -> bool {
        for poly in self.initial_polys {
            // Substitute variables with modular polynomials
            match poly.substitute_modular_polys(var_replacements) {
                Ok(substituted_poly) => {
                    // Take the remainder modulo q
                    let remainder = substituted_poly.remainder(q);
                    // Check if the remainder is zero
                    if !remainder.is_zero() {
                        info!(
                            "Equation {} = {} (mod {}) is not zero, remainder: {}",
                            poly, substituted_poly, q, remainder
                        );
                        return false;
                    }
                }
                Err(e) => {
                    info!("Error substituting variables in {}: {}", poly, e);
                    return false;
                }
            }
        }

        // All equations hold
        true
    }
}

mod tests {
    use super::*;
    #[cfg(test)]
    use test_log::test;

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
    fn test_check_factor() {
        // Create initial polynomials
        let poly1 = Poly::new("a + a*c^2 - 1 + c^2").unwrap();
        let poly2 = Poly::new("b + b*c^2 - 2*c").unwrap();
        let initial_polys = vec![Rc::new(poly1), Rc::new(poly2)];

        // Create Elimination with x_var = 0 (a), y_var = 1 (b)
        let mut elimination = Elimination::new(&initial_polys, 0, 1);

        // Get the variable to eliminate (should be var = 2 (c))
        let var_search_result = elimination.get_var_to_eliminate().unwrap();
        assert_eq!(var_search_result.var, 2); // c
        assert_eq!(var_search_result.min_degree, 2);
        assert_eq!(var_search_result.poly_index, 0);

        // Eliminate variable c
        elimination.eliminate_var(var_search_result);

        // Test check_factor for polynomial "b" - should return false
        let wrong_factor = Poly::new("a + 1").unwrap();
        assert_eq!(elimination.check_factor(&wrong_factor).unwrap(), false);

        // Test check_factor for polynomial "a^2 + b^2 - 1" - should return true
        let correct_factor = Poly::new("a^2 + b^2 - 1").unwrap();
        assert_eq!(elimination.check_factor(&correct_factor).unwrap(), true);
    }
}
