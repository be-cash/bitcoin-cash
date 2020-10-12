use std::convert::{TryFrom, TryInto};

pub type InnerInteger = i32;

pub const MIN_SCRIPT_INTEGER: InnerInteger = -0x7fff_ffff;
pub const MAX_SCRIPT_INTEGER: InnerInteger = 0x7fff_ffff;

#[derive(Error, Debug, Copy, Clone, PartialEq)]
pub enum IntegerError {
    #[error("Addition overflowed: {0} + {1}")]
    AddOverflow(InnerInteger, InnerInteger),

    #[error("Subtraction overflowed: {0} + {1}")]
    SubOverflow(InnerInteger, InnerInteger),

    #[error("Multiplication overflowed: {0} / {1}")]
    MulOverflow(InnerInteger, InnerInteger),

    #[error("Division error: {0} / {1}")]
    Division(InnerInteger, InnerInteger),

    #[error("Modulo error: {0} % {1}")]
    Modulo(InnerInteger, InnerInteger),

    #[error("Left shift overflow: {0} << {1}")]
    ShlOverflow(InnerInteger, u32),

    #[error("Right shift overflow: {0} << {1}")]
    ShrOverflow(InnerInteger, u32),

    #[error("Negation overflowed: -{0}")]
    NegOverflow(InnerInteger),

    #[error("Invalid script integer: {0}")]
    InvalidScriptInteger(InnerInteger),

    #[error("Cast overflowed: {0}")]
    CastOverflow(i128),

    #[error("Cast overflowed i128")]
    CastOverflowI128,
}

#[derive(Debug, Copy, Clone)]
pub struct Integer(InnerInteger);

#[derive(Debug, Copy, Clone)]
pub struct IntegerResult(Result<Integer, IntegerError>);

impl Integer {
    pub const ZERO: Integer = Integer(0);

    pub fn new(
        int: impl TryInto<InnerInteger> + TryInto<i128> + Clone,
    ) -> Result<Self, IntegerError> {
        int.clone()
            .try_into()
            .map_err(|_| match int.try_into() {
                Ok(value) => IntegerError::CastOverflow(value),
                Err(_) => IntegerError::CastOverflowI128,
            })
            .and_then(|value| {
                if !(MIN_SCRIPT_INTEGER..=MAX_SCRIPT_INTEGER).contains(&value) {
                    return Err(IntegerError::InvalidScriptInteger(value));
                } else {
                    return Ok(Integer(value));
                }
            })
    }

    pub fn value(self) -> InnerInteger {
        self.0
    }
}

impl IntegerResult {
    pub fn new(int: impl TryInto<InnerInteger> + TryInto<i128> + Clone) -> Self {
        IntegerResult(Integer::new(int))
    }

    fn op(
        self,
        other: impl TryInto<IntegerResult, Error = impl Into<IntegerError>>,
        op: impl Fn(InnerInteger, InnerInteger) -> Result<InnerInteger, IntegerError>,
    ) -> IntegerResult {
        IntegerResult(self.0.and_then(|a| {
            let a = a.0;
            let b = other.try_into().map_err(Into::into)?.0?.0;
            Integer::new(op(a, b)?)
        }))
    }

    fn unop(
        self,
        op: impl Fn(InnerInteger) -> Result<InnerInteger, IntegerError>,
    ) -> IntegerResult {
        IntegerResult(self.0.and_then(|value| Integer::new(op(value.0)?)))
    }

    pub fn value(self) -> Result<InnerInteger, IntegerError> {
        Ok(self.0?.0)
    }

    pub fn integer(self) -> Result<Integer, IntegerError> {
        self.0
    }
}

impl Default for Integer {
    fn default() -> Self {
        Integer(0)
    }
}

impl Default for IntegerResult {
    fn default() -> Self {
        IntegerResult(Ok(Default::default()))
    }
}

impl std::fmt::Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for IntegerResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Ok(int) => write!(f, "{}", int),
            Err(err) => write!(f, "(integer error: {})", err),
        }
    }
}

impl From<Integer> for InnerInteger {
    fn from(value: Integer) -> Self {
        value.0
    }
}

impl From<Integer> for IntegerResult {
    fn from(value: Integer) -> Self {
        IntegerResult(Ok(value))
    }
}

impl TryFrom<IntegerResult> for InnerInteger {
    type Error = IntegerError;

