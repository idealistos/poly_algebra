use std::{
    fmt::{Display, Formatter, Result},
    ops,
};

use float_next_after::NextAfter;

pub const ZERO_FINT: FInt = FInt(0.0, 0.0);

trait NextBeforeOrAfter {
    fn inc(self) -> f64;
    fn dec(self) -> f64;
}

impl NextBeforeOrAfter for f64 {
    fn inc(self) -> f64 {
        self.next_after(std::f64::INFINITY)
    }

    fn dec(self) -> f64 {
        self.next_after(std::f64::NEG_INFINITY)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FInt(f64, f64);
impl Display for FInt {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mean = self.midpoint();
        if self.1 - self.0 > 1e-8 && self.1 - self.0 > mean.abs() * 1e-8 {
            return write!(f, "{:.3} to {:.3}", self.0, self.1);
        }
        let default_str = format!("{}", mean);

        // If the default string is short enough, use it
        if default_str.len() <= 12 {
            return write!(f, "{}", default_str);
        }

        // Try to find a shorter representation that's still within the interval
        for precision in 0..=12 {
            let formatted = format!("{:.precision$}", mean);
            if let Ok(value) = formatted.parse::<f64>() {
                if value >= self.0 && value <= self.1 {
                    return write!(f, "{}", formatted);
                }
            }
        }

        // If no shorter representation found, use the default
        write!(f, "{}", default_str)
    }
}
impl FInt {
    pub const fn zero() -> Self {
        Self(0.0, 0.0)
    }

    pub fn new(value: f64) -> FInt {
        Self::new_with_bounds(value.dec(), value.inc())
    }

    pub fn new_with_delta(value: f64, delta: f64) -> FInt {
        if delta <= 0.0 {
            panic!("Wrong interval! delta {} <= 0", delta);
        }
        Self::new_with_bounds(value - delta, value + delta)
    }

    pub fn new_with_bounds(lower: f64, upper: f64) -> FInt {
        if lower > upper {
            panic!("Wrong interval! {} > {}", lower, upper);
        }
        Self(lower, upper)
    }

    pub fn negate(self) -> FInt {
        Self::new_with_bounds(-self.1, -self.0)
    }

    pub fn inverse(self) -> FInt {
        if self.0.is_nan() || self.1.is_nan() || (self.0 <= 0.0 && self.1 >= 0.0) {
            Self::new_with_bounds(f64::NAN, f64::NAN)
        } else if self.0 > 0.0 {
            Self::new_with_bounds((1.0 / self.1).dec(), (1.0 / self.0).inc())
        } else {
            let x = self.negate();
            // if !(x.0 <= 0.0 && x.1 >= 0.0) && !(x.0 > 0.0) {
            //     println!("Infinite recursion in inverse() for {:?}", self);
            // }
            x.inverse().negate()
        }
    }

    pub fn sqr(&self) -> FInt {
        *self * *self
    }

    pub fn sqrt(&self) -> FInt {
        if self.0 < 0.0 {
            Self::new_with_bounds(f64::NAN, f64::NAN)
        } else {
            Self::new_with_bounds(self.0.sqrt().dec(), self.1.sqrt().inc())
        }
    }

    pub fn always_positive(&self) -> bool {
        self.0 > 0.0
    }

    pub fn midpoint(&self) -> f64 {
        0.5 * (self.0 + self.1)
    }

    pub fn abs_bound(&self) -> f64 {
        f64::max(self.0.abs(), self.1.abs())
    }

    pub fn well_formed(&self) -> bool {
        !self.0.is_nan() && !self.1.is_nan()
    }

    pub fn precise(&self) -> bool {
        self.well_formed() && self.1 - self.0 < f64::max(1e-5 * self.0.abs(), 1e-10)
    }

    pub fn almost_equals(&self, x: FInt) -> bool {
        if (self.0 - x.0).abs() > 0.001
            && x.0.abs() < 1000.0
            && self.1 - self.0 < 0.001
            && x.1 - x.0 < 0.001
        {
            return false;
        }
        let m1 = self.midpoint();
        let m2 = x.midpoint();
        let mut delta = f64::max(0.001, f64::max(3.0 * (self.1 - self.0), 3.0 * (x.1 - x.0)));
        let max_abs = f64::max(
            f64::max(self.0.abs(), self.1.abs()),
            f64::max(x.0.abs(), x.1.abs()),
        );
        if max_abs > 1000.0 {
            delta *= max_abs / 1000.0;
        }
        (m1 - m2).abs() < delta
    }

    pub fn get_subinterval(
        x_interval: FInt,
        y_interval: FInt,
        rect: crate::poly_draw::Rectangle,
        sub_rect: crate::poly_draw::Rectangle,
    ) -> (FInt, FInt) {
        let x_ratio_0 = (sub_rect.x0 - rect.x0) as f64 / (rect.x1 - rect.x0) as f64;
        let x_ratio_1 = (sub_rect.x1 - rect.x0) as f64 / (rect.x1 - rect.x0) as f64;
        let y_ratio_0 = (sub_rect.y0 - rect.y0) as f64 / (rect.y1 - rect.y0) as f64;
        let y_ratio_1 = (sub_rect.y1 - rect.y0) as f64 / (rect.y1 - rect.y0) as f64;

        let sub_x = FInt::new_with_bounds(
            x_interval.0 + x_ratio_0 * (x_interval.1 - x_interval.0),
            x_interval.0 + x_ratio_1 * (x_interval.1 - x_interval.0),
        );
        let sub_y = FInt::new_with_bounds(
            y_interval.0 + y_ratio_0 * (y_interval.1 - y_interval.0),
            y_interval.0 + y_ratio_1 * (y_interval.1 - y_interval.0),
        );

        (sub_x, sub_y)
    }
}

impl ops::Add<FInt> for FInt {
    type Output = FInt;

