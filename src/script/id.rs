use std::fmt::{Display, Formatter};

#[derive(Clone, Default, PartialEq, Eq, Debug, Hash)]
pub enum ScriptId {
    #[default]
    Invalid,
    Id(u64),
}

impl Display for ScriptId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptId::Invalid => write!(f, "Invalid"), // ZJ-TODO: localize
            ScriptId::Id(id) => write!(f, "{id}"),
        }
    }
}