    fn try_from(value: IntegerResult) -> Result<Self, Self::Error> {
        Ok(value.0?.0)
    }
}

impl TryFrom<InnerInteger> for Integer {
    type Error = IntegerError;

    fn try_from(value: InnerInteger) -> Result<Self, Self::Error> {
        Integer::new(value)
    }
}

impl From<InnerInteger> for IntegerResult {
    fn from(int: InnerInteger) -> Self {
        IntegerResult::new(int)
    }
}

impl From<std::convert::Infallible> for IntegerError {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!("Impossible value")
    }
}

impl<T: Into<Integer> + Clone> PartialEq<T> for Integer {
    fn eq(&self, other: &T) -> bool {
        self.0 == other.clone().into().0
    }
}

impl Eq for Integer {}

impl<E, T> PartialEq<T> for IntegerResult
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E> + Clone,
{
    fn eq(&self, other: &T) -> bool {
        let a = match self.0 {
            Ok(a) => a,
            Err(_) => return false,
        };
        let b = match other
            .clone()
            .try_into()
            .map_err(Into::into)
            .and_then(|b| b.0)
        {
            Ok(b) => b,
            Err(_) => return false,
        };
        a.eq(&b)
    }
}

impl<T: Into<Integer> + Clone> PartialOrd<T> for Integer {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.clone().into())
    }
}

impl<E, T> PartialOrd<T> for IntegerResult
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E> + Clone,
{
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        let a = match self.0 {
            Ok(a) => a,
            Err(_) => return None,
        };
        let b = match other
            .clone()
            .try_into()
            .map_err(Into::into)
            .and_then(|b| b.0)
        {
            Ok(b) => b,
            Err(_) => return None,
        };
        a.partial_cmp(&b)
    }
}

impl std::ops::Deref for Integer {
    type Target = InnerInteger;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E, T> std::ops::Add<T> for IntegerResult
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn add(self, rhs: T) -> Self::Output {
        self.op(rhs, |a, b| {
            a.checked_add(b).ok_or(IntegerError::AddOverflow(a, b))
        })
    }
}

impl<E, T> std::ops::Add<T> for Integer
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn add(self, rhs: T) -> Self::Output {
        IntegerResult(Ok(self)) + rhs
    }
}

impl<E, T> std::ops::BitAnd<T> for IntegerResult
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn bitand(self, rhs: T) -> Self::Output {
        self.op(rhs, |a, b| Ok(a & b))
    }
}

impl<E, T> std::ops::BitAnd<T> for Integer
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn bitand(self, rhs: T) -> Self::Output {
        IntegerResult(Ok(self)) & rhs
    }
}

impl<E, T> std::ops::BitOr<T> for IntegerResult
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn bitor(self, rhs: T) -> Self::Output {
        self.op(rhs, |a, b| Ok(a | b))
    }
}

impl<E, T> std::ops::BitOr<T> for Integer
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn bitor(self, rhs: T) -> Self::Output {
        IntegerResult(Ok(self)) | rhs
    }
}

impl<E, T> std::ops::BitXor<T> for IntegerResult
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn bitxor(self, rhs: T) -> Self::Output {
        self.op(rhs, |a, b| Ok(a ^ b))
    }
}

impl<E, T> std::ops::BitXor<T> for Integer
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn bitxor(self, rhs: T) -> Self::Output {
        IntegerResult(Ok(self)) ^ rhs
    }
}

impl<E, T> std::ops::Div<T> for IntegerResult
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn div(self, rhs: T) -> Self::Output {
        self.op(rhs, |a, b| {
            a.checked_div(b).ok_or(IntegerError::Division(a, b))
        })
    }
}

impl<E, T> std::ops::Div<T> for Integer
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn div(self, rhs: T) -> Self::Output {
        IntegerResult(Ok(self)) / rhs
    }
}

impl<E, T> std::ops::Mul<T> for IntegerResult
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn mul(self, rhs: T) -> Self::Output {
        self.op(rhs, |a, b| {
            a.checked_mul(b).ok_or(IntegerError::MulOverflow(a, b))
        })
    }
}

impl<E, T> std::ops::Mul<T> for Integer
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn mul(self, rhs: T) -> Self::Output {
        IntegerResult(Ok(self)) * rhs
    }
}

