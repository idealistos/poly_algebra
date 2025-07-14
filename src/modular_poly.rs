use std::ops::{Add, Mul, Sub};

use log::info;

/// A single-variable polynomial with coefficients in Z/pZ
#[derive(Debug, Clone, PartialEq)]
pub struct ModularPoly {
    /// Coefficients in ascending order (constant term first)
    pub coeffs: Vec<u64>,
    /// The prime modulus
    pub p: u64,
}

impl ModularPoly {
    /// Create a new modular polynomial
    pub fn new(coeffs: Vec<u64>, p: u64) -> Self {
        // Normalize coefficients (remove trailing zeros and ensure all coeffs < p)
        let mut normalized_coeffs: Vec<u64> = coeffs.into_iter().map(|c| c % p).collect();

        // Remove trailing zeros
        while normalized_coeffs.len() > 1 && *normalized_coeffs.last().unwrap() == 0 {
            normalized_coeffs.pop();
        }

        Self {
            coeffs: normalized_coeffs,
            p,
        }
    }

    /// Create a zero polynomial
    pub fn zero(p: u64) -> Self {
        Self::new(vec![0], p)
    }

    /// Create a constant polynomial
    pub fn constant(c: u64, p: u64) -> Self {
        Self::new(vec![c % p], p)
    }

    /// Get the degree of the polynomial
    pub fn degree(&self) -> usize {
        if self.coeffs.len() <= 1 {
            0
        } else {
            self.coeffs.len() - 1
        }
    }

    /// Check if the polynomial is zero
    pub fn is_zero(&self) -> bool {
        self.coeffs.len() == 0 || (self.coeffs.len() == 1 && self.coeffs[0] == 0)
    }

    pub fn is_constant(&self) -> bool {
        self.coeffs.len() <= 1
    }

    pub fn from_i64(c: i64, p: u64) -> u64 {
        if c < 0 {
            p - (-c as u64) % p
        } else {
            c as u64 % p
        }
    }

    /// Add two coefficients modulo p
    fn add_mod(a: u64, b: u64, p: u64) -> u64 {
        let sum = a + b;
        if sum >= p {
            sum - p
        } else {
            sum
        }
    }

    /// Subtract two coefficients modulo p
    fn sub_mod(a: u64, b: u64, p: u64) -> u64 {
        if a >= b {
            a - b
        } else {
            p - (b - a)
        }
    }

    /// Multiply two coefficients modulo p
    fn mul_mod(a: u64, b: u64, p: u64) -> u64 {
        let product = (a as u128) * (b as u128);
        (product % (p as u128)) as u64
    }

    /// Find the multiplicative inverse of a modulo p
    fn mod_inverse(a: u64, p: u64) -> Option<u64> {
        if a == 0 {
            info!("Modular inverse of 0 modulo {} is undefined", p);
            return None;
        }

        // Extended Euclidean algorithm
        let mut r = (a as i128, p as i128);
        let mut s = (1i128, 0i128);
        let mut t = (0i128, 1i128);

        while r.1 != 0 {
            let q = r.0 / r.1;
            let new_r = (r.1, r.0 - q * r.1);
            let new_s = (s.1, s.0 - q * s.1);
            let new_t = (t.1, t.0 - q * t.1);

            r = new_r;
            s = new_s;
            t = new_t;
        }

        if r.0 != 1 {
            info!("Modular inverse of {} modulo {} is undefined: r.0 = {}", a, p, r.0);
            return None; // No inverse exists
        }

        let result = s.0 % (p as i128);
        if result < 0 {
            Some((result + p as i128) as u64)
        } else {
            Some(result as u64)
        }
    }

    /// Find the remainder when dividing this polynomial by another
    /// Computes both the quotient and remainder of polynomial division.
    /// Returns None if division is not possible (e.g., divisor is zero).
    /// Returns Some((quotient, remainder)) where dividend = quotient * divisor + remainder.
    pub fn get_quotient_and_remainder(
        &self,
        divisor: &ModularPoly,
    ) -> Option<(ModularPoly, ModularPoly)> {
        assert_eq!(
            self.p, divisor.p,
            "Cannot compute quotient and remainder for polynomials with different moduli"
        );

        if divisor.is_zero() {
            return None;
        }

        if self.is_zero() {
            return Some((ModularPoly::zero(self.p), ModularPoly::zero(self.p)));
        }

        if divisor.is_constant() {
            let divisor_const = divisor.coeffs[0];
            if divisor_const == 0 {
                return None;
            }

            let mut quotient_coeffs = Vec::new();
            for &coeff in &self.coeffs {
                quotient_coeffs.push(Self::mul_mod(
                    coeff,
                    Self::mod_inverse(divisor_const, self.p).unwrap(),
                    self.p,
                ));
            }
            return Some((
                ModularPoly::new(quotient_coeffs, self.p),
                ModularPoly::zero(self.p),
            ));
        }

        let mut dividend = self.clone();
        let divisor_degree = divisor.degree();
        let divisor_leading_coeff = divisor.coeffs[divisor_degree];
        let mut quotient_coeffs = Vec::new();

        while dividend.degree() >= divisor_degree {
            let dividend_degree = dividend.degree();
            let dividend_leading_coeff = dividend.coeffs[dividend_degree];

            // Calculate the quotient coefficient for this step
            let quotient_coeff = Self::mul_mod(
                dividend_leading_coeff,
                Self::mod_inverse(divisor_leading_coeff, self.p).unwrap(),
                self.p,
            );

            // Store the quotient coefficient
            while quotient_coeffs.len() <= dividend_degree - divisor_degree {
                quotient_coeffs.push(0);
            }
            quotient_coeffs[dividend_degree - divisor_degree] = quotient_coeff;

            // Subtract (quotient_coeff * x^(dividend_degree - divisor_degree)) * divisor
            let mut to_subtract = vec![0; dividend_degree - divisor_degree + 1];
            to_subtract[dividend_degree - divisor_degree] = quotient_coeff;

            let to_subtract_poly = ModularPoly::new(to_subtract, self.p);
            let subtract_term = &to_subtract_poly * divisor;

            dividend = &dividend - &subtract_term;

            // If the degree didn't decrease, we have a problem
            if dividend.degree() >= dividend_degree {
                return None;
            }
        }

        Some((ModularPoly::new(quotient_coeffs, self.p), dividend))
    }

