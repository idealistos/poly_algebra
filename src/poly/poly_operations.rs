use std::rc::Rc;
use std::time::Duration;

use crate::poly::{Poly, PolyConversion};
use log::info;

#[derive(Debug, Clone)]
pub enum SingleOutResult {
    Constant,
    Linear(Rc<Poly>, i64),
    Nonlinear,
}

pub trait PolyOperations {
    fn scale(&mut self, factor: i64);
    fn add_poly_scaled(&mut self, poly: &Poly, factor: i64);
    fn multiply(&self, poly: &Poly) -> Poly;
    fn extract_factor_and_remainder(&self, v: u8, degree: u32) -> (Poly, Poly);
    fn single_out(&self, v: u8) -> SingleOutResult;
    fn substitute_linear(&self, v: u8, poly: Rc<Poly>, k: i64) -> Poly;
    fn get_derivative(&self, v: u8) -> Poly;
    fn factor(&self) -> Result<Vec<Poly>, String>;
}

impl PolyOperations for Poly {
    fn scale(&mut self, factor: i64) {
        match self {
            Poly::Constant(n) => *n *= factor,
            Poly::Nested(_, polys) => {
                for poly in polys.iter_mut() {
                    let poly_mut = Rc::make_mut(poly);
                    poly_mut.scale(factor);
                }
            }
        }
    }

    fn add_poly_scaled(&mut self, poly: &Poly, factor: i64) {
        match (&mut *self, poly) {
            // Both are constants
            (Poly::Constant(n1), Poly::Constant(n2)) => {
                *n1 += factor * n2;
            }
            // Self is nested with variable v, poly is constant or has higher variable
            (Poly::Nested(_, polys), Poly::Constant(_)) => {
                if let Some(first) = polys.first_mut() {
                    let first_mut = Rc::make_mut(first);
                    first_mut.add_poly_scaled(poly, factor);
                }
            }
            (Poly::Nested(v, polys), Poly::Nested(v1, _)) if *v1 > *v => {
                if let Some(first) = polys.first_mut() {
                    let first_mut = Rc::make_mut(first);
                    first_mut.add_poly_scaled(poly, factor);
                }
            }
            // Poly is nested with variable v1, self is constant or has higher variable
            (Poly::Constant(_), Poly::Nested(_, _)) => {
                let mut scaled_poly = poly.clone();
                scaled_poly.scale(factor);
                scaled_poly.add_poly_scaled(self, 1);
                *self = scaled_poly;
            }
            (Poly::Nested(v, _), Poly::Nested(v1, _)) if *v > *v1 => {
                let mut scaled_poly = poly.clone();
                scaled_poly.scale(factor);
                scaled_poly.add_poly_scaled(self, 1);
                *self = scaled_poly;
            }
            // Both are nested with the same variable
            (Poly::Nested(v, polys), Poly::Nested(v1, polys1)) if v == v1 => {
                // Pad polys with zeros if needed
                while polys.len() < polys1.len() {
                    polys.push(Rc::new(Poly::Constant(0)));
                }
                // Add corresponding terms
                for (i, p1) in polys1.iter().enumerate() {
                    let poly_mut = Rc::make_mut(&mut polys[i]);
                    poly_mut.add_poly_scaled(p1, factor);
                }
            }
            // Handle remaining cases (should not occur in practice)
            _ => {
                // If we get here, something is wrong with the variable ordering
                panic!("Unexpected variable ordering in add_poly_scaled");
            }
        }
        self.cleanup();
    }

    fn multiply(&self, poly: &Poly) -> Poly {
        match (self, poly) {
            // If either is constant, scale the other
            (Poly::Constant(n), _) => {
                let mut result = poly.clone();
                result.scale(*n);
                result
            }
            (_, Poly::Constant(n)) => {
                let mut result = self.clone();
                result.scale(*n);
                result
            }
            // Both are nested with different variables
            (Poly::Nested(v, polys), Poly::Nested(v1, polys1)) if v != v1 => {
                let v_min = v.min(v1);
                let mut result = if v_min == v {
                    // Multiply each term in polys by poly
                    let mut new_polys = Vec::new();
                    for p in polys {
                        new_polys.push(Rc::new(p.multiply(poly)));
                    }
                    Poly::Nested(*v, new_polys)
                } else {
                    // Multiply each term in polys1 by self
                    let mut new_polys = Vec::new();
                    for p in polys1 {
                        new_polys.push(Rc::new(self.multiply(p)));
                    }
                    Poly::Nested(*v1, new_polys)
                };
                result.cleanup();
                result
            }
            // Both are nested with the same variable
            (Poly::Nested(v, polys), Poly::Nested(v1, polys1)) if v == v1 => {
                let mut result_polys = Vec::new();
                // For each degree in the result
                for i in 0..(polys.len() + polys1.len() - 1) {
                    let mut sum = Poly::Constant(0);
                    // Sum up all products that contribute to this degree
                    for j in 0..=i.min(polys.len() - 1) {
                        if i - j < polys1.len() {
                            let product = polys[j].multiply(&polys1[i - j]);
                            sum.add_poly_scaled(&product, 1);
                        }
                    }
                    result_polys.push(Rc::new(sum));
                }
                let mut result = Poly::Nested(*v, result_polys);
                result.cleanup();
                result
            }
            // This case should never occur
            _ => panic!("Unexpected case in multiply"),
        }
    }

