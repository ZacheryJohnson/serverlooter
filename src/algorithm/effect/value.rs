use std::fmt::{Display, Formatter};
use std::ops::Range;
use rand::RngExt;

pub type AlgorithmEffectValueT = i32;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AlgorithmEffectValue {
    /// This value will always be a single value (the provided `i32`).
    Static(AlgorithmEffectValueT),

    /// This value will be any integer between the lower and upper `i32` values, inclusive.
    /// Will panic if lower is greater than upper.
    RangeInclusive(AlgorithmEffectValueT, AlgorithmEffectValueT),
}

impl Display for AlgorithmEffectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlgorithmEffectValue::Static(val) => {
                write!(f, "{val}")
            }
            AlgorithmEffectValue::RangeInclusive(low, upp) => {
                write!(f, "{low}-{upp}")
            }
        }
    }
}

impl AlgorithmEffectValue {
    /// Gets or generates a value.
    /// Repeated calls may result in different values in the case of range values (such as [RangeInclusive](AlgorithmEffectValue::RangeInclusive)).
    pub fn make_value(&self) -> AlgorithmEffectValueT {
        let rng = &mut rand::rng();
        match self {
            Self::Static(v) => *v,
            Self::RangeInclusive(min, max) => {
                assert!(min <= max);
                rng.sample(
                    rand::distr::Uniform::new(*min, *max + 1).unwrap()
                )
            },
        }
    }
}

impl From<AlgorithmEffectValueT> for AlgorithmEffectValue {
    fn from(value: AlgorithmEffectValueT) -> Self {
        AlgorithmEffectValue::Static(value)
    }
}

impl From<Range<AlgorithmEffectValueT>> for AlgorithmEffectValue {
    fn from(value: Range<AlgorithmEffectValueT>) -> Self {
        AlgorithmEffectValue::RangeInclusive(value.start, value.end)
    }
}