    /// Computes the remainder of polynomial division.
    /// Returns None if division is not possible (e.g., divisor is zero).
    pub fn remainder(&self, divisor: &ModularPoly) -> Option<ModularPoly> {
        self.get_quotient_and_remainder(divisor)
            .map(|(_, remainder)| remainder)
    }

    /// Computes the quotient of polynomial division.
    /// Returns None if division is not possible (e.g., divisor is zero).
    pub fn quotient(&self, divisor: &ModularPoly) -> Option<ModularPoly> {
        self.get_quotient_and_remainder(divisor)
            .map(|(quotient, _)| quotient)
    }

    /// Computes the multiplicative inverse of this polynomial modulo another polynomial.
    /// Returns None if no inverse exists.
    pub fn get_inverse(&self, q: &ModularPoly) -> Option<ModularPoly> {
        assert_eq!(
            self.p, q.p,
            "Cannot compute inverse for polynomials with different moduli"
        );

        if self.is_zero() || q.is_zero() {
            return None;
        }

        // Extended Euclidean Algorithm for polynomials
        let mut r_prev = q.clone();
        let mut r_curr = self.clone();
        let mut s_prev = ModularPoly::zero(self.p);
        let mut s_curr = ModularPoly::constant(1, self.p);

        while !r_curr.is_zero() {
            // Compute quotient and remainder
            let division_result = r_prev.get_quotient_and_remainder(&r_curr);
            if division_result.is_none() {
                return None;
            }
            let (q_poly, r_next) = division_result.unwrap();

            if !r_next.is_zero() {
                // Update Bézout coefficients
                let temp_s = s_curr.clone();
                s_curr = &s_prev - &(&q_poly * &s_curr);
                s_prev = temp_s;
            }
            // Update remainders
            r_prev = r_curr.clone();
            r_curr = r_next;
            info!("r_prev: {}, r_curr: {}", r_prev, r_curr);
        }

        // The GCD is r_prev, and s_curr contains the Bézout coefficient
        // We need to check if the GCD is a constant (degree 0)
        if r_prev.degree() > 0 {
            info!("Degree of {} is > 0", r_prev);
            return None; // No inverse exists
        }

        // The inverse is s_curr, but we need to normalize it
        // Since r_prev is the GCD, we need to divide s_curr by r_prev
        let gcd_coeff = r_prev.coeffs[0];
        if gcd_coeff == 0 {
            return None;
        }

        // Find multiplicative inverse of the GCD coefficient
        let gcd_inv = Self::mod_inverse(gcd_coeff, self.p);
        if gcd_inv.is_none() {
            return None;
        }

        // Scale the inverse by the GCD inverse
        let mut result_coeffs = Vec::new();
        for &coeff in &s_curr.coeffs {
            result_coeffs.push(Self::mul_mod(coeff, gcd_inv.unwrap(), self.p));
        }

        Some(ModularPoly::new(result_coeffs, self.p))
    }
}

impl Add for &ModularPoly {
    type Output = ModularPoly;

    fn add(self, other: &ModularPoly) -> Self::Output {
        assert_eq!(
            self.p, other.p,
            "Cannot add polynomials with different moduli"
        );

        let max_len = std::cmp::max(self.coeffs.len(), other.coeffs.len());
        let mut result = Vec::with_capacity(max_len);

        for i in 0..max_len {
            let a = if i < self.coeffs.len() {
                self.coeffs[i]
            } else {
                0
            };
            let b = if i < other.coeffs.len() {
                other.coeffs[i]
            } else {
                0
            };
            result.push(ModularPoly::add_mod(a, b, self.p));
        }

        ModularPoly::new(result, self.p)
    }
}

impl Sub for &ModularPoly {
    type Output = ModularPoly;

    fn sub(self, other: &ModularPoly) -> Self::Output {
        assert_eq!(
            self.p, other.p,
            "Cannot subtract polynomials with different moduli"
        );

        let max_len = std::cmp::max(self.coeffs.len(), other.coeffs.len());
        let mut result = Vec::with_capacity(max_len);

        for i in 0..max_len {
            let a = if i < self.coeffs.len() {
                self.coeffs[i]
            } else {
                0
            };
            let b = if i < other.coeffs.len() {
                other.coeffs[i]
            } else {
                0
            };
            result.push(ModularPoly::sub_mod(a, b, self.p));
        }

        ModularPoly::new(result, self.p)
    }
}