    fn extract_factor_and_remainder(&self, v: u8, degree: u32) -> (Poly, Poly) {
        match self {
            Poly::Constant(_) => (Poly::Constant(0), self.clone()),
            Poly::Nested(v1, _) if *v1 > v => (Poly::Constant(0), self.clone()),
            Poly::Nested(v1, polys1) if *v1 < v => {
                let mut factor_polys = Vec::with_capacity(polys1.len());
                let mut remainder_polys = Vec::with_capacity(polys1.len());
                for p in polys1 {
                    let (f, r) = p.extract_factor_and_remainder(v, degree);
                    factor_polys.push(Rc::new(f));
                    remainder_polys.push(Rc::new(r));
                }
                let mut poly1 = Poly::Nested(*v1, factor_polys);
                let mut poly2 = Poly::Nested(*v1, remainder_polys);
                poly1.cleanup();
                poly2.cleanup();
                (poly1, poly2)
            }
            Poly::Nested(v1, polys1) if *v1 == v => {
                let d = degree as usize;
                let len = polys1.len();
                // Remainder: terms of degree < d
                let remainder =
                    Poly::Nested(*v1, polys1.iter().take(d.min(len)).cloned().collect());
                // Factor: terms of degree >= d, with degree shifted down by d
                let factor = if d >= len {
                    Poly::Constant(0)
                } else {
                    Poly::Nested(*v1, polys1.iter().skip(d).cloned().collect())
                };
                (factor, remainder)
            }
            _ => unreachable!(),
        }
    }

    fn single_out(&self, v: u8) -> SingleOutResult {
        match self {
            Poly::Constant(_) => SingleOutResult::Constant,
            Poly::Nested(v1, polys) => {
                if *v1 > v {
                    SingleOutResult::Constant
                } else if *v1 == v {
                    if polys.len() == 2 {
                        if let Poly::Constant(c) = *polys[1] {
                            SingleOutResult::Linear(polys[0].clone(), -c)
                        } else {
                            SingleOutResult::Nonlinear
                        }
                    } else {
                        SingleOutResult::Nonlinear
                    }
                } else {
                    // v1 < v
                    let mut results = Vec::new();
                    for poly in polys {
                        results.push(poly.single_out(v));
                    }

                    match &results[0] {
                        SingleOutResult::Linear(result_poly, k) => {
                            // Check if all other results are Constant
                            if results[1..]
                                .iter()
                                .all(|r| matches!(r, SingleOutResult::Constant))
                            {
                                let mut new_polys = polys.clone();
                                new_polys[0] = result_poly.clone();
                                SingleOutResult::Linear(Rc::new(Poly::Nested(*v1, new_polys)), *k)
                            } else {
                                SingleOutResult::Nonlinear
                            }
                        }
                        SingleOutResult::Constant => {
                            if results[1..]
                                .iter()
                                .all(|r| matches!(r, SingleOutResult::Constant))
                            {
                                SingleOutResult::Constant
                            } else {
                                SingleOutResult::Nonlinear
                            }
                        }
                        SingleOutResult::Nonlinear => SingleOutResult::Nonlinear,
                    }
                }
            }
        }
    }

    fn substitute_linear(&self, v: u8, poly: Rc<Poly>, k: i64) -> Poly {
        let d = self.get_degree(v);
        let mut factors = vec![Rc::new(Poly::Constant(0)); d as usize + 1];
        self.compute_factors(v, &mut factors);
        let mut factor0 = factors[0].clone();

        let result = Rc::make_mut(&mut factor0);
        result.scale(k.pow(d));
        let mut poly_power = poly.clone();

        for i in 1..=d {
            let product = factors[i as usize].multiply(&poly_power);
            result.add_poly_scaled(&product, k.pow(d - i));
            poly_power = Rc::new(poly_power.multiply(&poly));
        }
        result.cleanup();
        result.reduce_coefficients_if_large();
        result.clone()
    }

    fn get_derivative(&self, v: u8) -> Poly {
        match self {
            Poly::Constant(_) => Poly::Constant(0),
            Poly::Nested(v1, polys) => {
                if *v1 > v {
                    Poly::Constant(0)
                } else if *v1 == v {
                    let mut new_polys = Vec::with_capacity(polys.len() - 1);
                    for i in 0..polys.len() - 1 {
                        let mut poly = polys[i + 1].clone();
                        let poly_mut = Rc::make_mut(&mut poly);
                        poly_mut.scale((i + 1) as i64);
                        new_polys.push(poly);
                    }
                    let mut result = Poly::Nested(*v1, new_polys);
                    result.cleanup();
                    result
                } else {
                    let mut new_polys = Vec::with_capacity(polys.len());
                    for poly in polys {
                        new_polys.push(Rc::new(poly.get_derivative(v)));
                    }
                    let mut result = Poly::Nested(*v1, new_polys);
                    result.cleanup();
                    result
                }
            }
        }
    }

