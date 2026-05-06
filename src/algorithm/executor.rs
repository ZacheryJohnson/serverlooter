use std::sync::{Arc, Mutex, Weak};
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::effect::AlgorithmEffect;
use crate::executor::Executor;

#[derive(Clone)]
pub struct AlgorithmExecutor {
    pub(crate) algorithm: Weak<Mutex<Algorithm>>,
    pub(crate) instruction_pointer: u64,
    is_paused: bool,
}

impl AlgorithmExecutor {
    pub fn from(algorithm: Weak<Mutex<Algorithm>>) -> AlgorithmExecutor {
        AlgorithmExecutor {
            algorithm,
            instruction_pointer: 0,
            is_paused: true,
        }
    }

    pub fn from_arc(algorithm: &Arc<Mutex<Algorithm>>) -> AlgorithmExecutor {
        AlgorithmExecutor::from(Arc::downgrade(algorithm))
    }
}

impl Executor for AlgorithmExecutor {
    fn start_execution(&mut self) {
        self.is_paused = false;
    }

    fn stop_execution(&mut self) {
        self.is_paused = true;
    }

    fn tick_execution(&mut self, tick_count: u64) -> Vec<AlgorithmEffect> {
        if self.is_paused {
            return vec![];
        }

        let algorithm_arc = self.algorithm.upgrade().unwrap();
        let algorithm = algorithm_arc.lock().unwrap();

        let next_effects = algorithm
            .instruction_effects
            .iter()
            .filter(|(instruction_count, _)| **instruction_count > self.instruction_pointer && self.instruction_pointer + tick_count >= **instruction_count)
            .map(|(_, effects)| effects.to_owned())
            .flatten()
            .collect::<Vec<AlgorithmEffect>>();

        self.instruction_pointer = (self.instruction_pointer + tick_count).min(*algorithm.instruction_count);

        next_effects
    }

    fn is_complete(&self) -> bool {
        self.instruction_pointer >= self.total_instructions()
    }

    fn progress(&self) -> u64 {
        self.instruction_pointer
    }

    fn total_instructions(&self) -> u64 {
        let algorithm_arc = self.algorithm.upgrade().unwrap();
        let algorithm = algorithm_arc.lock().unwrap();
        *algorithm.instruction_count
    }
}