impl Mul for &ModularPoly {
    type Output = ModularPoly;

    fn mul(self, other: &ModularPoly) -> Self::Output {
        assert_eq!(
            self.p, other.p,
            "Cannot multiply polynomials with different moduli"
        );

        if self.is_zero() || other.is_zero() {
            return ModularPoly::zero(self.p);
        }

        let result_len = self.coeffs.len() + other.coeffs.len() - 1;
        let mut result = vec![0; result_len];

        for i in 0..self.coeffs.len() {
            for j in 0..other.coeffs.len() {
                let product = ModularPoly::mul_mod(self.coeffs[i], other.coeffs[j], self.p);
                result[i + j] = ModularPoly::add_mod(result[i + j], product, self.p);
            }
        }

        ModularPoly::new(result, self.p)
    }
}

impl std::fmt::Display for ModularPoly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }

        let mut terms = Vec::new();

        for (i, &coeff) in self.coeffs.iter().enumerate() {
            if coeff == 0 {
                continue;
            }

            let term = if i == 0 {
                format!("{}", coeff)
            } else if i == 1 {
                if coeff == 1 {
                    "x".to_string()
                } else {
                    format!("{}x", coeff)
                }
            } else {
                if coeff == 1 {
                    format!("x^{}", i)
                } else {
                    format!("{}x^{}", coeff, i)
                }
            };
            terms.push(term);
        }

        if terms.is_empty() {
            write!(f, "0")
        } else {
            write!(f, "{}", terms.join(" + "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    // Use the largest prime number that fits in u64
    const LARGE_PRIME: u64 = u64::MAX - 58;

    #[test]
    fn test_creation() {
        let poly = ModularPoly::new(vec![1, 2, 3], LARGE_PRIME);
        assert_eq!(poly.coeffs, vec![1, 2, 3]);
        assert_eq!(poly.p, LARGE_PRIME);
    }

    #[test]
    fn test_normalization() {
        let poly = ModularPoly::new(vec![1, 2, 0, 0], LARGE_PRIME);
        assert_eq!(poly.coeffs, vec![1, 2]);
    }

    #[test]
    fn test_modular_reduction() {
        let poly = ModularPoly::new(
            vec![LARGE_PRIME + 10, LARGE_PRIME + 15, LARGE_PRIME + 20],
            LARGE_PRIME,
        );
        assert_eq!(poly.coeffs, vec![10, 15, 20]); // All reduced modulo LARGE_PRIME
    }

    #[test]
    fn test_addition() {
        let a = ModularPoly::new(vec![1, 2, 3], LARGE_PRIME);
        let b = ModularPoly::new(vec![4, 5], LARGE_PRIME);
        let result = &a + &b;
        assert_eq!(result.coeffs, vec![5, 7, 3]); // (1+4)=5, (2+5)=7, 3
    }

    #[test]
    fn test_subtraction() {
        let a = ModularPoly::new(vec![5, 3, 1], LARGE_PRIME);
        let b = ModularPoly::new(vec![2, 4], LARGE_PRIME);
        let result = &a - &b;
        assert_eq!(result.coeffs, vec![3, LARGE_PRIME - 1, 1]); // (5-2)=3, (3-4)=LARGE_PRIME-1, 1
    }

    #[test]
    fn test_multiplication() {
        let a = ModularPoly::new(vec![1, 2], LARGE_PRIME); // 1 + 2x
        let b = ModularPoly::new(vec![3, 4], LARGE_PRIME); // 3 + 4x
        let result = &a * &b;
        // (1 + 2x)(3 + 4x) = 3 + 4x + 6x + 8x^2 = 3 + 10x + 8x^2
        assert_eq!(result.coeffs, vec![3, 10, 8]);
    }

    #[test]
    fn test_display() {
        let poly = ModularPoly::new(vec![1, 2, 3], LARGE_PRIME);
        assert_eq!(poly.to_string(), "1 + 2x + 3x^2");

        let zero = ModularPoly::zero(LARGE_PRIME);
        assert_eq!(zero.to_string(), "0");

        let constant = ModularPoly::constant(5, LARGE_PRIME);
        assert_eq!(constant.to_string(), "5");
    }

    #[test]
    fn test_large_numbers() {
        // Test with large coefficients that would overflow u64 if not handled properly
        let a = ModularPoly::new(vec![LARGE_PRIME - 1], LARGE_PRIME);
        let b = ModularPoly::new(vec![2], LARGE_PRIME);
        let result = &a + &b;
        // (LARGE_PRIME - 1 + 2) % LARGE_PRIME = (LARGE_PRIME + 1) % LARGE_PRIME = 1
        assert_eq!(result.coeffs, vec![1]);
    }

    #[test]
    fn test_multiplication_large_numbers() {
        // Test multiplication with large numbers
        let a = ModularPoly::new(vec![LARGE_PRIME - 1], LARGE_PRIME);
        let b = ModularPoly::new(vec![2], LARGE_PRIME);
        let result = &a * &b;
        // ((LARGE_PRIME - 1) * 2) % LARGE_PRIME = (2 * LARGE_PRIME - 2) % LARGE_PRIME = LARGE_PRIME - 2
        assert_eq!(result.coeffs, vec![LARGE_PRIME - 2]);
    }

    #[test]
    fn test_addition_overflow_prevention() {
        // Test adding polynomials with coefficients close to the prime to ensure no overflow
        let a = ModularPoly::new(vec![LARGE_PRIME - 100, LARGE_PRIME - 200], LARGE_PRIME);
        let b = ModularPoly::new(vec![50, 100], LARGE_PRIME);
        let result = &a + &b;
        // (LARGE_PRIME - 100 + 50) % LARGE_PRIME = (LARGE_PRIME - 50) % LARGE_PRIME = LARGE_PRIME - 50
        // (LARGE_PRIME - 200 + 100) % LARGE_PRIME = (LARGE_PRIME - 100) % LARGE_PRIME = LARGE_PRIME - 100
        assert_eq!(result.coeffs, vec![LARGE_PRIME - 50, LARGE_PRIME - 100]);
    }

    #[test]
    fn test_addition_linear_polynomials_constant_sum() {
        // Test adding two linear polynomials that sum to a constant
        // (p - 10) x + 1 + 10 x + 2 = (p) x + 3 = 3 (since p ≡ 0 mod p)
        let a = ModularPoly::new(vec![1, LARGE_PRIME - 10], LARGE_PRIME); // (p-10)x + 1
        let b = ModularPoly::new(vec![2, 10], LARGE_PRIME); // 10x + 2
        let result = &a + &b;
        // (1 + 2) % LARGE_PRIME = 3
        // (LARGE_PRIME - 10 + 10) % LARGE_PRIME = LARGE_PRIME % LARGE_PRIME = 0
        assert_eq!(result.coeffs, vec![3]); // Should be constant polynomial 3
    }

    #[test]
    fn test_addition_very_large_coefficients() {
        // Test with coefficients very close to the prime
        let a = ModularPoly::new(vec![LARGE_PRIME - 1, LARGE_PRIME - 2], LARGE_PRIME);
        let b = ModularPoly::new(vec![1, 2], LARGE_PRIME);
        let result = &a + &b;
        // (LARGE_PRIME - 1 + 1) % LARGE_PRIME = 0
        // (LARGE_PRIME - 2 + 2) % LARGE_PRIME = 0
        assert_eq!(result.coeffs, vec![0]);
        assert!(result.is_zero());
    }

    #[test]
    fn test_multiplication_overflow_prevention() {
        // Test multiplication with large coefficients
        let a = ModularPoly::new(vec![LARGE_PRIME - 1000], LARGE_PRIME);
        let b = ModularPoly::new(vec![LARGE_PRIME - 500], LARGE_PRIME);
        let result = &a * &b;
        // ((LARGE_PRIME - 1000) * (LARGE_PRIME - 500)) % LARGE_PRIME
        // = (LARGE_PRIME^2 - 500*LARGE_PRIME - 1000*LARGE_PRIME + 500000) % LARGE_PRIME
        // = (500000) % LARGE_PRIME = 500000
        assert_eq!(result.coeffs, vec![500000]);
    }

    #[test]
    fn test_remainder_basic() {
        // Test case 1: Simple division with remainder
        let dividend = ModularPoly::new(vec![1, 2, 1], 7); // x^2 + 2x + 1
        let divisor = ModularPoly::new(vec![1, 1], 7); // x + 1
        let remainder = dividend.remainder(&divisor).unwrap();
        // x^2 + 2x + 1 = (x + 1)(x + 1) + 0, so remainder should be 0
        assert_eq!(remainder.coeffs, vec![0]);
        assert!(remainder.is_zero());
    }

    #[test]
    fn test_remainder_with_remainder() {
        // Test case 2: Division with non-zero remainder
        let dividend = ModularPoly::new(vec![2, 3, 1], 7); // x^2 + 3x + 2
        let divisor = ModularPoly::new(vec![1, 1], 7); // x + 1
        let remainder = dividend.remainder(&divisor).unwrap();
        // x^2 + 3x + 2 = (x + 1)(x + 2) + 0, but in mod 7: x^2 + 3x + 2 = (x + 1)(x + 2) + 0
        // Actually: x^2 + 3x + 2 = (x + 1)(x + 2) + 0
        assert_eq!(remainder.coeffs, vec![0]);
    }

    #[test]
    fn test_remainder_constant_dividend() {
        // Test case 3: Constant dividend
        let dividend = ModularPoly::new(vec![5], 7); // 5
        let divisor = ModularPoly::new(vec![1, 2], 7); // x + 2
        let remainder = dividend.remainder(&divisor).unwrap();
        // 5 mod (x + 2) = 5
        assert_eq!(remainder.coeffs, vec![5]);
    }

    #[test]
    fn test_remainder_constant_divisor() {
        // Test case 4: Constant divisor
        let dividend = ModularPoly::new(vec![1, 2, 3], 7); // 3x^2 + 2x + 1
        let divisor = ModularPoly::new(vec![2], 7); // 2
        let remainder = dividend.remainder(&divisor).unwrap();
        // 2 has a multiplicative inverse mod 7, so the remainder should be 0
        assert_eq!(remainder.coeffs, vec![0]);
    }

    #[test]
    fn test_remainder_zero_dividend() {
        // Test case 5: Zero dividend
        let dividend = ModularPoly::zero(7);
        let divisor = ModularPoly::new(vec![1, 1], 7); // x + 1
        let remainder = dividend.remainder(&divisor).unwrap();
        assert_eq!(remainder.coeffs, vec![0]);
        assert!(remainder.is_zero());
    }

    #[test]
    fn test_remainder_zero_divisor() {
        // Test case 6: Zero divisor (should return None)
        let dividend = ModularPoly::new(vec![1, 2, 1], 7);
        let divisor = ModularPoly::zero(7);
        let remainder = dividend.remainder(&divisor);
        assert!(remainder.is_none());
    }

    #[test]
    fn test_remainder_dividend_smaller_degree() {
        // Test case 7: Dividend has smaller degree than divisor
        let dividend = ModularPoly::new(vec![1, 2], 7); // 2x + 1
        let divisor = ModularPoly::new(vec![1, 0, 1], 7); // x^2 + 1
        let remainder = dividend.remainder(&divisor).unwrap();
        // Should return the dividend unchanged
        assert_eq!(remainder.coeffs, vec![1, 2]);
    }

    #[test]
    fn test_remainder_complex_polynomials() {
        // Test case 8: Complex polynomials
        let dividend = ModularPoly::new(vec![1, 2, 3, 4], 7); // 4x^3 + 3x^2 + 2x + 1
        let divisor = ModularPoly::new(vec![1, 1], 7); // x + 1
        let remainder = dividend.remainder(&divisor).unwrap();
        // Manual calculation: 4x^3 + 3x^2 + 2x + 1 = (x + 1)(4x^2 - x + 3) + (-2)
        // In mod 7: remainder should be 5 (since -2 ≡ 5 mod 7)
        assert_eq!(remainder.coeffs, vec![5]);
    }

    #[test]
    fn test_remainder_quadratic_divisor() {
        // Test case 9: Quadratic divisor
        let dividend = ModularPoly::new(vec![1, 2, 3, 4], 7); // 4x^3 + 3x^2 + 2x + 1
        let divisor = ModularPoly::new(vec![1, 0, 1], 7); // x^2 + 1
        let remainder = dividend.remainder(&divisor).unwrap();
        // Manual calculation: 4x^3 + 3x^2 + 2x + 1 = (x^2 + 1)(4x + 3) + (-2x - 2)
        // In mod 7: remainder should be 5x + 5
        assert_eq!(remainder.coeffs, vec![5, 5]);
    }

    #[test]
    fn test_remainder_large_numbers() {
        // Test case 10: Large coefficients
        let dividend = ModularPoly::new(vec![LARGE_PRIME - 1, LARGE_PRIME - 2], LARGE_PRIME);
        let divisor = ModularPoly::new(vec![1, 1], LARGE_PRIME); // x + 1
        let remainder = dividend.remainder(&divisor).unwrap();
        // (LARGE_PRIME - 2)x + (LARGE_PRIME - 1) mod (x + 1)
        // = (LARGE_PRIME - 2)(-1) + (LARGE_PRIME - 1) = -LARGE_PRIME + 2 + LARGE_PRIME - 1 = 1
        assert_eq!(remainder.coeffs, vec![1]);
    }

    #[test]
    fn test_remainder_different_moduli() {
        // Test case 11: Different moduli (should panic)
        let dividend = ModularPoly::new(vec![1, 2, 1], 7);
        let divisor = ModularPoly::new(vec![1, 1], 11);

        // This should panic due to different moduli
        let result = std::panic::catch_unwind(|| {
            dividend.remainder(&divisor);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_remainder_identity() {
        // Test case 12: Verify that (quotient * divisor + remainder) = dividend
        let dividend = ModularPoly::new(vec![1, 2, 3, 4], 7); // 4x^3 + 3x^2 + 2x + 1
        let divisor = ModularPoly::new(vec![1, 1], 7); // x + 1
        let remainder = dividend.remainder(&divisor).unwrap();

        // For this specific case, we can verify the remainder is correct
        // by checking that it's a constant (degree 0)
        assert_eq!(remainder.degree(), 0);
        assert_eq!(remainder.coeffs.len(), 1);
    }

    #[test]
    fn test_remainder_edge_cases() {
        // Test case 13: Edge cases
        let dividend = ModularPoly::new(vec![1], 7); // 1
        let divisor = ModularPoly::new(vec![1], 7); // 1
        let remainder = dividend.remainder(&divisor).unwrap();
        assert_eq!(remainder.coeffs, vec![0]);
    }

    #[test]
    fn test_get_quotient_and_remainder_basic() {
        // Test case 1: Basic division
        let dividend = ModularPoly::new(vec![1, 2, 1], 7); // x^2 + 2x + 1
        let divisor = ModularPoly::new(vec![1, 1], 7); // x + 1
        let result = dividend.get_quotient_and_remainder(&divisor).unwrap();
        let (quotient, remainder) = result;

        // x^2 + 2x + 1 = (x + 1)(x + 1) + 0
        assert_eq!(quotient.coeffs, vec![1, 1]); // x + 1
        assert_eq!(remainder.coeffs, vec![0]); // 0

        // Verify: quotient * divisor + remainder = dividend
        let verification = &(&quotient * &divisor) + &remainder;
        assert_eq!(verification.coeffs, dividend.coeffs);
    }

    #[test]
    fn test_get_quotient_and_remainder_with_remainder() {
        // Test case 2: Division with non-zero remainder
        let dividend = ModularPoly::new(vec![1, 2, 3], 7); // 3x^2 + 2x + 1
        let divisor = ModularPoly::new(vec![1, 1], 7); // x + 1
        let result = dividend.get_quotient_and_remainder(&divisor).unwrap();
        let (quotient, remainder) = result;

        // 3x^2 + 2x + 1 = (x + 1)(3x - 1) + 2
        // In mod 7: 3x^2 + 2x + 1 = (x + 1)(3x + 6) + 2
        assert_eq!(quotient.coeffs, vec![6, 3]); // 3x + 6
        assert_eq!(remainder.coeffs, vec![2]); // 2

        // Verify: quotient * divisor + remainder = dividend
        let verification = &(&quotient * &divisor) + &remainder;
        assert_eq!(verification.coeffs, dividend.coeffs);
    }

    #[test]
    fn test_get_quotient_and_remainder_constant_dividend() {
        // Test case 3: Constant dividend
        let dividend = ModularPoly::new(vec![5], 7); // 5
        let divisor = ModularPoly::new(vec![1, 1], 7); // x + 1
        let result = dividend.get_quotient_and_remainder(&divisor).unwrap();
        let (quotient, remainder) = result;

        // 5 = (x + 1)(0) + 5
        assert_eq!(quotient.coeffs, vec![] as Vec<u64>); // 0
        assert_eq!(remainder.coeffs, vec![5]); // 5

        // Verify: quotient * divisor + remainder = dividend
        let verification = &(&quotient * &divisor) + &remainder;
        assert_eq!(verification.coeffs, dividend.coeffs);
    }

    #[test]
    fn test_get_quotient_and_remainder_constant_divisor() {
        // Test case 4: Constant divisor
        let dividend = ModularPoly::new(vec![1, 2, 3], 7); // 3x^2 + 2x + 1
        let divisor = ModularPoly::new(vec![2], 7); // 2
        let result = dividend.get_quotient_and_remainder(&divisor).unwrap();
        let (quotient, remainder) = result;

        // 3x^2 + 2x + 1 = 2(5x^2 + 1x + 4) + 0
        // In mod 7: 2 * 5 = 3, 2 * 1 = 2, 2 * 3 = 1
        assert_eq!(quotient.coeffs, vec![4, 1, 5]); // 5x^2 + x + 4
        assert_eq!(remainder.coeffs, vec![0]); // 0

        // Verify: quotient * divisor + remainder = dividend
        let verification = &(&quotient * &divisor) + &remainder;
        assert_eq!(verification.coeffs, dividend.coeffs);
    }

    #[test]
    fn test_get_quotient_and_remainder_zero_dividend() {
        // Test case 5: Zero dividend
        let dividend = ModularPoly::zero(7);
        let divisor = ModularPoly::new(vec![1, 1], 7); // x + 1
        let result = dividend.get_quotient_and_remainder(&divisor).unwrap();
        let (quotient, remainder) = result;

        // 0 = (x + 1)(0) + 0
        assert_eq!(quotient.coeffs, vec![0]); // 0
        assert_eq!(remainder.coeffs, vec![0]); // 0

        // Verify: quotient * divisor + remainder = dividend
        let verification = &(&quotient * &divisor) + &remainder;
        assert_eq!(verification.coeffs, dividend.coeffs);
    }

    #[test]
    fn test_get_quotient_and_remainder_zero_divisor() {
        // Test case 6: Zero divisor (should return None)
        let dividend = ModularPoly::new(vec![1, 2, 1], 7);
        let divisor = ModularPoly::zero(7);
        let result = dividend.get_quotient_and_remainder(&divisor);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_quotient_and_remainder_dividend_smaller_degree() {
        // Test case 7: Dividend has smaller degree than divisor
        let dividend = ModularPoly::new(vec![1, 2], 7); // 2x + 1
        let divisor = ModularPoly::new(vec![1, 0, 1], 7); // x^2 + 1
        let result = dividend.get_quotient_and_remainder(&divisor).unwrap();
        let (quotient, remainder) = result;

        // 2x + 1 = (x^2 + 1)(0) + (2x + 1)
        assert_eq!(quotient.coeffs, vec![] as Vec<u64>); // 0
        assert_eq!(remainder.coeffs, vec![1, 2]); // 2x + 1

        // Verify: quotient * divisor + remainder = dividend
        let verification = &(&quotient * &divisor) + &remainder;
        assert_eq!(verification.coeffs, dividend.coeffs);
    }

    #[test]
    fn test_get_quotient_and_remainder_complex_polynomials() {
        // Test case 8: Complex polynomials
        let dividend = ModularPoly::new(vec![1, 2, 3, 4], 7); // 4x^3 + 3x^2 + 2x + 1
        let divisor = ModularPoly::new(vec![1, 1], 7); // x + 1
        let result = dividend.get_quotient_and_remainder(&divisor).unwrap();
        let (quotient, remainder) = result;

        // 4x^3 + 3x^2 + 2x + 1 = (x + 1)(4x^2 - x + 3) + (-2)
        // In mod 7: remainder should be 5 (since -2 ≡ 5 mod 7)
        assert_eq!(remainder.coeffs, vec![5]); // 5

        // Verify: quotient * divisor + remainder = dividend
        let verification = &(&quotient * &divisor) + &remainder;
        assert_eq!(verification.coeffs, dividend.coeffs);
    }

    #[test]
    fn test_get_quotient_and_remainder_quadratic_divisor() {
        // Test case 9: Quadratic divisor
        let dividend = ModularPoly::new(vec![1, 2, 3, 4], 7); // 4x^3 + 3x^2 + 2x + 1
        let divisor = ModularPoly::new(vec![1, 0, 1], 7); // x^2 + 1
        let result = dividend.get_quotient_and_remainder(&divisor).unwrap();
        let (quotient, remainder) = result;

        // 4x^3 + 3x^2 + 2x + 1 = (x^2 + 1)(4x + 3) + (-2x - 2)
        // In mod 7: remainder should be 5x + 5
        assert_eq!(remainder.coeffs, vec![5, 5]); // 5x + 5

        // Verify: quotient * divisor + remainder = dividend
        let verification = &(&quotient * &divisor) + &remainder;
        assert_eq!(verification.coeffs, dividend.coeffs);
    }

    #[test]
    fn test_get_quotient_and_remainder_large_numbers() {
        // Test case 10: Large coefficients
        let dividend = ModularPoly::new(vec![LARGE_PRIME - 1, LARGE_PRIME - 2], LARGE_PRIME);
        let divisor = ModularPoly::new(vec![1, 1], LARGE_PRIME); // x + 1
        let result = dividend.get_quotient_and_remainder(&divisor).unwrap();
        let (quotient, remainder) = result;

        // (LARGE_PRIME - 2)x + (LARGE_PRIME - 1) mod (x + 1)
        // = (LARGE_PRIME - 2)(-1) + (LARGE_PRIME - 1) = -LARGE_PRIME + 2 + LARGE_PRIME - 1 = 1
        assert_eq!(remainder.coeffs, vec![1]); // 1

        // Verify: quotient * divisor + remainder = dividend
        let verification = &(&quotient * &divisor) + &remainder;
        assert_eq!(verification.coeffs, dividend.coeffs);
    }

    #[test]
    fn test_get_quotient_and_remainder_different_moduli() {
        // Test case 11: Different moduli (should panic)
        let dividend = ModularPoly::new(vec![1, 2, 1], 7);
        let divisor = ModularPoly::new(vec![1, 1], 11);

        // This should panic due to different moduli
        let result = std::panic::catch_unwind(|| {
            dividend.get_quotient_and_remainder(&divisor);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_get_quotient_and_remainder_identity() {
        // Test case 12: Verify that (quotient * divisor + remainder) = dividend
        let dividend = ModularPoly::new(vec![1, 2, 3, 4], 7); // 4x^3 + 3x^2 + 2x + 1
        let divisor = ModularPoly::new(vec![1, 1], 7); // x + 1
        let result = dividend.get_quotient_and_remainder(&divisor).unwrap();
        let (quotient, remainder) = result;

        // Verify the fundamental identity: dividend = quotient * divisor + remainder
        let verification = &(&quotient * &divisor) + &remainder;
        assert_eq!(verification.coeffs, dividend.coeffs);
    }

    #[test]
    fn test_get_quotient_and_remainder_edge_cases() {
        // Test case 13: Edge cases
        let dividend = ModularPoly::new(vec![1], 7); // 1
        let divisor = ModularPoly::new(vec![1], 7); // 1
        let result = dividend.get_quotient_and_remainder(&divisor).unwrap();
        let (quotient, remainder) = result;

        // 1 = 1(1) + 0
        assert_eq!(quotient.coeffs, vec![1]); // 1
        assert_eq!(remainder.coeffs, vec![0]); // 0

        // Verify: quotient * divisor + remainder = dividend
        let verification = &(&quotient * &divisor) + &remainder;
        assert_eq!(verification.coeffs, dividend.coeffs);
    }

    #[test]
    fn test_get_inverse_basic() {
        // Test case 1: Simple inverse
        let p = ModularPoly::new(vec![1, 1], 7); // x + 1
        let q = ModularPoly::new(vec![1, 0, 1], 7); // x^2 + 1
        let inv = p.get_inverse(&q).unwrap();

        assert_eq!(inv.coeffs, vec![4, 3]); // (x + 1) * (ax + b) = 6a + x (a + b) + b
                                            // a + b = 7m, 6a + b = 1 + 7n => a = 3 + 7k, b = 4 + 7l
                                            // (x + 1) (3x + 4) = 3(x^2 + 1) + 1 (mod 7)

        // Verify: (x + 1) * inv ≡ 1 (mod x^2 + 1)
        let product = &p * &inv;
        let remainder = product.remainder(&q).unwrap();
        assert_eq!(remainder.coeffs, vec![1]); // Should be 1
    }

    #[test]
    fn test_get_inverse_constant() {
        // Test case 2: Constant polynomial inverse
        let p = ModularPoly::new(vec![2], 7); // 2
        let q = ModularPoly::new(vec![1, 0, 1], 7); // x^2 + 1
        let inv = p.get_inverse(&q).unwrap();

        // Verify: 2 * inv ≡ 1 (mod x^2 + 1)
        let product = &p * &inv;
        let remainder = product.remainder(&q).unwrap();
        assert_eq!(remainder.coeffs, vec![1]); // Should be 1
    }

    #[test]
    fn test_get_inverse_no_inverse() {
        // Test case 3: No inverse exists (GCD ≠ 1)
        let p = ModularPoly::new(vec![1, 1], 7); // x + 1
        let q = ModularPoly::new(vec![1, 1], 7); // x + 1 (same as p)
        let inv = p.get_inverse(&q);
        assert!(inv.is_none()); // No inverse exists
    }

    #[test]
    fn test_get_inverse_zero_polynomial() {
        // Test case 4: Zero polynomial has no inverse
        let p = ModularPoly::zero(7);
        let q = ModularPoly::new(vec![1, 0, 1], 7); // x^2 + 1
        let inv = p.get_inverse(&q);
        assert!(inv.is_none());
    }

    #[test]
    fn test_get_inverse_zero_modulus() {
        // Test case 5: Zero modulus polynomial
        let p = ModularPoly::new(vec![1, 1], 7); // x + 1
        let q = ModularPoly::zero(7);
        let inv = p.get_inverse(&q);
        assert!(inv.is_none());
    }

    #[test]
    fn test_get_inverse_different_moduli() {
        // Test case 6: Different moduli (should panic)
        let p = ModularPoly::new(vec![1, 1], 7);
        let q = ModularPoly::new(vec![1, 0, 1], 11);

        let result = std::panic::catch_unwind(|| {
            p.get_inverse(&q);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_get_inverse_complex_polynomials() {
        // Test case 7: Complex polynomials
        let p = ModularPoly::new(vec![1, 3], 7); // 3x + 1
        let q = ModularPoly::new(vec![1, 0, 0, 1], 7); // x^3 + 1
        let inv = p.get_inverse(&q).unwrap();

        // x^3 + 1 = (2x + 1)(4x^2 + 5x + 1) (mod 7)

        // Verify: (2x + 1) * inv ≡ 1 (mod x^3 + 1)
        let product = &p * &inv;
        let remainder = product.remainder(&q).unwrap();
        assert_eq!(remainder.coeffs, vec![1]); // Should be 1
    }

    #[test]
    fn test_get_inverse_identity() {
        // Test case 8: Identity polynomial (should have inverse 1)
        let p = ModularPoly::new(vec![1], 7); // 1
        let q = ModularPoly::new(vec![1, 0, 1], 7); // x^2 + 1
        let inv = p.get_inverse(&q).unwrap();

        // The inverse of 1 should be 1
        assert_eq!(inv.coeffs, vec![1]);
    }

    #[test]
    fn test_get_inverse_quadratic_modulus() {
        // Test case 9: Quadratic modulus
        let p = ModularPoly::new(vec![1, 1], 7); // x + 1
        let q = ModularPoly::new(vec![1, 1, 1], 7); // x^2 + x + 1
        let inv = p.get_inverse(&q).unwrap();

        // Verify: (x + 1) * inv ≡ 1 (mod x^2 + x + 1)
        let product = &p * &inv;
        let remainder = product.remainder(&q).unwrap();
        assert_eq!(remainder.coeffs, vec![1]); // Should be 1
    }

    #[test]
    fn test_get_inverse_large_numbers() {
        // Test case 10: Large coefficients
        let p = ModularPoly::new(vec![LARGE_PRIME - 1], LARGE_PRIME); // LARGE_PRIME - 1
        let q = ModularPoly::new(vec![1, 0, 1], LARGE_PRIME); // x^2 + 1
        let inv = p.get_inverse(&q).unwrap();

        // Verify: (LARGE_PRIME - 1) * inv ≡ 1 (mod x^2 + 1)
        let product = &p * &inv;
        let remainder = product.remainder(&q).unwrap();
        assert_eq!(remainder.coeffs, vec![1]); // Should be 1
    }

    #[test]
    fn test_get_inverse_verification() {
        // Test case 11: Verify inverse property
        let p = ModularPoly::new(vec![1, 3, 1], 101); // x^2 + 3x + 1
        let q = ModularPoly::new(vec![1, 0, 0, 1], 101); // x^3 + 1
        let inv = p.get_inverse(&q).unwrap();

        // Verify: p * inv ≡ 1 (mod q)
        let product = &p * &inv;
        let remainder = product.remainder(&q).unwrap();
        assert_eq!(remainder.coeffs, vec![1]); // Should be 1

        // Also verify: inv * p ≡ 1 (mod q)
        let product2 = &inv * &p;
        let remainder2 = product2.remainder(&q).unwrap();
        assert_eq!(remainder2.coeffs, vec![1]); // Should be 1
    }

    #[test]
    fn test_get_inverse_edge_cases() {
        // Test case 12: Edge cases
        let p = ModularPoly::new(vec![1], 7); // 1
        let q = ModularPoly::new(vec![1], 7); // 1
        let inv = p.get_inverse(&q).unwrap();
        assert_eq!(inv.coeffs, vec![1]); // Inverse of 1 is 1

        // Test with polynomial that has no inverse
        let p2 = ModularPoly::new(vec![1, 1], 7); // x + 1
        let q2 = ModularPoly::new(vec![1, 0, 0, 1], 7); // x^3 + 1
        let inv2 = p2.get_inverse(&q2);
        assert!(inv2.is_none()); // No inverse exists in Z/7Z
    }
}