    fn factor(&self) -> Result<Vec<Poly>, String> {
        const PARI_PATH: &str = r"C:\progs\pari\gp.exe";

        // Create the Pari/GP factoring task
        let poly_str = format!("{}", self);
        let pari_task = format!(
            "{{expr = Vec(factor({}));print(expr[1]);print(expr[2]);}}",
            poly_str
        );

        // Execute gp.exe -q from the specified folder
        let mut child = std::process::Command::new(PARI_PATH)
            .arg("-q")
            .arg("-s")
            .arg("128000000")
            .current_dir(r"C:\progs\pari\")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn gp.exe: {}", e))?;

        let child_id = child.id();

        // Write the pari_task to stdin
        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(pari_task.as_bytes())
                .map_err(|e| format!("Failed to write to gp.exe stdin: {}", e))?;
        }

        // Wait for output with timeout
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let result = child.wait_with_output();
            let _ = tx.send(result);
        });

        // Wait for either the result or timeout
        let output = match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Err(format!("Failed to get output from gp.exe: {}", e));
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Timeout occurred - kill the child process
                let _ = std::process::Command::new("taskkill")
                    .args(["/PID", &child_id.to_string(), "/F"])
                    .output();
                info!("Pari/GP call takes too long, returning original polynomial");
                return Ok(vec![self.clone()]);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return Err("Thread communication failed".to_string());
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("gp.exe failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.len() < 2 {
            println!("gp.exe output: {}", stdout);
            return Err(format!(
                "Expected at least 2 lines of output from gp.exe. Output: {}",
                stdout
            ));
        }

        // Parse the first line as "[<poly1>,<poly2>,..<polyN>]~"
        let factors_line = lines[0].trim();
        if !factors_line.starts_with('[') || !factors_line.ends_with("]~") {
            return Err(format!("Invalid factors line format: {}", factors_line));
        }

        let factors_content = &factors_line[1..factors_line.len() - 2]; // Remove "[...]~"
        let factor_strings: Vec<&str> = Self::parse_pari_list(factors_content)?;

        // Parse the second line as "[<degree1>,<degree2>,..,<degreeN>]~"
        let degrees_line = lines[1].trim();
        if !degrees_line.starts_with('[') || !degrees_line.ends_with("]~") {
            return Err(format!("Invalid degrees line format: {}", degrees_line));
        }

        let degrees_content = &degrees_line[1..degrees_line.len() - 2]; // Remove "[...]~"
        let degree_strings: Vec<&str> = Self::parse_pari_list(degrees_content)?;

        if factor_strings.len() != degree_strings.len() {
            return Err(format!(
                "Mismatch between factors ({}) and degrees ({})",
                factor_strings.len(),
                degree_strings.len()
            ));
        }

        // Convert factor strings to Poly objects
        let mut factors: Vec<Poly> = Vec::new();
        for factor_str in factor_strings {
            let poly = Poly::from_poly_expression(factor_str)
                .map_err(|e| format!("Failed to parse factor '{}': {}", factor_str, e))?;
            factors.push(poly);
        }

        // Parse degrees
        let mut degrees: Vec<u32> = Vec::new();
        for degree_str in degree_strings {
            let degree = degree_str
                .parse::<u32>()
                .map_err(|e| format!("Failed to parse degree '{}': {}", degree_str, e))?;
            degrees.push(degree);
        }

        // Reconstruct the polynomial and verify it matches the original
        let mut reconstructed = Poly::Constant(1);
        for (factor, &degree) in factors.iter().zip(degrees.iter()) {
            let mut factor_power = factor.clone();
            for _ in 1..degree {
                factor_power = factor_power.multiply(factor);
            }
            reconstructed = reconstructed.multiply(&factor_power);
        }

        if reconstructed != *self {
            reconstructed.apply_to_coefficients(|x| -x);
            if reconstructed != *self {
                return Err(format!(
                    "Factorization verification failed. Original: {}, Reconstructed: {}",
                    self, reconstructed
                ));
            }
        }

        Ok(factors)
    }
}

