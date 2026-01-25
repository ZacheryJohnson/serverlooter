use std::fmt::{Display, Formatter};
use std::ops::Range;
use rand::Rng;
use crate::server::ServerStatType;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AlgorithmEffectValue {
    /// This value will always be a single value (the provided `i32`).
    Static(i32),

    /// This value will be any integer between the lower and upper `i32` values, inclusive.
    /// Will panic if lower is greater than upper.
    RangeInclusive(i32, i32),
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
    pub fn make_value(&self) -> i32 {
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

impl From<i32> for AlgorithmEffectValue {
    fn from(value: i32) -> Self {
        AlgorithmEffectValue::Static(value)
    }
}

impl From<Range<i32>> for AlgorithmEffectValue {
    fn from(value: Range<i32>) -> Self {
        AlgorithmEffectValue::RangeInclusive(value.start, value.end)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AlgorithmEffectTarget {
    /// The algorithm effect will target the server running the algorithm.
    /// This is often used for self-buffs.
    Host,

    /// The algorithm effect will target the other server.
    /// This is often used for debuffs.
    Remote,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AlgorithmEffect {
    /// `Siphon` steals credits from the target machine.
    /// The higher the `potency`, the more credits will be stolen from the target.
    Siphon { potency: AlgorithmEffectValue, },

    /// `Exfil` steals algorithms from the target machine.
    /// The higher the `potency`, the stronger algorithms will be stolen from the target.
    Exfil { potency: AlgorithmEffectValue, },

    /// `Modify` alters the stats of a server.
    /// This can be used to buff or debuff a `target`, either the hosting server or remote target server.
    /// The higher the `potency`, the stronger the effect on `stat`.
    Modify { target: AlgorithmEffectTarget, stat: ServerStatType, potency: AlgorithmEffectValue },
}

impl Display for AlgorithmEffect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // ZJ-TODO: localize
        match self {
            AlgorithmEffect::Siphon { potency } => {
                write!(f, "Siphon {potency}")
            },
            AlgorithmEffect::Exfil { potency } => {
                write!(f, "Exfil {potency}")
            },
            AlgorithmEffect::Modify { target, stat, potency } => {
                write!(f, "Modify {target:?}'s {stat:?} by {potency}")
            }
        }
    }
}