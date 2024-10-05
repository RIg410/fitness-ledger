use std::{
    fmt::{Debug, Display},
    iter::Sum,
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

const DECIMALS: u8 = 2;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(i64);

impl Decimal {
    pub fn int(value: i64) -> Decimal {
        Decimal(value * 10i64.pow(DECIMALS as u32))
    }

    pub fn is_negative(&self) -> bool {
        self.0 < 0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn zero() -> Decimal {
        Decimal::int(0)
    }

    pub fn inner(&self) -> i64 {
        self.0
    }
}

impl Debug for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.0 as f64 / 10i64.pow(DECIMALS as u32) as f64;
        write!(f, "{:.2}", value)
    }
}

impl Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.0 as f64 / 10i64.pow(DECIMALS as u32) as f64;
        write!(f, "{:.2}", value)
    }
}

impl From<f64> for Decimal {
    fn from(value: f64) -> Self {
        Decimal((value * 10f64.powi(DECIMALS as i32)) as i64)
    }
}

impl TryFrom<&str> for Decimal {
    type Error = ParseDecimalError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let val = value.parse::<f64>().map_err(|_| ParseDecimalError)?;
        Ok(Decimal((val * 10f64.powi(DECIMALS as i32)) as i64))
    }
}

impl FromStr for Decimal {
    type Err = ParseDecimalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Decimal::try_from(s)
    }
}

impl From<u32> for Decimal {
    fn from(value: u32) -> Self {
        Decimal::int(value as i64)
    }
}

impl std::ops::AddAssign for Decimal {
    fn add_assign(&mut self, other: Decimal) {
        self.0 += other.0;
    }
}

impl std::ops::SubAssign for Decimal {
    fn sub_assign(&mut self, other: Decimal) {
        self.0 -= other.0;
    }
}

impl std::ops::MulAssign for Decimal {
    fn mul_assign(&mut self, other: Decimal) {
        self.0 = (self.0 * other.0) / 10i64.pow(DECIMALS as u32);
    }
}

impl std::ops::DivAssign for Decimal {
    fn div_assign(&mut self, other: Decimal) {
        self.0 = (self.0 * 10i64.pow(DECIMALS as u32)) / other.0;
    }
}

impl std::ops::Add for Decimal {
    type Output = Decimal;

    fn add(self, other: Decimal) -> Decimal {
        Decimal(self.0 + other.0)
    }
}

impl std::ops::Sub for Decimal {
    type Output = Decimal;

    fn sub(self, other: Decimal) -> Decimal {
        Decimal(self.0 - other.0)
    }
}

impl std::ops::Mul for Decimal {
    type Output = Decimal;

    fn mul(self, other: Decimal) -> Decimal {
        Decimal((self.0 * other.0) / 10i64.pow(DECIMALS as u32))
    }
}

impl std::ops::Div for Decimal {
    type Output = Decimal;

    fn div(self, other: Decimal) -> Decimal {
        Decimal((self.0 * 10i64.pow(DECIMALS as u32)) / other.0)
    }
}

impl Sum for Decimal {
    fn sum<I: Iterator<Item = Decimal>>(iter: I) -> Decimal {
        iter.fold(Decimal::zero(), |acc, x| acc + x)
    }
}

#[derive(Debug)]
pub struct ParseDecimalError;

impl std::fmt::Display for ParseDecimalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse decimal value")
    }
}

impl std::error::Error for ParseDecimalError {}

impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.0)
    }
}

impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i64::deserialize(deserializer)?;
        Ok(Decimal(value))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let decimal = Decimal::int(123456);
        assert_eq!("123456.00", format!("{}", decimal));

        let decimal = Decimal::int(-123456);
        assert_eq!("-123456.00", format!("{}", decimal));
        let decimal = Decimal::int(0);
        assert_eq!("0.00", format!("{}", decimal));
    }

    #[test]
    fn test_from_f64_display() {
        let decimal = Decimal::from(123456.78);
        assert_eq!("123456.78", format!("{}", decimal));

        let decimal = Decimal::from(-123456.78);
        assert_eq!("-123456.78", format!("{}", decimal));

        let decimal = Decimal::from(123456.0);
        assert_eq!("123456.00", format!("{}", decimal));

        let decimal = Decimal::from(-123456.0);
        assert_eq!("-123456.00", format!("{}", decimal));

        let decimal = Decimal::from(0.0);
        assert_eq!("0.00", format!("{}", decimal));

        let decimal = Decimal::from(0.000);
        assert_eq!("0.00", format!("{}", decimal));

        let decimal = Decimal::from(0.0001);
        assert_eq!("0.00", format!("{}", decimal));

        let decimal = Decimal::from(0.001);
        assert_eq!("0.00", format!("{}", decimal));

        let decimal = Decimal::from(0.01);
        assert_eq!("0.01", format!("{}", decimal));

        let decimal = Decimal::from(0.1);
        assert_eq!("0.10", format!("{}", decimal));

        let decimal = Decimal::from(0.10);
        assert_eq!("0.10", format!("{}", decimal));

        let decimal = Decimal::from(0.100);
        assert_eq!("0.10", format!("{}", decimal));

        let decimal = Decimal::from(0.1000);
        assert_eq!("0.10", format!("{}", decimal));

        let decimal = Decimal::from(0.1001);
        assert_eq!("0.10", format!("{}", decimal));

        let decimal = Decimal::from(0.101);
        assert_eq!("0.10", format!("{}", decimal));

        let decimal = Decimal::from(0.11);
        assert_eq!("0.11", format!("{}", decimal));

        let decimal = Decimal::from(0.111);
        assert_eq!("0.11", format!("{}", decimal));
    }

    #[test]
    fn test_from_str_display() {
        let decimal = Decimal::try_from("123456.78").unwrap();
        assert_eq!("123456.78", format!("{}", decimal));

        let decimal = Decimal::try_from("-123456.78").unwrap();
        assert_eq!("-123456.78", format!("{}", decimal));

        let decimal = Decimal::try_from("123456").unwrap();
        assert_eq!("123456.00", format!("{}", decimal));

        let decimal = Decimal::try_from("-123456").unwrap();
        assert_eq!("-123456.00", format!("{}", decimal));

        let decimal = Decimal::try_from("0").unwrap();
        assert_eq!("0.00", format!("{}", decimal));

        let decimal = Decimal::try_from("0.0").unwrap();
        assert_eq!("0.00", format!("{}", decimal));

        let decimal = Decimal::try_from("0.000").unwrap();
        assert_eq!("0.00", format!("{}", decimal));

        let decimal = Decimal::try_from("0.0001").unwrap();
        assert_eq!("0.00", format!("{}", decimal));

        let decimal = Decimal::try_from("0.001").unwrap();
        assert_eq!("0.00", format!("{}", decimal));

        let decimal = Decimal::try_from("0.01").unwrap();
        assert_eq!("0.01", format!("{}", decimal));

        let decimal = Decimal::try_from("0.1").unwrap();
        assert_eq!("0.10", format!("{}", decimal));

        let decimal = Decimal::try_from("0.10").unwrap();
        assert_eq!("0.10", format!("{}", decimal));

        let decimal = Decimal::try_from("0.100").unwrap();
        assert_eq!("0.10", format!("{}", decimal));

        let decimal = Decimal::try_from("0.1000").unwrap();
        assert_eq!("0.10", format!("{}", decimal));

        let decimal = Decimal::try_from("0.1001").unwrap();
        assert_eq!("0.10", format!("{}", decimal));

        let decimal = Decimal::try_from("0.101").unwrap();
        assert_eq!("0.10", format!("{}", decimal));

        let decimal = Decimal::try_from("0.11").unwrap();
        assert_eq!("0.11", format!("{}", decimal));

        let decimal = Decimal::try_from("0.111").unwrap();
        assert_eq!("0.11", format!("{}", decimal));
    }

    #[test]
    fn test_addition() {
        let decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(678.90);
        let result = decimal1 + decimal2;
        assert_eq!("802.35", format!("{}", result));

        let decimal1 = Decimal::from(-123.45);
        let decimal2 = Decimal::from(678.90);
        let result = decimal1 + decimal2;
        assert_eq!("555.45", format!("{}", result));

        let decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(-678.90);
        let result = decimal1 + decimal2;
        assert_eq!("-555.45", format!("{}", result));
    }

    #[test]
    fn test_subtraction() {
        let decimal1 = Decimal::from(678.90);
        let decimal2 = Decimal::from(123.45);
        let result = decimal1 - decimal2;
        assert_eq!("555.45", format!("{}", result));

        let decimal1 = Decimal::from(-123.45);
        let decimal2 = Decimal::from(678.90);
        let result = decimal1 - decimal2;
        assert_eq!("-802.35", format!("{}", result));

        let decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(-678.90);
        let result = decimal1 - decimal2;
        assert_eq!("802.35", format!("{}", result));
    }

    #[test]
    fn test_multiplication() {
        let decimal1 = Decimal::from(12.34);
        let decimal2 = Decimal::from(56.78);
        let result = decimal1 * decimal2;
        assert_eq!("700.66", format!("{}", result));

        let decimal1 = Decimal::from(-12.34);
        let decimal2 = Decimal::from(56.78);
        let result = decimal1 * decimal2;
        assert_eq!("-700.66", format!("{}", result));

        let decimal1 = Decimal::from(12.34);
        let decimal2 = Decimal::from(-56.78);
        let result = decimal1 * decimal2;
        assert_eq!("-700.66", format!("{}", result));
    }

    #[test]
    fn test_division() {
        let decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(6.78);
        let result = decimal1 / decimal2;
        assert_eq!("18.20", format!("{}", result));

        let decimal1 = Decimal::from(-123.45);
        let decimal2 = Decimal::from(6.78);
        let result = decimal1 / decimal2;
        assert_eq!("-18.20", format!("{}", result));

        let decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(-6.78);
        let result = decimal1 / decimal2;
        assert_eq!("-18.20", format!("{}", result));
    }

    #[test]
    fn test_add_assign() {
        let mut decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(678.90);
        decimal1 += decimal2;
        assert_eq!("802.35", format!("{}", decimal1));

        let mut decimal1 = Decimal::from(-123.45);
        let decimal2 = Decimal::from(678.90);
        decimal1 += decimal2;
        assert_eq!("555.45", format!("{}", decimal1));

        let mut decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(-678.90);
        decimal1 += decimal2;
        assert_eq!("-555.45", format!("{}", decimal1));
    }

    #[test]
    fn test_sub_assign() {
        let mut decimal1 = Decimal::from(678.90);
        let decimal2 = Decimal::from(123.45);
        decimal1 -= decimal2;
        assert_eq!("555.45", format!("{}", decimal1));

        let mut decimal1 = Decimal::from(-123.45);
        let decimal2 = Decimal::from(678.90);
        decimal1 -= decimal2;
        assert_eq!("-802.35", format!("{}", decimal1));

        let mut decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(-678.90);
        decimal1 -= decimal2;
        assert_eq!("802.35", format!("{}", decimal1));
    }

    #[test]
    fn test_mul_assign() {
        let mut decimal1 = Decimal::from(12.34);
        let decimal2 = Decimal::from(56.78);
        decimal1 *= decimal2;
        assert_eq!("700.66", format!("{}", decimal1));

        let mut decimal1 = Decimal::from(-12.34);
        let decimal2 = Decimal::from(56.78);
        decimal1 *= decimal2;
        assert_eq!("-700.66", format!("{}", decimal1));

        let mut decimal1 = Decimal::from(12.34);
        let decimal2 = Decimal::from(-56.78);
        decimal1 *= decimal2;
        assert_eq!("-700.66", format!("{}", decimal1));
    }

    #[test]
    fn test_div_assign() {
        let mut decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(6.78);
        decimal1 /= decimal2;
        assert_eq!("18.20", format!("{}", decimal1));

        let mut decimal1 = Decimal::from(-123.45);
        let decimal2 = Decimal::from(6.78);
        decimal1 /= decimal2;
        assert_eq!("-18.20", format!("{}", decimal1));

        let mut decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(-6.78);
        decimal1 /= decimal2;
        assert_eq!("-18.20", format!("{}", decimal1));
    }

    #[test]
    fn test_equality() {
        let decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(123.45);
        assert_eq!(decimal1, decimal2);

        let decimal1 = Decimal::from(-123.45);
        let decimal2 = Decimal::from(-123.45);
        assert_eq!(decimal1, decimal2);

        let decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(678.90);
        assert_ne!(decimal1, decimal2);

        let decimal1 = Decimal::from(-123.45);
        let decimal2 = Decimal::from(123.45);
        assert_ne!(decimal1, decimal2);
    }

    #[test]
    fn test_ordering() {
        let decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(678.90);
        assert!(decimal1 < decimal2);

        let decimal1 = Decimal::from(678.90);
        let decimal2 = Decimal::from(123.45);
        assert!(decimal1 > decimal2);

        let decimal1 = Decimal::from(-123.45);
        let decimal2 = Decimal::from(123.45);
        assert!(decimal1 < decimal2);

        let decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(-123.45);
        assert!(decimal1 > decimal2);

        let decimal1 = Decimal::from(123.45);
        let decimal2 = Decimal::from(123.45);
        assert!(decimal1 <= decimal2);
        assert!(decimal1 >= decimal2);
    }
}