impl Poly {
    // self = factors[0] + v * factors[1] + ... + v^d * factors[d]
    fn compute_factors(&self, v: u8, factors: &mut [Rc<Poly>]) {
        match self {
            Poly::Constant(_) => {
                let factor0 = Rc::make_mut(&mut factors[0]);
                factor0.add_poly_scaled(self, 1);
            }
            Poly::Nested(v1, polys) => {
                if *v1 > v {
                    let factor0 = Rc::make_mut(&mut factors[0]);
                    factor0.add_poly_scaled(self, 1);
                } else if *v1 == v {
                    for (i, poly) in polys.iter().enumerate() {
                        if i < factors.len() {
                            let factor = Rc::make_mut(&mut factors[i]);
                            factor.add_poly_scaled(poly, 1);
                        }
                    }
                } else if *v1 < v {
                    let mut inner_factors = Vec::new();
                    let mut d_max = 0;
                    for (i, poly) in polys.iter().enumerate() {
                        let d = poly.get_degree(v);
                        d_max = d_max.max(d);
                        inner_factors.push(vec![Rc::new(Poly::Constant(0)); (d + 1) as usize]);
                        poly.compute_factors(v, &mut inner_factors[i]);
                    }
                    for j in 0..(d_max as usize) + 1 {
                        let inner_polys: Vec<Rc<Poly>> = inner_factors
                            .iter()
                            .map(|ps| {
                                if j >= ps.len() {
                                    Rc::new(Poly::Constant(0))
                                } else {
                                    ps[j].clone()
                                }
                            })
                            .collect();
                        let poly1 = Poly::Nested(*v1, inner_polys);
                        let factor = Rc::make_mut(&mut factors[j]);
                        factor.add_poly_scaled(&poly1, 1);
                    }
                }
            }
        }
    }

