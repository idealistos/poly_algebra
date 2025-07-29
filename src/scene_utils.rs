use crate::elimination::Elimination;
use crate::poly::{Poly, PolyOperations, SingleOutResult};
use crate::scene::{CurveEquationAndFactors, Plot, SceneOptions};
use crate::scene_object::SceneError;
use gcd::Gcd;
use log::info;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct IdentifierExtraction {
    pub function_names: Vec<String>,
    pub object_names: Vec<String>,
    pub field_names: Vec<String>,
    pub method_names: Vec<String>,
}

pub struct SceneUtils;

impl SceneUtils {
    pub fn to_equations(
        python_expressions: String,
    ) -> Result<(Vec<String>, Vec<Plot>), SceneError> {
        let python_code = format!(
            "from equation_processor import *\n{}\n\n# Print all equations\nfor eq in equations:\n    print(eq)\nprint()\n# Print all plots\nfor plot in plots:\n    print(plot)",
            python_expressions
        );
        info!("Python code: {}", python_code);

        let py_dir = Path::new("src/py");
        let output = Command::new("python3")
            .current_dir(py_dir)
            .arg("-c")
            .arg(python_code)
            .output()
            .map_err(|e| SceneError::DatabaseError(format!("Failed to run Python: {}", e)))?;
        println!("output status: {:?}", output.status);
        println!(
            "output stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        if !output.stderr.is_empty() {
            println!(
                "output stderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        if !output.status.success() {
            return Err(SceneError::DatabaseError(format!(
                "Python execution failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut lines = output_str.lines();

        // Collect equations until we hit an empty line
        let mut equations = Vec::new();
        while let Some(line) = lines.next() {
            if line.is_empty() {
                break;
            }
            equations.push(line.to_string());
        }

        // Collect plots
        let mut plots = Vec::new();
        while let Some(line) = lines.next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                plots.push(Plot {
                    name: parts[0].to_string(),
                    x: parts[1].to_string(),
                    y: parts[2].to_string(),
                });
            }
        }

        Ok((equations, plots))
    }

    pub fn get_curve_equation_and_factors(
        equations: Vec<&str>,
        plot: &Plot,
        options: SceneOptions,
    ) -> Result<CurveEquationAndFactors, SceneError> {
        // Convert equations to polynomials
        let mut polys: Vec<Rc<Poly>> = equations
            .into_iter()
            .map(|s| {
                Rc::new(
                    Poly::new(s)
                        .map_err(|e| SceneError::InvalidEquation(e.to_string()))
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();

        // Convert x and y to variable indices
        let (x_var, y_var) = Self::parse_plot_vars(plot)?;

        // Collect all variables used in polynomials
        let mut vars = [false; 256];
        for poly in &polys {
            poly.fill_in_variables(&mut vars);
        }

        // Process each variable that's not x or y
        for (v, has_var) in vars.iter().enumerate() {
            if *has_var && v != x_var as usize && v != y_var as usize {
                // Get single_out results for all polynomials
                let results: Vec<SingleOutResult> =
                    polys.iter().map(|p| p.single_out(v as u8)).collect();

                // Find a linear result to use for substitution
                let mut linear_idx = None;
                let mut linear_poly = None;
                let mut linear_k = 0;

                for (i, result) in results.iter().enumerate() {
                    if let SingleOutResult::Linear(p, k) = result {
                        linear_idx = Some(i);
                        linear_poly = Some(p.clone());
                        linear_k = *k;
                        break;
                    }
                }

                // If we found a linear result, use it to substitute in other polynomials
                if let (Some(idx), Some(poly)) = (linear_idx, linear_poly) {
                    let mut new_polys = Vec::new();
                    for (i, result) in results.iter().enumerate() {
                        if i == idx {
                            continue; // Skip the polynomial we used for substitution
                        }
                        match result {
                            SingleOutResult::Constant => {
                                new_polys.push(polys[i].clone());
                            }
                            SingleOutResult::Linear(_, _) | SingleOutResult::Nonlinear => {
                                new_polys.push(Rc::new(polys[i].substitute_linear(
                                    v as u8,
                                    poly.clone(),
                                    linear_k,
                                )));
                            }
                        }
                    }
                    polys = new_polys;
                }
            }
        }
        polys = Poly::retain_relevant_polys(polys, x_var, y_var);
        info!(
            "Initial reduced system: \n{}",
            polys
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join("\n")
        );

        let systems = Self::split_into_irreducible_systems(polys);

        // Handle possible errors returned from eliminate_and_factor
        let mut all_factors = Vec::new();
        for system in systems {
            let factors = Self::eliminate_and_factor(system, x_var, y_var, &options)?;
            all_factors.extend(factors);
        }

        // Compute unique factors using is_proportional
        let mut unique_factors = Vec::new();
        for factor in all_factors {
            let mut is_duplicate = false;
            for existing_factor in &unique_factors {
                let mut factor_option = None;
                if factor.is_proportional(existing_factor, &mut factor_option) {
                    is_duplicate = true;
                    break;
                }
            }
            if !is_duplicate {
                unique_factors.push(factor);
            }
        }

        // Compute the final equation by multiplying all factors
        let mut equation = if let Some(first_factor) = unique_factors.first() {
            first_factor.clone()
        } else {
            Poly::Constant(1) // Default to 1 if no factors
        };

        for factor in unique_factors.iter().skip(1) {
            equation = equation.multiply(factor);
        }

        Ok(CurveEquationAndFactors {
            curve_equation: equation,
            factors: unique_factors,
        })
    }

    pub fn split_into_irreducible_systems(polys: Vec<Rc<Poly>>) -> Vec<Vec<Rc<Poly>>> {
        if polys.is_empty() {
            return vec![];
        }

        // Factor each polynomial
        let mut factored_polys: Vec<Vec<Rc<Poly>>> = Vec::new();
        for poly in &polys {
            match poly.factor() {
                Ok(factors) => {
                    if factors.len() > 1 {
                        info!(
                            "Equation factored: {}",
                            factors
                                .iter()
                                .map(|p| format!("{}", p))
                                .collect::<Vec<String>>()
                                .join(" * ")
                        );
                    }
                    // Convert factors to Rc<Poly> to avoid cloning
                    let rc_factors: Vec<Rc<Poly>> =
                        factors.into_iter().map(|p| Rc::new(p)).collect();
                    factored_polys.push(rc_factors);
                }
                Err(_) => {
                    // If factoring fails, treat the polynomial as irreducible
                    factored_polys.push(vec![poly.clone()]);
                }
            }
        }

        // Generate all combinations of factors
        let mut combinations = Vec::new();
        Self::generate_combinations(&factored_polys, 0, &mut Vec::new(), &mut combinations);

        combinations
    }

    fn generate_combinations(
        factored_polys: &[Vec<Rc<Poly>>],
        current_index: usize,
        current_combination: &mut Vec<Rc<Poly>>,
        combinations: &mut Vec<Vec<Rc<Poly>>>,
    ) {
        if current_index >= factored_polys.len() {
            // We have a complete combination
            combinations.push(current_combination.clone());
            return;
        }

        // Try each factor from the current polynomial
        for factor in &factored_polys[current_index] {
            current_combination.push(factor.clone());
            Self::generate_combinations(
                factored_polys,
                current_index + 1,
                current_combination,
                combinations,
            );
            current_combination.pop();
        }
    }

    pub fn eliminate_and_factor(
        polys: Vec<Rc<Poly>>,
        x_var: u8,
        y_var: u8,
        options: &SceneOptions,
    ) -> Result<Vec<Poly>, SceneError> {
        let mut polys = polys;
        let mut reduced_further = false;

        // Eliminate variables that are present in univariate polynomials
        loop {
            // Find the first polynomial that is univariate and matches Nested(v, _) with v != x_var and v != y_var
            let mut found_univariate = false;
            let mut uni_poly_index = 0;
            let mut uni_var = 0;

            for (i, poly) in polys.clone().into_iter().enumerate() {
                if poly.is_univariate() {
                    if let Poly::Nested(v, _) = *poly {
                        if v != x_var && v != y_var {
                            uni_poly_index = i;
                            uni_var = v;
                            found_univariate = true;
                            break;
                        }
                    }
                }
            }

            if !found_univariate {
                break;
            }

            reduced_further = true;

            // Create new list with eliminated variables
            let mut new_polys = Vec::new();
            let uni_poly = polys[uni_poly_index].clone();

            for (i, poly) in polys.clone().into_iter().enumerate() {
                if i == uni_poly_index {
                    // Skip the univariate polynomial itself
                    continue;
                }

                // Check if this polynomial contains the variable v
                let mut vars = [false; 256];
                poly.fill_in_variables(&mut vars);

                if poly.get_degree(uni_var) > 0 {
                    // Polynomial contains the variable, eliminate it
                    let eliminated = Self::eliminate_univariate(poly, uni_poly.clone(), uni_var);
                    new_polys.push(eliminated);
                } else {
                    // Polynomial doesn't contain the variable, keep it as is
                    new_polys.push(poly.clone());
                }
            }

            // Replace polys with the new list and continue the loop
            polys = new_polys;
        }
        if reduced_further {
            info!(
                "Reduced system: \n{}",
                polys
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            );
        } else {
            info!("No further reduction possible");
        }

        let mut elimination = Elimination::new(&polys, x_var, y_var, options.reduce_factors);
        loop {
            match elimination.get_var_to_eliminate() {
                Some(var_search_result) => {
                    info!(
                        "--- Eliminating variable {} from\n{}",
                        Poly::var_to_string(var_search_result.var),
                        elimination
                            .polys
                            .iter()
                            .map(|p| p.to_string())
                            .collect::<Vec<String>>()
                            .join("\n")
                    );
                    elimination.eliminate_var(var_search_result);
                }
                None => break,
            }
        }
        let polys = elimination.polys.clone();

        // Check if we have exactly one polynomial left
        if polys.len() != 1 {
            return Err(SceneError::InvalidEquation(format!(
                "Expected exactly one equation after elimination, got {}",
                polys.len()
            )));
        }

        // Verify the remaining polynomial only depends on x and y
        let mut vars = [false; 256];
        polys[0].fill_in_variables(&mut vars);
        for (v, has_var) in vars.iter().enumerate() {
            if *has_var && v != x_var as usize && v != y_var as usize {
                return Err(SceneError::InvalidEquation(format!(
                    "Remaining equation depends on variable {}",
                    (v as u8 + b'a') as char
                )));
            }
        }
        let mut result = polys[0].clone();
        Rc::make_mut(&mut result).reduce_coefficients_if_above(1);
        let factors = result
            .factor()
            .map_err(|e| SceneError::InvalidEquation(e))?;

        let mut product_factors = Vec::new();

        if factors.len() > 1 {
            info!(
                "Factors: {}",
                factors
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            );
        }
        for factor in factors {
            if elimination
                .check_factor(&factor)
                .map_err(|e| SceneError::InvalidEquation(e))?
            {
                product_factors.push(factor);
            } else {
                info!("Skipping factor {}", factor);
            }
        }

        Ok(product_factors)
    }

    fn eliminate_univariate(poly: Rc<Poly>, uni_poly: Rc<Poly>, uni_var: u8) -> Rc<Poly> {
        let uni_coeffs = if let Poly::Nested(_, coeffs) = &*uni_poly {
            coeffs
                .iter()
                .map(|c| {
                    if let Poly::Constant(value) = **c {
                        value
                    } else {
                        0
                    }
                })
                .collect()
        } else {
            vec![0]
        };
        let projections = Self::express_in_basis(poly, &uni_coeffs, uni_var);
        let reduced_projections = Self::remove_gaps(projections, &uni_coeffs);
        Self::reduce_using_projections(reduced_projections.0, reduced_projections.1)
    }

    fn express_in_basis(poly: Rc<Poly>, uni_coeffs: &Vec<i64>, uni_var: u8) -> Vec<Rc<Poly>> {
        let d = uni_coeffs.len() as u32 - 1;
        let d_poly = poly.get_degree(uni_var);

        let lc = uni_coeffs[d as usize];

        // If d_poly >= d, multiply poly by lc^(d_poly - d + 1)
        let adjusted_poly = if d_poly >= d {
            let mut result = Poly::Constant(0);
            let multiplier = lc.pow(d_poly - d + 1);
            result.add_poly_scaled(&*poly, multiplier);
            &Rc::new(result)
        } else {
            &poly
        };

        // Initially, Sum u_power_coeffs[j] u^j = -lc * u^d
        let mut u_power_coeffs = uni_coeffs[0..(d as usize)].to_vec();

        let mut projections = Vec::new();
        let u_components = adjusted_poly.decompose(uni_var);
        // poly Sum c_i u^i, with c_i = u_components[i]
        for (i, u_component) in u_components.into_iter().enumerate() {
            if (i as u32) < d {
                projections.push(u_component.clone());
            } else {
                // Update the formula for lc^{i - d + 1} u^i in terms of 1, u,.., u^{d-1}
                // Note that lc * u^d = -k0 - k1 u - .. - k_{d-1} u^{d-1}

                if (i as u32) > d {
                    // Given -lc^{i - d} u^{i-1} = Sum u_power_coeffs[j] u^j (j = 0,.., d - 1)
                    // multiply the identity by lc * u to find -lc^{i - d + 1} u^i
                    let mut new_u_power_coeffs = vec![0; d as usize];
                    for j in 0..((d as usize) - 1) {
                        new_u_power_coeffs[j + 1] = u_power_coeffs[j] * lc;
                    }
                    for j in 0..(d as usize) {
                        new_u_power_coeffs[j] -= u_power_coeffs[(d - 1) as usize] * uni_coeffs[j];
                    }
                    u_power_coeffs = new_u_power_coeffs;
                }
                // lc^{i - d + 1} * u^i = -u_power_coeffs,
                // thus projections[j] -= u_power_coeffs[j] * c_i / lc^{i - d + 1}
                let lc_degree = lc.pow((i as u32) - d + 1);
                let mut scaled_u_component = u_component.clone();
                Rc::make_mut(&mut scaled_u_component).apply_to_coefficients(|c| c / lc_degree);
                for j in 0..d {
                    Rc::make_mut(&mut projections[j as usize])
                        .add_poly_scaled(&scaled_u_component, -u_power_coeffs[j as usize]);
                }
            }
        }

        projections
    }

    fn remove_gaps(projections: Vec<Rc<Poly>>, uni_coeffs: &Vec<i64>) -> (Vec<Rc<Poly>>, Vec<i64>) {
        // Find indices i where projections[i] is non-empty or uni_coeffs[i] != 0
        let mut indices = Vec::new();
        for i in 0..projections.len().max(uni_coeffs.len()) {
            let projection_non_empty =
                i < projections.len() && !matches!(*projections[i], Poly::Constant(0));
            let coeff_non_zero = i < uni_coeffs.len() && uni_coeffs[i] != 0;

            if projection_non_empty || coeff_non_zero {
                indices.push(i);
            }
        }

        // If no indices found, return empty arrays
        if indices.is_empty() {
            return (Vec::new(), Vec::new());
        }

        // Find GCD of all indices
        let mut gcd = indices[0];
        for &index in indices.iter().skip(1) {
            gcd = gcd.gcd(index);
        }

        // If GCD is 1, return original data
        if gcd == 1 {
            return (projections, uni_coeffs.clone());
        }

        // Otherwise, construct arrays with elements at positions 0, GCD, 2*GCD, etc.
        let mut new_projections = Vec::new();
        let mut new_coeffs = Vec::new();

        let max_len = projections.len().max(uni_coeffs.len());
        let mut i = 0;
        while i < max_len {
            // Add projection if it exists
            if i < projections.len() {
                new_projections.push(projections[i].clone());
            }

            // Add coefficient if it exists
            if i < uni_coeffs.len() {
                new_coeffs.push(uni_coeffs[i]);
            }

            i += gcd;
        }

        (new_projections, new_coeffs)
    }

    fn reduce_using_projections(projections: Vec<Rc<Poly>>, uni_coeffs: Vec<i64>) -> Rc<Poly> {
        let d = projections.len();

        // Get the matrices separately
        let mut i_matrix = Self::get_i_matrix(&uni_coeffs);
        let mut p_matrix = Self::get_p_matrix(&projections);

        // Perform Gaussian elimination
        let mut reduced_p_matrix = Self::gauss_elimination(&mut i_matrix, &mut p_matrix);

        // Reduce each row by common GCD
        for row in reduced_p_matrix.iter_mut() {
            Self::reduce_by_common_gcd(row);
        }

        // Transpose the matrix and reduce by GCD again
        reduced_p_matrix = Self::transpose_matrix(&reduced_p_matrix);
        for row in reduced_p_matrix.iter_mut() {
            Self::reduce_by_common_gcd(row);
        }

        // Compute the determinant of the reduced matrix
        Self::compute_determinant_poly(&reduced_p_matrix)
    }

    fn gauss_elimination(
        i_matrix: &mut Vec<Vec<i64>>,
        p_matrix: &mut Vec<Vec<Rc<Poly>>>,
    ) -> Vec<Vec<Rc<Poly>>> {
        let d = p_matrix.len(); // p_matrix has d rows
        let matrix_size = 2 * d - 1;

        // Initialize remaining_columns: all columns are available initially
        let mut remaining_columns = vec![true; matrix_size];

        // Loop for i = 0, ..., d - 2 (Gaussian elimination on i_matrix)
        for i in 0..(d - 1) {
            // Find the smallest (by absolute value) non-zero value in row i
            let mut min_abs_val = i64::MAX;
            let mut pivot_col = 0;

            for j in 0..matrix_size {
                if remaining_columns[j] && i_matrix[i][j] != 0 {
                    let abs_val = i_matrix[i][j].abs();
                    if abs_val < min_abs_val {
                        min_abs_val = abs_val;
                        pivot_col = j;
                    }
                }
            }

            // If no non-zero element found, the determinant is zero
            if min_abs_val == i64::MAX {
                return Vec::new();
            }

            // Mark this column as used
            remaining_columns[pivot_col] = false;

            // For each remaining column k, perform elimination
            for k in 0..matrix_size {
                if remaining_columns[k] && i_matrix[i][k] != 0 {
                    // Compute the multiplier: we want to eliminate i_matrix[i][k]
                    // using i_matrix[i][pivot_col] as the pivot
                    let pivot_val = i_matrix[i][pivot_col];
                    let target_val = i_matrix[i][k];

                    // Find LCM to avoid division
                    let gcd = pivot_val.unsigned_abs().gcd(target_val.unsigned_abs()) as i64;
                    let pivot_mult = pivot_val / gcd;
                    let target_mult = target_val / gcd;

                    // Apply the linear combination to all rows
                    for l in (i + 1)..(d - 1) {
                        // Update i_matrix
                        i_matrix[l][k] =
                            pivot_mult * i_matrix[l][k] - target_mult * i_matrix[l][pivot_col];
                    }
                    for l in 0..d {
                        // Update p_matrix
                        let mut new_poly = Poly::Constant(0);
                        new_poly.add_poly_scaled(&*p_matrix[l][k], pivot_mult);
                        new_poly.add_poly_scaled(&*p_matrix[l][pivot_col], -target_mult);
                        p_matrix[l][k] = Rc::new(new_poly);
                    }
                }
            }
        }

        // Remove deleted columns from p_matrix before returning
        let mut final_p_matrix = Vec::new();
        for row in p_matrix.iter() {
            let mut new_row = Vec::new();
            for (j, &is_remaining) in remaining_columns.iter().enumerate() {
                if is_remaining {
                    new_row.push(row[j].clone());
                }
            }
            final_p_matrix.push(new_row);
        }

        final_p_matrix
    }

    fn transpose_matrix(matrix: &Vec<Vec<Rc<Poly>>>) -> Vec<Vec<Rc<Poly>>> {
        let rows = matrix.len();
        let cols = matrix[0].len();
        let mut transposed = vec![vec![Rc::new(Poly::Constant(0)); rows]; cols];

        for i in 0..rows {
            for j in 0..cols {
                transposed[j][i] = matrix[i][j].clone();
            }
        }
        transposed
    }

    fn compute_determinant_poly(matrix: &Vec<Vec<Rc<Poly>>>) -> Rc<Poly> {
        let n = matrix.len();
        if n == 0 {
            return Rc::new(Poly::Constant(0));
        }
        if n == 1 {
            return matrix[0][0].clone();
        }
        if n == 2 {
            // For 2x2 matrix: det = a*d - b*c
            let a = &matrix[0][0];
            let b = &matrix[0][1];
            let c = &matrix[1][0];
            let d = &matrix[1][1];

            let ad = a.multiply(d);
            let bc = b.multiply(c);
            let mut result = ad;
            result.add_poly_scaled(&bc, -1);
            return Rc::new(result);
        }

        // For larger matrices, use cofactor expansion along the first row
        let mut determinant = Poly::Constant(0);

        for j in 0..n {
            let cofactor = if j % 2 == 0 { 1 } else { -1 };
            let minor = Self::compute_minor_poly(matrix, 0, j);
            let cofactor_poly = Self::compute_determinant_poly(&minor);

            let term = matrix[0][j].multiply(&*cofactor_poly);

            determinant.add_poly_scaled(&term, cofactor);
        }

        Rc::new(determinant)
    }

    fn compute_minor_poly(
        matrix: &Vec<Vec<Rc<Poly>>>,
        row: usize,
        col: usize,
    ) -> Vec<Vec<Rc<Poly>>> {
        let n = matrix.len();
        let mut minor = Vec::new();

        for i in 0..n {
            if i != row {
                let mut minor_row = Vec::new();
                for j in 0..n {
                    if j != col {
                        minor_row.push(matrix[i][j].clone());
                    }
                }
                minor.push(minor_row);
            }
        }

        minor
    }

    fn reduce_by_common_gcd(polys: &mut Vec<Rc<Poly>>) {
        // Find the common GCD of coefficients across all polynomials
        let mut common_gcd = 0i64;

        for poly in polys.iter() {
            let poly_gcd = poly.get_coefficient_gcd();
            if poly_gcd != 0 {
                if common_gcd == 0 {
                    common_gcd = poly_gcd;
                } else {
                    common_gcd = common_gcd.unsigned_abs().gcd(poly_gcd.unsigned_abs()) as i64;
                }
            }
        }

        // If no common GCD found or it's 1, do nothing
        if common_gcd == 0 || common_gcd == 1 {
            return;
        }

        // Divide all coefficients by the common GCD
        for poly in polys.iter_mut() {
            Rc::make_mut(poly).apply_to_coefficients(|x| x / common_gcd);
        }
    }

    fn get_i_matrix(uni_coeffs: &Vec<i64>) -> Vec<Vec<i64>> {
        let d = uni_coeffs.len() - 1; // uni_coeffs has size d + 1
        let matrix_size = 2 * d - 1;

        let mut i_matrix = Vec::new();
        for i in 0..(d - 1) {
            let mut row = vec![0i64; matrix_size];
            // Place uni_coeffs starting at position i
            for j in 0..uni_coeffs.len() {
                if i + j < matrix_size {
                    row[i + j] = uni_coeffs[j];
                }
            }
            i_matrix.push(row);
        }
        i_matrix
    }

    fn get_p_matrix(projections: &Vec<Rc<Poly>>) -> Vec<Vec<Rc<Poly>>> {
        let d = projections.len(); // projections has size d
        let matrix_size = 2 * d - 1;

        let mut p_matrix = Vec::new();
        for i in 0..d {
            let mut row = vec![Rc::new(Poly::Constant(0)); matrix_size];
            // Place projections starting at position i
            for j in 0..projections.len() {
                if i + j < matrix_size {
                    row[i + j] = projections[j].clone();
                }
            }
            p_matrix.push(row);
        }
        p_matrix
    }

    pub fn parse_plot_vars(plot: &Plot) -> Result<(u8, u8), SceneError> {
        let x_var =
            Poly::parse_var(&plot.x).map_err(|e| SceneError::InvalidEquation(e.to_string()))?;
        let y_var =
            Poly::parse_var(&plot.y).map_err(|e| SceneError::InvalidEquation(e.to_string()))?;
        Ok((x_var, y_var))
    }

    pub fn evaluate_initial_values(
        python_expressions: &String,
        expressions: &Vec<String>,
    ) -> Result<Vec<f64>, SceneError> {
        let prepared_expressions = expressions
            .iter()
            .map(|s| Self::prepare_expression(s))
            .map(|s| format!("print(({}).float_initial)", s))
            .collect::<Vec<String>>()
            .join("\n");
        let python_code = format!(
            "from equation_processor import *\ncompute_float_initial[0] = True\n{}\n{}",
            python_expressions, prepared_expressions
        );
        info!("Python code: {}", python_code);

        let py_dir = Path::new("src/py");
        let output = Command::new("python3")
            .current_dir(py_dir)
            .arg("-c")
            .arg(python_code)
            .output()
            .map_err(|e| SceneError::DatabaseError(format!("Failed to run Python: {}", e)))?;
        println!("output status: {:?}", output.status);
        println!(
            "output stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        if !output.stderr.is_empty() {
            println!(
                "output stderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        if !output.status.success() {
            return Err(SceneError::DatabaseError(format!(
                "Python execution failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();

        // Parse each line as a float
        let mut values = Vec::new();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                continue; // Skip empty lines
            }

            match trimmed_line.parse::<f64>() {
                Ok(value) => values.push(value),
                Err(_) => {
                    return Err(SceneError::DatabaseError(format!(
                        "Failed to parse line {} as float: '{}'",
                        line_num + 1,
                        trimmed_line
                    )));
                }
            }
        }

        Ok(values)
    }

    pub fn extract_identifiers(expression: &String) -> IdentifierExtraction {
        let re = Regex::new(r"\b[a-zA-Z_]\w*\b").unwrap();
        let mut function_names = HashSet::new();
        let mut object_names = HashSet::new();
        let mut field_names = HashSet::new();
        let mut method_names = HashSet::new();

        for mat in re.find_iter(expression) {
            let identifier = mat.as_str();
            let start = mat.start();
            let end = mat.end();

            // Check what precedes the identifier (ignoring whitespace)
            let mut preceded_by_dot = false;
            if start > 0 {
                let before_identifier = &expression[..start];
                let trimmed_before = before_identifier.trim_end();
                if !trimmed_before.is_empty() {
                    preceded_by_dot = trimmed_before.ends_with('.');
                }
            }

            // Check what follows the identifier (ignoring whitespace)
            let mut followed_by_paren = false;
            if end < expression.len() {
                let after_identifier = &expression[end..];
                let trimmed_after = after_identifier.trim_start();
                if !trimmed_after.is_empty() {
                    followed_by_paren = trimmed_after.starts_with('(');
                }
            }

            // Categorize the identifier
            if followed_by_paren && !preceded_by_dot {
                function_names.insert(identifier.to_string());
            } else if !followed_by_paren && !preceded_by_dot {
                object_names.insert(identifier.to_string());
            } else if followed_by_paren && preceded_by_dot {
                method_names.insert(identifier.to_string());
            } else if !followed_by_paren && preceded_by_dot {
                field_names.insert(identifier.to_string());
            }
        }

        IdentifierExtraction {
            function_names: {
                let mut vec: Vec<String> = function_names.into_iter().collect();
                vec.sort();
                vec
            },
            object_names: {
                let mut vec: Vec<String> = object_names.into_iter().collect();
                vec.sort();
                vec
            },
            field_names: {
                let mut vec: Vec<String> = field_names.into_iter().collect();
                vec.sort();
                vec
            },
            method_names: {
                let mut vec: Vec<String> = method_names.into_iter().collect();
                vec.sort();
                vec
            },
        }
    }

    pub fn prepare_expression(expression: &String) -> String {
        let formula = expression.replace("^", "**");
        // Use regex to find standalone integers and wrap them with i()
        let re = Regex::new(r"\(\s*(\d+)\s*/\s*(\d+)\s*\)").unwrap();
        let formula = re.replace_all(&formula, "q(__$1, __$2)").to_string();
        let re = Regex::new(r"\b\d+\b").unwrap();
        let formula = re.replace_all(&formula, "i($0)").to_string();
        let re = Regex::new(r"\b__(\d+)\b").unwrap();
        let formula = re.replace_all(&formula, "$1").to_string();
        formula
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_curve_equation() {
        let result = SceneUtils::get_curve_equation_and_factors(
            vec!["a^2 - 2*c", "b^2 - 3*d", "d - c"],
            &Plot {
                name: "plotA".to_string(),
                x: "a".to_string(),
                y: "b".to_string(),
            },
            SceneOptions::default(),
        )
        .unwrap();
        assert_eq!(format!("{}", result.curve_equation), "2*b^2 - 3*a^2");
    }

    #[test]
    fn test_equations_and_plots_generation() {
        let python_expressions = vec![
            "A = FixedPoint(0, 0)",
            "X = FreePoint(3, 4)",
            "plot(\"P1\", X)",
            "is_constant(d_sqr(A, X))",
        ]
        .join("\n");
        let (equations, plots) = SceneUtils::to_equations(python_expressions).unwrap();

        // The equations should contain specific formulas for the distance
        let expected_equations = [
            "0 - a - c",
            "c^2 - d",
            "0 - b - e",
            "e^2 - f",
            "d + f - g",
            "g - 25",
        ];

        for eq in expected_equations.iter() {
            assert!(
                equations.contains(&eq.to_string()),
                "Expected equation '{}' not found in equations: {:?}",
                eq,
                equations
            );
        }

        // Check plots
        assert_eq!(plots.len(), 1);
        assert_eq!(plots[0].x, "a");
        assert_eq!(plots[0].y, "b");
    }

    #[test]
    fn test_parse_plot_vars() {
        let plot = Plot {
            name: "test_plot".to_string(),
            x: "a".to_string(),
            y: "b".to_string(),
        };

        let (x_var, y_var) = SceneUtils::parse_plot_vars(&plot).unwrap();
        assert_eq!(x_var, 0); // 'a' should be variable 0
        assert_eq!(y_var, 1); // 'b' should be variable 1
    }

    #[test]
    fn test_parse_plot_vars_invalid() {
        let plot = Plot {
            name: "test_plot".to_string(),
            x: "invalid_var".to_string(),
            y: "b".to_string(),
        };

        let result = SceneUtils::parse_plot_vars(&plot);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_identifiers_function_call() {
        let expression = "d(A, X)".to_string();
        let result = SceneUtils::extract_identifiers(&expression);

        assert_eq!(result.function_names, vec!["d"]);
        assert_eq!(result.object_names, vec!["A", "X"]);
        assert_eq!(result.field_names, Vec::<String>::new());
        assert_eq!(result.method_names, Vec::<String>::new());
    }

    #[test]
    fn test_extract_identifiers_field_access() {
        let expression = "B.y + A.x".to_string();
        let result = SceneUtils::extract_identifiers(&expression);

        assert_eq!(result.function_names, Vec::<String>::new());
        assert_eq!(result.object_names, vec!["A", "B"]);
        assert_eq!(result.field_names, vec!["x", "y"]);
        assert_eq!(result.method_names, Vec::<String>::new());
    }

    #[test]
    fn test_extract_identifiers_function_with_number() {
        let expression = "sqrt(2) * t".to_string();
        let result = SceneUtils::extract_identifiers(&expression);

        assert_eq!(result.function_names, vec!["sqrt"]);
        assert_eq!(result.object_names, vec!["t"]);
        assert_eq!(result.field_names, Vec::<String>::new());
        assert_eq!(result.method_names, Vec::<String>::new());
    }

    #[test]
    fn test_extract_identifiers_method_calls() {
        let expression = "t.abs() + u.abs()".to_string();
        let result = SceneUtils::extract_identifiers(&expression);

        assert_eq!(result.function_names, Vec::<String>::new());
        assert_eq!(result.object_names, vec!["t", "u"]);
        assert_eq!(result.field_names, Vec::<String>::new());
        assert_eq!(result.method_names, vec!["abs"]);
    }

    #[test]
    fn test_extract_identifiers_number_only() {
        let expression = "10".to_string();
        let result = SceneUtils::extract_identifiers(&expression);

        assert_eq!(result.function_names, Vec::<String>::new());
        assert_eq!(result.object_names, Vec::<String>::new());
        assert_eq!(result.field_names, Vec::<String>::new());
        assert_eq!(result.method_names, Vec::<String>::new());
    }

    #[tokio::test]
    async fn test_evaluate_initial_values() {
        let python_expressions = "X = FreePoint(1, 2)".to_string();
        let expressions = vec!["2^(1/2)".to_string(), "X.x + X.y".to_string()];

        let result = SceneUtils::evaluate_initial_values(&python_expressions, &expressions);

        match result {
            Ok(values) => {
                assert_eq!(values.len(), 2, "Expected 2 values, got {}", values.len());

                // Check first expression: 2^(1/2) = sqrt(2) ≈ 1.4142135623730951
                let sqrt_2 = values[0];
                let expected_sqrt_2 = 2.0_f64.sqrt();
                assert!(
                    (sqrt_2 - expected_sqrt_2).abs() < 1e-10,
                    "Expected sqrt(2) ≈ {}, got {}",
                    expected_sqrt_2,
                    sqrt_2
                );

                // Check second expression: X.x + X.y = 1 + 2 = 3
                let sum = values[1];
                assert!(
                    (sum - 3.0).abs() < 1e-10,
                    "Expected X.x + X.y = 3, got {}",
                    sum
                );

                println!("Successfully evaluated expressions:");
                println!("  2^(1/2) = {}", sqrt_2);
                println!("  X.x + X.y = {}", sum);
            }
            Err(e) => {
                panic!("Failed to evaluate initial values: {:?}", e);
            }
        }
    }

    #[test]
    fn test_split_into_irreducible_systems_empty() {
        let polys: Vec<Rc<Poly>> = vec![];
        let result = SceneUtils::split_into_irreducible_systems(polys);
        assert_eq!(result, vec![] as Vec<Vec<Rc<Poly>>>);
    }

    #[test]
    fn test_split_into_irreducible_systems_single_poly() {
        let polys = vec![Rc::new(Poly::new("x^2 - 1").unwrap())];
        let result = SceneUtils::split_into_irreducible_systems(polys);

        // Should return systems with factors of x^2 - 1 = (x-1)(x+1)
        assert_eq!(result.len(), 2);

        // Check that each system has exactly one polynomial
        for system in &result {
            assert_eq!(system.len(), 1);
        }

        // Check that the systems contain the expected factors
        let system_strings: Vec<String> =
            result.iter().map(|system| system[0].to_string()).collect();

        // The exact factors depend on the factoring implementation
        // but we should have 2 different systems
        assert_eq!(system_strings.len(), 2);
        assert_ne!(system_strings[0], system_strings[1]);
    }

    #[test]
    fn test_split_into_irreducible_systems_two_polys() {
        let polys = vec![
            Rc::new(Poly::new("x^2 - 1").unwrap()), // (x-1)(x+1)
            Rc::new(Poly::new("y^2 - 4").unwrap()), // (y-2)(y+2)
        ];
        let result = SceneUtils::split_into_irreducible_systems(polys);

        // Should return 2 * 2 = 4 combinations
        assert_eq!(result.len(), 4);

        // Check that each system has exactly two polynomials
        for system in &result {
            assert_eq!(system.len(), 2);
        }

        // All systems should be different
        let system_strings: Vec<String> = result
            .iter()
            .map(|system| {
                let mut sorted = system.iter().map(|p| p.to_string()).collect::<Vec<_>>();
                sorted.sort();
                sorted.join(";")
            })
            .collect();

        // Should have 4 unique combinations
        let unique_systems: std::collections::HashSet<String> =
            system_strings.into_iter().collect();
        assert_eq!(unique_systems.len(), 4);
    }

    #[test]
    fn test_split_into_irreducible_systems_factoring_failure() {
        // Create a polynomial that might fail to factor (complex case)
        let polys = vec![
            Rc::new(Poly::new("x^2 + 1").unwrap()), // Irreducible over reals
            Rc::new(Poly::new("y^2 - 1").unwrap()), // (y-1)(y+1)
        ];
        let result = SceneUtils::split_into_irreducible_systems(polys);

        // Should handle factoring failure gracefully
        assert!(!result.is_empty());

        // Each system should have 2 polynomials
        for system in &result {
            assert_eq!(system.len(), 2);
        }
    }

    #[test]
    fn test_express_in_basis() {
        // Test case 1: uni_coeffs = [-2, 3, 1] (a^2 + 3*a - 2), poly = b*a + b^2
        let uni_coeffs = vec![-2, 3, 1]; // coefficients of a^2 + 3*a - 2
        let poly = Rc::new(Poly::new("b*a + b^2").unwrap());
        let result = SceneUtils::express_in_basis(poly, &uni_coeffs, 0); // variable 'a'

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Rc::new(Poly::new("b^2").unwrap())); // constant term
        assert_eq!(result[1], Rc::new(Poly::new("b").unwrap())); // coefficient of a

        // Test case 2: uni_coeffs = [-2, 3, 1] (a^2 + 3*a - 2), poly = b*a^2
        let poly = Rc::new(Poly::new("b*a^2").unwrap());
        let result = SceneUtils::express_in_basis(poly, &uni_coeffs, 0);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Rc::new(Poly::new("2*b").unwrap())); // constant term
        assert_eq!(result[1], Rc::new(Poly::new("-3*b").unwrap())); // coefficient of a

        // Test case 3: uni_coeffs = [-2, 3, 1] (a^2 + 3*a - 2), poly = b*a^3
        let poly = Rc::new(Poly::new("b*a^3").unwrap());
        let result = SceneUtils::express_in_basis(poly, &uni_coeffs, 0);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Rc::new(Poly::new("-6*b").unwrap())); // constant term
        assert_eq!(result[1], Rc::new(Poly::new("11*b").unwrap())); // coefficient of a
    }

    #[test]
    fn test_remove_gaps() {
        // Test case 1: uni_coeffs = [-2, 3, 1] (a^2 + 3*a - 2), projections = [b, c]
        let uni_coeffs = vec![-2, 3, 1]; // coefficients of a^2 + 3*a - 2
        let projections = vec![
            Rc::new(Poly::new("b").unwrap()),
            Rc::new(Poly::new("c").unwrap()),
        ];
        let result = SceneUtils::remove_gaps(projections, &uni_coeffs);
        let poly_strs: Vec<String> = result.0.iter().map(|p| format!("{}", p)).collect();
        assert_eq!(poly_strs, vec!["b", "c"]);
        assert_eq!(result.1, uni_coeffs);

        // Test case 2: uni_coeffs = [-2, 0, 1] (a^2 - 2), projections = [b, 0]
        let uni_coeffs = vec![-2, 0, 1]; // coefficients of a^2 - 2
        let projections = vec![
            Rc::new(Poly::new("b").unwrap()),
            Rc::new(Poly::new("0").unwrap()),
        ];
        let result = SceneUtils::remove_gaps(projections, &uni_coeffs);
        let poly_strs: Vec<String> = result.0.iter().map(|p| format!("{}", p)).collect();
        assert_eq!(poly_strs, vec!["b"]);
        assert_eq!(result.1, vec![-2, 1]);

        // Test case 3: uni_coeffs = [-2, 0, 3, 0, 1] (a^4 + 3*a^2 - 2), projections = [b, 0, c, 0]
        let uni_coeffs = vec![-2, 0, 3, 0, 1]; // coefficients of a^4 + 3*a^2 - 2
        let projections = vec![
            Rc::new(Poly::new("b").unwrap()),
            Rc::new(Poly::new("0").unwrap()),
            Rc::new(Poly::new("c").unwrap()),
            Rc::new(Poly::new("0").unwrap()),
        ];
        let result = SceneUtils::remove_gaps(projections, &uni_coeffs);
        let poly_strs: Vec<String> = result.0.iter().map(|p| format!("{}", p)).collect();
        assert_eq!(poly_strs, vec!["b", "c"]);
        assert_eq!(result.1, vec![-2, 3, 1]);
    }

    #[test]
    fn test_reduce_using_projections() {
        // Test case: d = 2, uni_coeffs = [-2, 3, 1], projections = [5, 7]
        let uni_coeffs = vec![-2, 3, 1]; // coefficients of a^2 + 3*a - 2
        let projections = vec![Rc::new(Poly::Constant(5)), Rc::new(Poly::Constant(7))];

        let result = SceneUtils::reduce_using_projections(projections, uni_coeffs);

        // The result should be a polynomial representing the determinant
        // For this simple case, we expect a constant polynomial
        assert!(matches!(*result, Poly::Constant(_)));
    }

    #[test]
    fn test_get_i_matrix() {
        // Test case: d = 2, uni_coeffs = [-2, 3, 1] (size d + 1 = 3)
        let uni_coeffs = vec![-2, 3, 1];
        let i_matrix = SceneUtils::get_i_matrix(&uni_coeffs);

        // Should have d - 1 = 1 row
        assert_eq!(i_matrix.len(), 1);

        // Matrix size should be 2*d - 1 = 3
        assert_eq!(i_matrix[0].len(), 3);

        // First row should be [-2, 3, 1]
        assert_eq!(i_matrix[0], vec![-2, 3, 1]);
    }

    #[test]
    fn test_get_p_matrix() {
        // Test case: d = 2, projections = [5, 7] (size d = 2)
        let projections = vec![Rc::new(Poly::Constant(5)), Rc::new(Poly::Constant(7))];
        let p_matrix = SceneUtils::get_p_matrix(&projections);

        // Should have d = 2 rows
        assert_eq!(p_matrix.len(), 2);

        // Matrix size should be 2*d - 1 = 3
        assert_eq!(p_matrix[0].len(), 3);
        assert_eq!(p_matrix[1].len(), 3);

        // First row should be [5, 7, 0]
        assert_eq!(*p_matrix[0][0], Poly::Constant(5));
        assert_eq!(*p_matrix[0][1], Poly::Constant(7));
        assert_eq!(*p_matrix[0][2], Poly::Constant(0));

        // Second row should be [0, 5, 7]
        assert_eq!(*p_matrix[1][0], Poly::Constant(0));
        assert_eq!(*p_matrix[1][1], Poly::Constant(5));
        assert_eq!(*p_matrix[1][2], Poly::Constant(7));
    }

    #[test]
    fn test_gauss_elimination() {
        // Test case: d = 2
        let mut i_matrix = vec![vec![-2, 3, 1]]; // 1 row, 3 columns
        let mut p_matrix = vec![
            vec![
                Rc::new(Poly::Constant(5)),
                Rc::new(Poly::Constant(7)),
                Rc::new(Poly::Constant(0)),
            ],
            vec![
                Rc::new(Poly::Constant(0)),
                Rc::new(Poly::Constant(5)),
                Rc::new(Poly::Constant(7)),
            ],
        ]; // 2 rows, 3 columns

        let reduced_p_matrix = SceneUtils::gauss_elimination(&mut i_matrix, &mut p_matrix);

        // The reduced p_matrix should have the same number of rows but fewer columns
        assert_eq!(reduced_p_matrix.len(), 2);
        // Should have fewer columns since one column was eliminated
        assert_eq!(
            reduced_p_matrix[0],
            vec![Rc::new(Poly::Constant(5)), Rc::new(Poly::Constant(7))]
        );
        assert_eq!(
            reduced_p_matrix[1],
            vec![Rc::new(Poly::Constant(14)), Rc::new(Poly::Constant(-16))]
        );
    }

    #[test]
    fn test_reduce_by_common_gcd() {
        // Test case: polynomials with common GCD of 6
        let mut polys = vec![
            Rc::new(Poly::new("6*x + 12").unwrap()),  // GCD = 6
            Rc::new(Poly::new("18*y + 24").unwrap()), // GCD = 6
        ];

        SceneUtils::reduce_by_common_gcd(&mut polys);

        // Should have same number of polynomials
        assert_eq!(polys.len(), 2);

        // Coefficients should be divided by 6
        assert_eq!(*polys[0], Poly::new("x + 2").unwrap());
        assert_eq!(*polys[1], Poly::new("3*y + 4").unwrap());
    }

    #[test]
    fn test_reduce_by_common_gcd_no_common_factor() {
        // Test case: polynomials with no common GCD
        let mut polys = vec![
            Rc::new(Poly::new("x + 2").unwrap()),
            Rc::new(Poly::new("3*y + 4").unwrap()),
        ];

        let original_polys = polys.clone();
        SceneUtils::reduce_by_common_gcd(&mut polys);

        // Should return the original polynomials unchanged
        assert_eq!(polys.len(), 2);
        assert_eq!(*polys[0], *original_polys[0]);
        assert_eq!(*polys[1], *original_polys[1]);
    }

    #[test]
    fn test_transpose_matrix() {
        // Test case: 2x3 matrix
        let matrix = vec![
            vec![
                Rc::new(Poly::Constant(1)),
                Rc::new(Poly::Constant(2)),
                Rc::new(Poly::Constant(3)),
            ],
            vec![
                Rc::new(Poly::Constant(4)),
                Rc::new(Poly::Constant(5)),
                Rc::new(Poly::Constant(6)),
            ],
        ];

        let transposed = SceneUtils::transpose_matrix(&matrix);

        // Should be 3x2 matrix
        assert_eq!(transposed.len(), 3);
        assert_eq!(transposed[0].len(), 2);
        assert_eq!(transposed[1].len(), 2);
        assert_eq!(transposed[2].len(), 2);

        // Check transposed values
        assert_eq!(*transposed[0][0], Poly::Constant(1));
        assert_eq!(*transposed[0][1], Poly::Constant(4));
        assert_eq!(*transposed[1][0], Poly::Constant(2));
        assert_eq!(*transposed[1][1], Poly::Constant(5));
        assert_eq!(*transposed[2][0], Poly::Constant(3));
        assert_eq!(*transposed[2][1], Poly::Constant(6));
    }

    #[test]
    fn test_compute_determinant_poly_1x1() {
        let matrix = vec![vec![Rc::new(Poly::Constant(5))]];
        let det = SceneUtils::compute_determinant_poly(&matrix);
        assert_eq!(*det, Poly::Constant(5));
    }

    #[test]
    fn test_compute_determinant_poly_2x2() {
        let matrix = vec![
            vec![Rc::new(Poly::Constant(1)), Rc::new(Poly::Constant(2))],
            vec![Rc::new(Poly::Constant(3)), Rc::new(Poly::Constant(4))],
        ];
        let det = SceneUtils::compute_determinant_poly(&matrix);
        // det = 1*4 - 2*3 = 4 - 6 = -2
        assert_eq!(*det, Poly::Constant(-2));
    }

    #[test]
    fn test_compute_determinant_poly_3x3() {
        let matrix = vec![
            vec![
                Rc::new(Poly::Constant(1)),
                Rc::new(Poly::Constant(2)),
                Rc::new(Poly::Constant(3)),
            ],
            vec![
                Rc::new(Poly::Constant(4)),
                Rc::new(Poly::Constant(5)),
                Rc::new(Poly::Constant(6)),
            ],
            vec![
                Rc::new(Poly::Constant(7)),
                Rc::new(Poly::Constant(8)),
                Rc::new(Poly::Constant(9)),
            ],
        ];
        let det = SceneUtils::compute_determinant_poly(&matrix);
        // det = 1*(5*9 - 6*8) - 2*(4*9 - 6*7) + 3*(4*8 - 5*7)
        // = 1*(45-48) - 2*(36-42) + 3*(32-35)
        // = 1*(-3) - 2*(-6) + 3*(-3)
        // = -3 + 12 - 9 = 0
        assert_eq!(*det, Poly::Constant(0));
    }

    #[test]
    fn test_compute_minor_poly() {
        let matrix = vec![
            vec![
                Rc::new(Poly::Constant(1)),
                Rc::new(Poly::Constant(2)),
                Rc::new(Poly::Constant(3)),
            ],
            vec![
                Rc::new(Poly::Constant(4)),
                Rc::new(Poly::Constant(5)),
                Rc::new(Poly::Constant(6)),
            ],
            vec![
                Rc::new(Poly::Constant(7)),
                Rc::new(Poly::Constant(8)),
                Rc::new(Poly::Constant(9)),
            ],
        ];
        let minor = SceneUtils::compute_minor_poly(&matrix, 0, 0);

        // Should be 2x2 matrix
        assert_eq!(minor.len(), 2);
        assert_eq!(minor[0].len(), 2);
        assert_eq!(minor[1].len(), 2);

        // Check values (removing row 0, col 0)
        assert_eq!(*minor[0][0], Poly::Constant(5));
        assert_eq!(*minor[0][1], Poly::Constant(6));
        assert_eq!(*minor[1][0], Poly::Constant(8));
        assert_eq!(*minor[1][1], Poly::Constant(9));
    }

    #[test]
    fn test_eliminate_univariate() {
        // Variable 'a' corresponds to uni_var = 0
        let uni_poly = Rc::new(Poly::new("2*a^2 - 1").unwrap());
        let poly = Rc::new(Poly::new("a^3*b + a^2*c - a").unwrap());

        let result = SceneUtils::eliminate_univariate(poly, uni_poly, 0); // uni_var = 0 for 'a'

        assert_eq!(format!("{}", *result), "-4 + 2*c^2 + 4*b - b^2");

        let uni_poly = Rc::new(Poly::new("2*a^3 - 1").unwrap());
        let poly = Rc::new(Poly::new("a^2*b + c").unwrap());

        let result = SceneUtils::eliminate_univariate(poly, uni_poly, 0); // uni_var = 0 for 'a'

        assert_eq!(format!("{}", *result), "4*c^3 + b^3");
    }
}