impl<E, T> std::ops::Rem<T> for IntegerResult
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn rem(self, rhs: T) -> Self::Output {
        self.op(rhs, |a, b| {
            a.checked_rem(b).ok_or(IntegerError::Modulo(a, b))
        })
    }
}

impl<E, T> std::ops::Rem<T> for Integer
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn rem(self, rhs: T) -> Self::Output {
        IntegerResult(Ok(self)) % rhs
    }
}

impl<E, T> std::ops::Sub<T> for IntegerResult
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn sub(self, rhs: T) -> Self::Output {
        self.op(rhs, |a, b| {
            a.checked_sub(b).ok_or(IntegerError::SubOverflow(a, b))
        })
    }
}

impl<E, T> std::ops::Sub<T> for Integer
where
    E: Into<IntegerError>,
    T: TryInto<IntegerResult, Error = E>,
{
    type Output = IntegerResult;

    fn sub(self, rhs: T) -> Self::Output {
        IntegerResult(Ok(self)) - rhs
    }
}

impl std::ops::Neg for IntegerResult {
    type Output = IntegerResult;

    fn neg(self) -> Self::Output {
        self.unop(|a| a.checked_neg().ok_or(IntegerError::NegOverflow(a)))
    }
}

impl std::ops::Neg for Integer {
    type Output = Integer;

    fn neg(self) -> Self::Output {
        Integer(-self.0)
    }
}

impl std::ops::Shl<u32> for IntegerResult {
    type Output = IntegerResult;

    fn shl(self, rhs: u32) -> Self::Output {
        self.unop(|a| a.checked_shl(rhs).ok_or(IntegerError::ShlOverflow(a, rhs)))
    }
}

impl std::ops::Shl<u32> for Integer {
    type Output = IntegerResult;

    fn shl(self, rhs: u32) -> Self::Output {
        IntegerResult(Ok(self)) << rhs
    }
}

impl std::ops::Shr<u32> for IntegerResult {
    type Output = IntegerResult;

    fn shr(self, rhs: u32) -> Self::Output {
        self.unop(|a| a.checked_shr(rhs).ok_or(IntegerError::ShrOverflow(a, rhs)))
    }
}

impl std::ops::Shr<u32> for Integer {
    type Output = IntegerResult;

    fn shr(self, rhs: u32) -> Self::Output {
        IntegerResult(Ok(self)) >> rhs
    }
}

#[cfg(test)]
mod test {
    use super::{Integer, IntegerError, IntegerResult, MAX_SCRIPT_INTEGER, MIN_SCRIPT_INTEGER};

    #[test]
    fn test_new() -> Result<(), IntegerError> {
        assert_eq!(Integer::new(0)?.value(), 0);
        assert_eq!(Integer::new(0u128)?.value(), 0);
        assert_eq!(
            Integer::new(MIN_SCRIPT_INTEGER)?.value(),
            MIN_SCRIPT_INTEGER
        );
        assert_eq!(
            Integer::new(MAX_SCRIPT_INTEGER)?.value(),
            MAX_SCRIPT_INTEGER
        );

        assert_eq!(
            Integer::new(-0x8000_0000).unwrap_err(),
            IntegerError::InvalidScriptInteger(-0x8000_0000)
        );
        assert_eq!(
            Integer::new(0x1000_0000_0000i128).unwrap_err(),
            IntegerError::CastOverflow(0x1000_0000_0000i128),
        );
        Ok(())
    }

    #[test]
    fn test_eq() -> Result<(), IntegerError> {
        assert_eq!(Integer::new(0)?, Integer::new(0)?);
        assert_eq!(Integer::new(1)?, Integer::new(1)?);
        assert_eq!(IntegerResult::new(0), IntegerResult::new(0));
        assert_eq!(IntegerResult::new(1), IntegerResult::new(1));
        assert_eq!(IntegerResult::new(0), 0);
        assert_eq!(IntegerResult::new(1), 1);
        assert_eq!(
            Integer::new(MAX_SCRIPT_INTEGER),
            Integer::new(MAX_SCRIPT_INTEGER)
        );
        assert_eq!(
            Integer::new(MIN_SCRIPT_INTEGER),
            Integer::new(MIN_SCRIPT_INTEGER)
        );
        assert_eq!(IntegerResult::new(MAX_SCRIPT_INTEGER), MAX_SCRIPT_INTEGER);
        assert_eq!(IntegerResult::new(MIN_SCRIPT_INTEGER), MIN_SCRIPT_INTEGER);
        Ok(())
    }

