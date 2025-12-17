use std::collections::HashMap;

/// Reputations are the equivalent of Runescape levels.
/// They increase by performing actions successfully.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum ReputationType {
    /// Extract is a BlackHat skill used to gain credits from servers.
    /// Reputation is gained when successfully gaining credits.
    Extract,
}

pub struct Reputations {
    pub reputation_values: HashMap<ReputationType, u32>,
}

impl Reputations {
    pub fn new() -> Self {
        Reputations { reputation_values: HashMap::new() }
    }

    pub fn get(&self, reputation_type: ReputationType) -> u32 {
        self.reputation_values.get(&reputation_type).unwrap_or(&0).to_owned()
    }

    pub fn get_mut(&mut self, reputation_type: ReputationType) -> &mut u32 {
        self.reputation_values.entry(reputation_type).or_default()
    }
}