    fn parse_pari_list(content: &str) -> Result<Vec<&str>, String> {
        let result: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale() {
        // Test scaling a constant
        let mut p = Poly::new("5").unwrap();
        p.scale(2);
        assert_eq!(p, Poly::new("10").unwrap());

        // Test scaling a simple polynomial
        let mut p = Poly::new("1 + 2*a").unwrap();
        p.scale(3);
        assert_eq!(p, Poly::new("3 + 6*a").unwrap());

        // Test scaling a nested polynomial
        let mut p = Poly::new("1 + 2*b + 3*a").unwrap();
        p.scale(-2);
        assert_eq!(p, Poly::new("-2 - 4*b - 6*a").unwrap());

        // Test scaling with zero
        let mut p = Poly::new("1 + 2*a").unwrap();
        p.scale(0);
        assert_eq!(
            p,
            Poly::Nested(
                0,
                vec![Rc::new(Poly::Constant(0)), Rc::new(Poly::Constant(0)),],
            )
        );

        // Test scaling with cleanup
        let mut p = Poly::new("1 + 2*a + 0*a^2").unwrap();
        p.scale(2);
        p.cleanup();
        assert_eq!(p, Poly::new("2 + 4*a").unwrap());
    }

    #[test]
    fn test_add_poly_scaled_constants() {
        let mut p1 = Poly::new("5").unwrap();
        let p2 = Poly::new("3").unwrap();
        p1.add_poly_scaled(&p2, 2);
        assert_eq!(format!("{}", p1), "11"); // 5 + 2*3 = 11
    }

    #[test]
    fn test_add_poly_scaled_constant_to_poly() {
        let mut p1 = Poly::new("1 + 2*a").unwrap();
        let p2 = Poly::new("3").unwrap();
        p1.add_poly_scaled(&p2, 2);
        assert_eq!(format!("{}", p1), "7 + 2*a"); // 1 + 2*3 + 2*a
    }

    #[test]
    fn test_add_poly_scaled_same_variable() {
        let mut p1 = Poly::new("1 + 2*a").unwrap();
        let p2 = Poly::new("3 + 4*a").unwrap();
        p1.add_poly_scaled(&p2, 2);
        assert_eq!(format!("{}", p1), "7 + 10*a"); // 1 + 2*3 + (2 + 2*4)*a
    }

    #[test]
    fn test_add_poly_scaled_different_variables() {
        let mut p1 = Poly::new("1 + 2*a").unwrap();
        let p2 = Poly::new("3 + 4*b").unwrap();
        p1.add_poly_scaled(&p2, 2);
        assert_eq!(format!("{}", p1), "7 + 8*b + 2*a"); // 1 + 2*3 + 2*a + 2*4*b
    }

    #[test]
    fn test_add_poly_scaled_padding() {
        let mut p1 = Poly::new("1 + 2*a").unwrap();
        let p2 = Poly::new("3 + 4*a + 5*a^2").unwrap();
        p1.add_poly_scaled(&p2, 2);
        assert_eq!(format!("{}", p1), "7 + 10*a + 10*a^2"); // 1 + 2*3 + (2 + 2*4)*a + 2*5*a^2
    }

    #[test]
    fn test_add_poly_scaled_negative() {
        let mut p1 = Poly::new("1 + 2*a").unwrap();
        let p2 = Poly::new("3 + 4*a").unwrap();
        p1.add_poly_scaled(&p2, -2);
        assert_eq!(format!("{}", p1), "-5 - 6*a"); // 1 + (-2)*3 + (2 + (-2)*4)*a
    }

    #[test]
    fn test_add_poly_scaled_subtract_self() {
        let mut p1 = Poly::new("1 + 2*a + 3*b").unwrap();
        let p2 = p1.clone();
        p1.add_poly_scaled(&p2, -1);
        assert_eq!(format!("{}", p1), "0"); // p1 - p1 = 0
    }

    #[test]
    fn test_add_poly_scaled_complex1() {
        let mut p1 = Poly::new("a*b + 2*b^2 + a^2").unwrap();
        let p2 = Poly::new("b + b^3 + b*c^2").unwrap();
        p1.add_poly_scaled(&p2, 2);
        assert_eq!(
            format!("{}", p1),
            "2*b + 2*c^2*b + 2*b^2 + 2*b^3 + b*a + a^2"
        );
    }

    #[test]
    fn test_add_poly_scaled_complex2() {
        let mut p1 = Poly::new("a^2*b^2 + b^2*c + 3*c^2*a").unwrap();
        let p2 = Poly::new("5*a^2*b^2 + b*c^2 - c^2*a").unwrap();
        p1.add_poly_scaled(&p2, 3);
        assert_eq!(format!("{}", p1), "3*c^2*b + c*b^2 + 16*b^2*a^2");
    }

    #[test]
    fn test_add_poly_scaled_complex3() {
        let mut p1 = Poly::new("a^3 + b^3 + c^3").unwrap();
        let p2 = Poly::new("3*a*b*c").unwrap();
        p1.add_poly_scaled(&p2, -1);
        assert_eq!(format!("{}", p1), "c^3 + b^3 - 3*c*b*a + a^3");
    }

    #[test]
    fn test_add_poly_scaled_complex4() {
        let mut p1 = Poly::new("a^2 + b^2 + c^2").unwrap();
        let p2 = Poly::new("2*a*b + 2*b*c + 2*c*a").unwrap();
        p1.add_poly_scaled(&p2, 1);
        assert_eq!(format!("{}", p1), "c^2 + 2*c*b + b^2 + 2*c*a + 2*b*a + a^2");
    }

    #[test]
    fn test_add_poly_scaled_complex5() {
        let mut p1 = Poly::new("a^4 + b^4 + c^4").unwrap();
        let p2 = Poly::new("4*a^3*b + 6*a^2*b^2 + 4*a*b^3").unwrap();
        p1.add_poly_scaled(&p2, 1);
        assert_eq!(
            format!("{}", p1),
            "c^4 + b^4 + 4*b^3*a + 6*b^2*a^2 + 4*b*a^3 + a^4"
        );
    }

    #[test]
    fn test_multiply_constants() {
        let p1 = Poly::new("5").unwrap();
        let p2 = Poly::new("3").unwrap();
        let result = p1.multiply(&p2);
        assert_eq!(format!("{}", result), "15");
    }

    #[test]
    fn test_multiply_constant_poly() {
        let p1 = Poly::new("3").unwrap();
        let p2 = Poly::new("1 + 2*a").unwrap();
        let result = p1.multiply(&p2);
        assert_eq!(format!("{}", result), "3 + 6*a");
    }

    #[test]
    fn test_multiply_same_variable() {
        let p1 = Poly::new("1 + 2*a").unwrap();
        let p2 = Poly::new("3 + 4*a").unwrap();
        let result = p1.multiply(&p2);
        assert_eq!(format!("{}", result), "3 + 10*a + 8*a^2");
    }

    #[test]
    fn test_multiply_different_variables() {
        let p1 = Poly::new("1 + 2*a").unwrap();
        let p2 = Poly::new("3 + 4*b").unwrap();
        let result = p1.multiply(&p2);
        assert_eq!(format!("{}", result), "3 + 4*b + 6*a + 8*b*a");
    }

    #[test]
    fn test_multiply_complex1() {
        let p1 = Poly::new("a + b").unwrap();
        let p2 = Poly::new("a - b").unwrap();
        let result = p1.multiply(&p2);
        assert_eq!(format!("{}", result), "-b^2 + a^2");
    }

    #[test]
    fn test_multiply_complex2() {
        let p1 = Poly::new("a + b + c").unwrap();
        let p2 = Poly::new("a + b + c").unwrap();
        let result = p1.multiply(&p2);
        assert_eq!(
            format!("{}", result),
            "c^2 + 2*c*b + b^2 + 2*c*a + 2*b*a + a^2"
        );
    }

    #[test]
    fn test_multiply_complex3() {
        let p1 = Poly::new("a^2 + b^2").unwrap();
        let p2 = Poly::new("a^2 - b^2").unwrap();
        let result = p1.multiply(&p2);
        assert_eq!(format!("{}", result), "-b^4 + a^4");
    }

    #[test]
    fn test_multiply_complex4() {
        let p1 = Poly::new("a^3 + b^3").unwrap();
        let p2 = Poly::new("a + b").unwrap();
        let result = p1.multiply(&p2);
        assert_eq!(format!("{}", result), "b^4 + b^3*a + b*a^3 + a^4");
    }

    #[test]
    fn test_multiply_complex5() {
        let p1 = Poly::new("a^2 + b^2 + c^2").unwrap();
        let p2 = Poly::new("a + b + c").unwrap();
        let result = p1.multiply(&p2);
        assert_eq!(
            format!("{}", result),
            "c^3 + c^2*b + c*b^2 + b^3 + c^2*a + b^2*a + c*a^2 + b*a^2 + a^3"
        );
    }

    #[test]
    fn test_extract_factor_and_remainder_constant() {
        let p = Poly::new("5").unwrap();
        let (factor, remainder) = p.extract_factor_and_remainder(0, 1);
        assert_eq!(format!("{}", factor), "0");
        assert_eq!(format!("{}", remainder), "5");
    }

    #[test]
    fn test_extract_factor_and_remainder_higher_variable() {
        let p = Poly::new("1 + 2*b").unwrap();
        let (factor, remainder) = p.extract_factor_and_remainder(0, 1);
        assert_eq!(format!("{}", factor), "0");
        assert_eq!(format!("{}", remainder), "1 + 2*b");
    }

    #[test]
    fn test_extract_factor_and_remainder_same_variable() {
        let p = Poly::new("1 + 2*a + 3*a^2 + 4*a^3").unwrap();
        let (factor, remainder) = p.extract_factor_and_remainder(0, 2);
        assert_eq!(format!("{}", factor), "3 + 4*a");
        assert_eq!(format!("{}", remainder), "1 + 2*a");
    }

    #[test]
    fn test_extract_factor_and_remainder_lower_variable() {
        let p = Poly::new("1 + 2*a + 3*a*b + 4*a*b^2").unwrap();
        let (factor, remainder) = p.extract_factor_and_remainder(1, 1);
        assert_eq!(format!("{}", factor), "3*a + 4*b*a");
        assert_eq!(format!("{}", remainder), "1 + 2*a");
    }

    #[test]
    fn test_single_out() {
        // Test cases for single_out(1) where 1 is 'b'
        let test_cases = vec![
            (
                "2*b - c",
                SingleOutResult::Linear(Rc::new(Poly::new("-1*c").unwrap()), -2),
            ),
            (
                "a - 2*b",
                SingleOutResult::Linear(Rc::new(Poly::new("a").unwrap()), 2),
            ),
            ("b + b^2", SingleOutResult::Nonlinear),
            ("a^2 + b^2", SingleOutResult::Nonlinear),
            (
                "a^2 + b",
                SingleOutResult::Linear(Rc::new(Poly::new("a^2").unwrap()), -1),
            ),
            ("a^2 + a*b", SingleOutResult::Nonlinear),
        ];

        for (input, expected) in test_cases {
            let poly = Poly::new(input).unwrap();
            let result = poly.single_out(1);
            assert_eq!(
                format!("{:?}", result),
                format!("{:?}", expected),
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_substitute_linear() {
        // Test substituting a linear term
        let poly = Poly::new("a + 2*b").unwrap();
        let sub = Poly::new("c + d").unwrap();
        let result = poly.substitute_linear(1, Rc::new(sub), 3);
        assert_eq!(format!("{}", result), "2*d + 2*c + 3*a");

        // Test substituting a quadratic term
        let poly = Poly::new("a + b^2").unwrap();
        let sub = Poly::new("c + d").unwrap();
        let result = poly.substitute_linear(1, Rc::new(sub), 2);
        assert_eq!(format!("{}", result), "d^2 + 2*d*c + c^2 + 4*a");

        // Test substituting a term with multiple variables
        let poly = Poly::new("a*b + b^2").unwrap();
        let sub = Poly::new("c + d").unwrap();
        let result = poly.substitute_linear(1, Rc::new(sub), 2);
        assert_eq!(format!("{}", result), "d^2 + 2*d*c + c^2 + 2*d*a + 2*c*a");
    }

    #[test]
    fn test_compute_factors() {
        // Helper function to create factors array
        fn create_factors() -> Vec<Rc<Poly>> {
            vec![Rc::new(Poly::Constant(0)); 5]
        }

        // Test case 1: a^2 + a + 1
        let poly = Poly::new("a^2 + a + 1").unwrap();
        let mut factors = create_factors();
        poly.compute_factors(1, &mut factors);
        assert_eq!(format!("{}", factors[0]), "1 + a + a^2");
        assert_eq!(format!("{}", factors[1]), "0");
        assert_eq!(format!("{}", factors[2]), "0");
        assert_eq!(format!("{}", factors[3]), "0");
        assert_eq!(format!("{}", factors[4]), "0");

        // Test case 2: b^2 + b + 1
        let poly = Poly::new("b^2 + b + 1").unwrap();
        let mut factors = create_factors();
        poly.compute_factors(1, &mut factors);
        assert_eq!(format!("{}", factors[0]), "1");
        assert_eq!(format!("{}", factors[1]), "1");
        assert_eq!(format!("{}", factors[2]), "1");
        assert_eq!(format!("{}", factors[3]), "0");
        assert_eq!(format!("{}", factors[4]), "0");

        // Test case 3: a*b
        let poly = Poly::new("a*b").unwrap();
        let mut factors = create_factors();
        poly.compute_factors(1, &mut factors);
        assert_eq!(format!("{}", factors[0]), "0");
        assert_eq!(format!("{}", factors[1]), "a");
        assert_eq!(format!("{}", factors[2]), "0");
        assert_eq!(format!("{}", factors[3]), "0");
        assert_eq!(format!("{}", factors[4]), "0");

        // Test case 4: a^2*b + a*b^2 + 2*b + 1 + a^3
        let poly = Poly::new("a^2*b + a*b^2 + 2*b + 1 + a^3").unwrap();
        let mut factors = create_factors();
        poly.compute_factors(1, &mut factors);
        assert_eq!(format!("{}", factors[0]), "1 + a^3");
        assert_eq!(format!("{}", factors[1]), "2 + a^2");
        assert_eq!(format!("{}", factors[2]), "a");
        assert_eq!(format!("{}", factors[3]), "0");
        assert_eq!(format!("{}", factors[4]), "0");

        // Test case 5: c^2 + c + 1
        let poly = Poly::new("c^2 + c + 1").unwrap();
        let mut factors = create_factors();
        poly.compute_factors(1, &mut factors);
        assert_eq!(format!("{}", factors[0]), "1 + c + c^2");
        assert_eq!(format!("{}", factors[1]), "0");
        assert_eq!(format!("{}", factors[2]), "0");
        assert_eq!(format!("{}", factors[3]), "0");
        assert_eq!(format!("{}", factors[4]), "0");

        // Test case 6: b*c
        let poly = Poly::new("b*c").unwrap();
        let mut factors = create_factors();
        poly.compute_factors(1, &mut factors);
        assert_eq!(format!("{}", factors[0]), "0");
        assert_eq!(format!("{}", factors[1]), "c");
        assert_eq!(format!("{}", factors[2]), "0");
        assert_eq!(format!("{}", factors[3]), "0");
        assert_eq!(format!("{}", factors[4]), "0");

        // Test case 7: c^2*b + c*b^2 + 2*b + 1 + c^3
        let poly = Poly::new("c^2*b + c*b^2 + 2*b + 1 + c^3").unwrap();
        let mut factors = create_factors();
        poly.compute_factors(1, &mut factors);
        assert_eq!(format!("{}", factors[0]), "1 + c^3");
        assert_eq!(format!("{}", factors[1]), "2 + c^2");
        assert_eq!(format!("{}", factors[2]), "c");
        assert_eq!(format!("{}", factors[3]), "0");
        assert_eq!(format!("{}", factors[4]), "0");

        // Test case 8: a*b*c + a^2*b^2*c^2
        let poly = Poly::new("a*b*c + a^2*b^2*c^2").unwrap();
        let mut factors = create_factors();
        poly.compute_factors(1, &mut factors);
        assert_eq!(format!("{}", factors[0]), "0");
        assert_eq!(format!("{}", factors[1]), "c*a");
        assert_eq!(format!("{}", factors[2]), "c^2*a^2");
        assert_eq!(format!("{}", factors[3]), "0");
        assert_eq!(format!("{}", factors[4]), "0");
    }

    #[test]
    fn test_get_derivative() {
        // Test derivative of constant
        let poly = Poly::new("5").unwrap();
        let result = poly.get_derivative(0);
        assert_eq!(format!("{}", result), "0");

        // Test derivative of linear term
        let poly = Poly::new("3 + 2*a").unwrap();
        let result = poly.get_derivative(0);
        assert_eq!(format!("{}", result), "2");

        // Test derivative of quadratic term
        let poly = Poly::new("a^2 + 2*a + 1").unwrap();
        let result = poly.get_derivative(0);
        assert_eq!(format!("{}", result), "2 + 2*a");

        // Test derivative of cubic term
        let poly = Poly::new("a^3 + 2*a^2 + 3*a + 4").unwrap();
        let result = poly.get_derivative(0);
        assert_eq!(format!("{}", result), "3 + 4*a + 3*a^2");

        // Test derivative with respect to higher variable
        let poly = Poly::new("a^2 + 2*a + 1").unwrap();
        let result = poly.get_derivative(1);
        assert_eq!(format!("{}", result), "0");

        // Test derivative with respect to lower variable
        let poly = Poly::new("a + 2*b*a + b^2*a").unwrap();
        let result = poly.get_derivative(0);
        assert_eq!(format!("{}", result), "1 + 2*b + b^2");

        // Test derivative of product
        let poly = Poly::new("a*b^2").unwrap();
        let result = poly.get_derivative(0);
        assert_eq!(format!("{}", result), "b^2");

        // Test derivative of complex polynomial
        let poly = Poly::new("a^2*b^2 + 2*a*b + a^3").unwrap();
        let result = poly.get_derivative(0);
        assert_eq!(format!("{}", result), "2*b + 2*b^2*a + 3*a^2");
    }

    #[test]
    fn test_factor() {
        // Test case 1: Simple polynomial that factors
        let poly = Poly::new("a^2 - b^2").unwrap();
        let factors = poly.factor().unwrap();
        assert_eq!(factors.len(), 2);
        // The factors should be (a + b) and (a - b) in some order
        let factor_strings: Vec<String> = factors.iter().map(|f| format!("{}", f)).collect();
        assert!(
            factor_strings.contains(&"b + a".to_string())
                || factor_strings.contains(&"-b - a".to_string())
        );
        assert!(
            factor_strings.contains(&"-b + a".to_string())
                || factor_strings.contains(&"b - a".to_string())
        );

        // Test case 2: Constant polynomial
        let poly = Poly::new("5").unwrap();
        let factors = poly.factor().unwrap();
        assert_eq!(factors.len(), 1);
        assert_eq!(format!("{}", factors[0]), "5");

        // Test case 3: Linear polynomial (should be irreducible)
        let poly = Poly::new("a + b").unwrap();
        let factors = poly.factor().unwrap();
        assert_eq!(factors.len(), 1);
        assert_eq!(format!("{}", factors[0]), "b + a");

        // Test case 4: Quadratic polynomial
        let poly = Poly::new("a^2 + 2*a + 1").unwrap();
        let factors = poly.factor().unwrap();
        assert_eq!(factors.len(), 1);
        assert_eq!(format!("{}", factors[0]), "1 + a");

        // Test case 5: Polynomial with multiple factors
        let poly = Poly::new("a^3 - a").unwrap();
        let factors = poly.factor().unwrap();
        assert_eq!(factors.len(), 3);
        // Should factor as a * (a + 1) * (a - 1)
        let factor_strings: Vec<String> = factors.iter().map(|f| format!("{}", f)).collect();
        assert!(factor_strings.contains(&"a".to_string()));
        assert!(
            factor_strings.contains(&"1 + a".to_string())
                || factor_strings.contains(&"a + 1".to_string())
        );
        assert!(
            factor_strings.contains(&"-1 + a".to_string())
                || factor_strings.contains(&"a - 1".to_string())
        );
    }

    #[test]
    fn test_factor_error_cases() {
        // Test case 1: Polynomial that might cause Pari/GP errors
        let poly = Poly::new("a^2 + b^2 + c^2").unwrap();
        // This might fail if Pari/GP is not available or if the polynomial is too complex
        let result = poly.factor();
        // We don't assert success here since it depends on Pari/GP availability
        match result {
            Ok(factors) => {
                // If it succeeds, verify the factors
                let mut reconstructed = Poly::Constant(1);
                for factor in factors {
                    reconstructed = reconstructed.multiply(&factor);
                }
                assert_eq!(reconstructed, poly);
            }
            Err(e) => {
                // If it fails, the error should be descriptive
                assert!(e.contains("gp.exe") || e.contains("Failed") || e.contains("Invalid"));
            }
        }
    }

    #[test]
    fn test_factor_timeout() {
        let poly_in_pari_format = "(102*b^5 + 204*b^4 + 102*b^3)*a^12 + ((102*c + 102)*b^5 + (102*c^2 + 102*c + 612)*b^4 + (102*c^2 + 714)*b^3 + 204*b^2)*a^11 + ((204*c + 204)*b^3 + (204*c^2 + 1020)*b^2)*a^10 + (34*b^6 + 68*b^5 + (117*c + 34)*b^4 + 234*c*b^3 + 117*c*b^2)*a^9 + (15*b^8 + 30*b^7 + (34*c + 49)*b^6 + (34*c^2 + 34*c + 204)*b^5 + (151*c^2 + 117*c + 238)*b^4 + (117*c^3 + 117*c^2 + 702*c + 68)*b^3 + (117*c^3 + 819*c)*b^2 + 234*c*b)*a^8 + ((15*c + 15)*b^8 + (15*c^2 + 15*c + 90)*b^7 + (15*c^2 + 105)*b^6 + 30*b^5 + (68*c + 68)*b^4 + (68*c^2 + 340)*b^3 + (234*c^2 + 234*c)*b^2 + (234*c^3 + 1170*c)*b)*a^7 + ((30*c + 30)*b^6 + (30*c^2 + 39*c + 150)*b^5 + 78*c*b^4 + 39*c*b^3)*a^6 + (5*b^9 + 10*b^8 + 5*b^7 + (39*c^2 + 39*c)*b^5 + (39*c^3 + 39*c^2 + 234*c)*b^4 + (39*c^3 + 273*c + 3)*b^3 + (78*c + 6)*b^2 + 3*b)*a^5 + ((5*c + 5)*b^9 + (5*c^2 + 5*c + 30)*b^8 + (5*c^2 + 35)*b^7 + 10*b^6 + (78*c^2 + 81*c + 3)*b^3 + (78*c^3 + 3*c^2 + 393*c + 18)*b^2 + (3*c^2 + 21)*b + 6)*a^4 + ((10*c + 10)*b^7 + (10*c^2 + 50)*b^6 + (6*c + 6)*b + (6*c^2 + 30))*a^3 + (b^4 + 2*b^3 + b^2)*a^2 + ((c + 1)*b^4 + (c^2 + c + 6)*b^3 + (c^2 + 7)*b^2 + 2*b)*a + ((2*c + 2)*b^2 + (2*c^2 + 10)*b)";
        // let poly_in_pari_format = "a^2 - b^2";
        let poly = Poly::from_poly_expression(poly_in_pari_format).unwrap();
        let result = poly.factor().unwrap();
        for factor in result.clone() {
            println!("factor: {}", factor);
        }
        assert!(result.len() == 1);
    }
}