    #[test]
    fn test_ne() -> Result<(), IntegerError> {
        assert_ne!(Integer::new(0), Integer::new(1));
        assert_ne!(Integer::new(1), Integer::new(0));
        assert_ne!(IntegerResult::new(0), IntegerResult::new(1));
        assert_ne!(IntegerResult::new(1), IntegerResult::new(0));
        assert_ne!(IntegerResult::new(0), 1);
        assert_ne!(IntegerResult::new(1), 0);

        assert_ne!(IntegerResult::new(-0x8000_0000), -0x8000_0000);
        assert_ne!(
            IntegerResult::new(-0x8000_0000),
            IntegerResult::new(-0x8000_0000)
        );
        assert_ne!(IntegerResult::new(-0x8000_0000), 0);
        assert_ne!(IntegerResult::new(-0x8000_0000), 1);
        assert_ne!(IntegerResult::new(-0x8000_0000), IntegerResult::new(0));

        assert_ne!(IntegerResult::new(0x1000_0000_0000u128), 0);
        assert_ne!(
            IntegerResult::new(0x1000_0000_0000u128),
            IntegerResult::new(0)
        );
        assert_ne!(
            IntegerResult::new(0x1000_0000_0000u128),
            IntegerResult::new(-0x8000_0000)
        );
        assert_ne!(
            IntegerResult::new(0x1000_0000_0000u128),
            IntegerResult::new(0x1000_0000_0000u128)
        );
        Ok(())
    }

    #[test]
    fn test_ord() -> Result<(), IntegerError> {
        assert!(Integer::new(0)? < Integer::new(1)?);
        assert!(IntegerResult::new(0) < IntegerResult::new(1));
        assert!(IntegerResult::new(0) < 1);
        assert!(Integer::new(0)? <= Integer::new(1)?);
        assert!(IntegerResult::new(0) <= IntegerResult::new(1));
        assert!(IntegerResult::new(0) <= 1);
        assert!(Integer::new(1)? > Integer::new(0)?);
        assert!(IntegerResult::new(1) > IntegerResult::new(0));
        assert!(IntegerResult::new(1) > 0);
        assert!(Integer::new(1)? >= Integer::new(0)?);
        assert!(IntegerResult::new(1) >= IntegerResult::new(0));
        assert!(IntegerResult::new(1) >= 0);

        assert!(Integer::new(MIN_SCRIPT_INTEGER)? >= Integer::new(MIN_SCRIPT_INTEGER)?);
        assert!(IntegerResult::new(MIN_SCRIPT_INTEGER) >= MIN_SCRIPT_INTEGER);
        assert!(IntegerResult::new(MIN_SCRIPT_INTEGER + 1) > MIN_SCRIPT_INTEGER);

        assert!(!(IntegerResult::new(-0x8000_0000) < IntegerResult::new(0)));
        assert!(!(IntegerResult::new(-0x8000_0000) < 0));
        assert!(!(IntegerResult::new(-0x8000_0000) <= IntegerResult::new(0)));
        assert!(!(IntegerResult::new(-0x8000_0000) <= 0));
        assert!(!(IntegerResult::new(-0x8000_0000) > IntegerResult::new(0)));
        assert!(!(IntegerResult::new(-0x8000_0000) > 0));
        assert!(!(IntegerResult::new(-0x8000_0000) >= IntegerResult::new(0)));
        assert!(!(IntegerResult::new(-0x8000_0000) >= 0));

        assert!(!(IntegerResult::new(0) < IntegerResult::new(-0x8000_0000)));
        assert!(!(IntegerResult::new(0) < -0x8000_0000));
        assert!(!(IntegerResult::new(0) <= IntegerResult::new(-0x8000_0000)));
        assert!(!(IntegerResult::new(0) <= -0x8000_0000));
        assert!(!(IntegerResult::new(0) > IntegerResult::new(-0x8000_0000)));
        assert!(!(IntegerResult::new(0) > -0x8000_0000));
        assert!(!(IntegerResult::new(0) >= IntegerResult::new(-0x8000_0000)));
        assert!(!(IntegerResult::new(0) >= -0x8000_0000));
        Ok(())
    }