    fn add(self, x: FInt) -> FInt {
        Self::new_with_bounds((self.0 + x.0).dec(), (self.1 + x.1).inc())
    }
}

impl ops::Sub<FInt> for FInt {
    type Output = FInt;

    fn sub(self, x: FInt) -> FInt {
        Self::new_with_bounds((self.0 - x.1).dec(), (self.1 - x.0).inc())
    }
}

impl ops::Mul<FInt> for FInt {
    type Output = FInt;

    fn mul(self, x: FInt) -> FInt {
        if self.0 >= 0.0 {
            if x.0 >= 0.0 {
                return Self::new_with_bounds((self.0 * x.0).dec(), (self.1 * x.1).inc());
            } else if x.1 <= 0.0 {
                return Self::new_with_bounds((self.1 * x.0).dec(), (self.0 * x.1).dec());
            }
        } else if self.1 <= 0.0 {
            if x.0 >= 0.0 {
                return Self::new_with_bounds((self.0 * x.1).dec(), (self.1 * x.0).dec());
            } else if x.1 <= 0.0 {
                return Self::new_with_bounds((self.1 * x.1).dec(), (self.0 * x.0).dec());
            }
        }
        let v00 = self.0 * x.0;
        let v01 = self.0 * x.1;
        let v10 = self.1 * x.0;
        let v11 = self.1 * x.1;
        Self::new_with_bounds(
            v00.min(v01).min(v10).min(v11).dec(),
            v00.max(v01).max(v10).max(v11).inc(),
        )
    }
}

impl ops::Div<FInt> for FInt {
    type Output = FInt;

    fn div(self, x: FInt) -> FInt {
        self * x.inverse()
    }
}

impl PartialEq for FInt {
    fn eq(&self, x: &FInt) -> bool {
        !(x.1 < self.0 || x.0 > self.1)
    }
}
impl Eq for FInt {}

#[cfg(test)]
mod tests {
    use crate::poly_draw::Rectangle;

    use super::*;

    #[test]
    fn test_add() {
        let result = FInt::new(1.0) + FInt::new(1.0);
        assert!((result.0 - 2.0).abs() < 1e-14);
        assert!(result.1 - result.0 > 0.0);
    }

    #[test]
    fn test_complex_operation() {
        let result = FInt::new(1.2) / (FInt::new(1.00001) - FInt::new(0.5) * FInt::new(2.0))
            - FInt::new(120000.0);
        assert_eq!(
            format!("{:?}", result),
            "FInt(-7.447568350471557e-6, 9.872077498584987e-6)"
        );
    }

    #[test]
    fn test_fint_display() {
        // Test exact values
        assert_eq!(format!("{}", FInt::new(1.0)), "1");
        assert_eq!(format!("{}", FInt::new(1.5)), "1.5");
        assert_eq!(format!("{}", FInt::new(-2.0)), "-2");

        // Test values with numerical noise
        let noisy = FInt::new(1.9999999999999998);
        assert_eq!(format!("{}", noisy), "2");

        // Test values that need some precision
        let precise = FInt::new(1.2345678901234567);
        assert_eq!(format!("{}", precise), "1.2345678901234567");

        // Test values that are exactly representable with less precision
        let exact = FInt::new(1.2345678901234567);
        assert_eq!(format!("{}", exact), "1.2345678901234567");
    }

    #[test]
    fn test_subinterval() {
        let x_region = FInt::new_with_bounds(-1.0, 1.0);
        let y_region = FInt::new_with_bounds(-1.0, 1.0);
        let rect = Rectangle::new(0, 0, 4, 4);

        // Test top-left subregion
        let sub_rect = Rectangle::new(0, 0, 2, 2);
        let (sub_x, sub_y) = FInt::get_subinterval(x_region, y_region, rect, sub_rect);
        assert_eq!(format!("{:?}", sub_x), "FInt(-1.0, 0.0)");
        assert_eq!(format!("{:?}", sub_y), "FInt(-1.0, 0.0)");

        // Test top-right subregion
        let sub_rect = Rectangle::new(2, 0, 4, 2);
        let (sub_x, sub_y) = FInt::get_subinterval(x_region, y_region, rect, sub_rect);
        assert_eq!(format!("{:?}", sub_x), "FInt(0.0, 1.0)");
        assert_eq!(format!("{:?}", sub_y), "FInt(-1.0, 0.0)");

        // Test bottom-left subregion
        let sub_rect = Rectangle::new(0, 2, 2, 4);
        let (sub_x, sub_y) = FInt::get_subinterval(x_region, y_region, rect, sub_rect);
        assert_eq!(format!("{:?}", sub_x), "FInt(-1.0, 0.0)");
        assert_eq!(format!("{:?}", sub_y), "FInt(0.0, 1.0)");

        // Test bottom-right subregion
        let sub_rect = Rectangle::new(2, 2, 4, 4);
        let (sub_x, sub_y) = FInt::get_subinterval(x_region, y_region, rect, sub_rect);
        assert_eq!(format!("{:?}", sub_x), "FInt(0.0, 1.0)");
        assert_eq!(format!("{:?}", sub_y), "FInt(0.0, 1.0)");
    }
}
