use crate::algorithm::effect::AlgorithmEffect;
use crate::algorithm::id::AlgorithmId;

#[derive(Clone)]
pub struct Algorithm {
    pub id: AlgorithmId,

    /// How many instructions does this algorithm contain?
    /// Once all instructions are executed, the algorithm is considered complete
    pub instruction_count: u64,

    /// What effects are applied on what instruction?
    /// This could be a hashmap, but with so few effects per algorithm this is plenty efficient
    pub instruction_effects: Vec<(u64, Vec<AlgorithmEffect>)>,
}

impl PartialEq for Algorithm {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Default for Algorithm {
    fn default() -> Self {
        Algorithm {
            id: AlgorithmId::Invalid,
            instruction_count: 0,
            instruction_effects: vec![],
        }
    }
}