    #[test]
    fn test_add() -> Result<(), IntegerError> {
        assert_eq!(Integer::new(0)? + Integer::new(1)?, 1);
        assert_eq!(Integer::new(0)? + 1, 1);
        assert_eq!(Integer::new(1000)? + Integer::new(2000)?, 3000);
        assert_eq!(Integer::new(1000)? + 2000, 3000);
        assert_eq!(
            Integer::new(MIN_SCRIPT_INTEGER)? + Integer::new(MAX_SCRIPT_INTEGER)?,
            0
        );
        assert_eq!(Integer::new(MIN_SCRIPT_INTEGER)? + MAX_SCRIPT_INTEGER, 0);
        assert_eq!(
            Integer::new(MAX_SCRIPT_INTEGER)? + Integer::new(MIN_SCRIPT_INTEGER)?,
            0
        );
        assert_eq!(Integer::new(MAX_SCRIPT_INTEGER)? + MIN_SCRIPT_INTEGER, 0);

        assert_eq!(
            (Integer::new(MIN_SCRIPT_INTEGER)? + (-1))
                .value()
                .unwrap_err(),
            IntegerError::InvalidScriptInteger(-0x8000_0000)
        );
        assert_eq!(
            (Integer::new(MIN_SCRIPT_INTEGER)? + (-2))
                .value()
                .unwrap_err(),
            IntegerError::AddOverflow(MIN_SCRIPT_INTEGER, -2)
        );
        Ok(())
    }

    #[test]
    fn test_sub() -> Result<(), IntegerError> {
        assert_eq!(Integer::new(0)? - Integer::new(1)?, -1);
        assert_eq!(Integer::new(0)? - 1, -1);
        assert_eq!(Integer::new(1000)? - Integer::new(2000)?, -1000);
        assert_eq!(Integer::new(1000)? - 2000, -1000);
        assert_eq!(
            Integer::new(MAX_SCRIPT_INTEGER)? - Integer::new(MAX_SCRIPT_INTEGER)?,
            0
        );
        assert_eq!(Integer::new(MIN_SCRIPT_INTEGER)? - MIN_SCRIPT_INTEGER, 0);
        assert_eq!(
            Integer::new(MIN_SCRIPT_INTEGER)? - Integer::new(MIN_SCRIPT_INTEGER)?,
            0
        );
        assert_eq!(Integer::new(MIN_SCRIPT_INTEGER)? - MIN_SCRIPT_INTEGER, 0);

        assert_eq!(
            (Integer::new(MIN_SCRIPT_INTEGER)? - 1).value().unwrap_err(),
            IntegerError::InvalidScriptInteger(-0x8000_0000)
        );
        assert_eq!(
            (Integer::new(MIN_SCRIPT_INTEGER)? - 2).value().unwrap_err(),
            IntegerError::SubOverflow(MIN_SCRIPT_INTEGER, 2)
        );
        Ok(())
    }

    #[test]
    fn test_div() -> Result<(), IntegerError> {
        assert_eq!(Integer::new(7)? / Integer::new(3)?, 2);
        assert_eq!(Integer::new(7)? / 3, 2);
        assert_eq!(Integer::new(2000)? / Integer::new(1000)?, 2);
        assert_eq!(Integer::new(2000)? / 1000, 2);
        assert_eq!(
            Integer::new(MAX_SCRIPT_INTEGER)? / Integer::new(MAX_SCRIPT_INTEGER)?,
            1
        );
        assert_eq!(Integer::new(MIN_SCRIPT_INTEGER)? / MAX_SCRIPT_INTEGER, -1);
        assert_eq!(
            Integer::new(MAX_SCRIPT_INTEGER)? / Integer::new(MIN_SCRIPT_INTEGER)?,
            -1
        );
        assert_eq!(Integer::new(MIN_SCRIPT_INTEGER)? / MIN_SCRIPT_INTEGER, 1);

        assert_eq!(
            (Integer::new(MIN_SCRIPT_INTEGER)? / 0).value().unwrap_err(),
            IntegerError::Division(MIN_SCRIPT_INTEGER, 0)
        );
        Ok(())
    }

