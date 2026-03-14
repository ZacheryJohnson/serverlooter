use std::fmt::{Debug, Formatter};

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum AlgorithmEffectTarget {
    /// The algorithm effect will target the server running the algorithm.
    /// This is often used for self-buffs.
    SelfServer,

    /// The algorithm effect will target the other server.
    /// This is often used for debuffs.
    TargetServer,
}

impl Debug for AlgorithmEffectTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SelfServer => write!(f, "Self"),
            Self::TargetServer => write!(f, "Target"),
        }
    }
}