    #[test]
    fn test_rem() -> Result<(), IntegerError> {
        assert_eq!(Integer::new(7)? % Integer::new(3)?, 1);
        assert_eq!(Integer::new(7)? % 3, 1);
        assert_eq!(Integer::new(2000)? % Integer::new(1000)?, 0);
        assert_eq!(Integer::new(2000)? % 1000, 0);
        assert_eq!(
            Integer::new(MAX_SCRIPT_INTEGER)? % Integer::new(MAX_SCRIPT_INTEGER)?,
            0
        );
        assert_eq!(Integer::new(MIN_SCRIPT_INTEGER)? % MAX_SCRIPT_INTEGER, 0);
        assert_eq!(
            Integer::new(MAX_SCRIPT_INTEGER)? % Integer::new(MIN_SCRIPT_INTEGER)?,
            0
        );
        assert_eq!(Integer::new(MIN_SCRIPT_INTEGER)? % MIN_SCRIPT_INTEGER, 0);

        assert_eq!(
            (Integer::new(MIN_SCRIPT_INTEGER)? % 0).value().unwrap_err(),
            IntegerError::Modulo(MIN_SCRIPT_INTEGER, 0)
        );
        Ok(())
    }

    #[test]
    fn test_mul() -> Result<(), IntegerError> {
        assert_eq!(Integer::new(7)? * Integer::new(3)?, 21);
        assert_eq!(Integer::new(7)? * 3, 21);
        assert_eq!(Integer::new(2000)? * Integer::new(1000)?, 2_000_000);
        assert_eq!(Integer::new(2000)? * 1000, 2_000_000);
        assert_eq!(
            Integer::new(MAX_SCRIPT_INTEGER)? * Integer::new(1)?,
            MAX_SCRIPT_INTEGER
        );
        assert_eq!(Integer::new(MIN_SCRIPT_INTEGER)? * 1, MIN_SCRIPT_INTEGER);

        assert_eq!(
            (Integer::new(MAX_SCRIPT_INTEGER)? * 2).value().unwrap_err(),
            IntegerError::MulOverflow(MAX_SCRIPT_INTEGER, 2)
        );
        assert_eq!(
            (Integer::new(-0x8000)? * 0x10000).value().unwrap_err(),
            IntegerError::InvalidScriptInteger(-0x8000_0000)
        );
        assert_eq!(
            (Integer::new(0x8000)? * 0x10000).value().unwrap_err(),
            IntegerError::MulOverflow(0x8000, 0x10000)
        );
        Ok(())
    }

    #[test]
    fn test_shr() -> Result<(), IntegerError> {
        assert_eq!(Integer::new(0x700)? >> 4, 0x70);
        assert_eq!(Integer::new(0x2000)? >> 10, 8);
        assert_eq!(Integer::new(MAX_SCRIPT_INTEGER)? >> 8, 0x007f_ffff,);
        assert_eq!(Integer::new(MIN_SCRIPT_INTEGER)? >> 8, -0x0080_0000);
        assert_eq!(Integer::new(MAX_SCRIPT_INTEGER)? >> 31, 0);

        assert_eq!(
            (Integer::new(MAX_SCRIPT_INTEGER)? >> 32)
                .value()
                .unwrap_err(),
            IntegerError::ShrOverflow(MAX_SCRIPT_INTEGER, 32)
        );
        Ok(())
    }

    #[test]
    fn test_shl() -> Result<(), IntegerError> {
        assert_eq!(Integer::new(0x700)? << 4, 0x7000);
        assert_eq!(Integer::new(0x2000)? << 8, 0x20_0000);
        assert_eq!(Integer::new(MAX_SCRIPT_INTEGER)? << 8, -0x100,);
        assert_eq!(Integer::new(MIN_SCRIPT_INTEGER)? << 8, 0x100);
        assert_eq!(Integer::new(MAX_SCRIPT_INTEGER)? << 30, -0x4000_0000);

        assert_eq!(
            (Integer::new(MAX_SCRIPT_INTEGER)? << 31)
                .value()
                .unwrap_err(),
            IntegerError::InvalidScriptInteger(-0x8000_0000)
        );
        assert_eq!(
            (Integer::new(MAX_SCRIPT_INTEGER)? << 32)
                .value()
                .unwrap_err(),
            IntegerError::ShlOverflow(MAX_SCRIPT_INTEGER, 32)
        );
        Ok(())
    }
}
