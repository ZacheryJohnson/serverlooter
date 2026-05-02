use std::sync::{Arc, Mutex};
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::effect::AlgorithmEffect;
use crate::executor::Executor;

#[derive(Clone)]
pub struct AlgorithmExecutor {
    pub(crate) algorithm: Arc<Mutex<Algorithm>>,
    pub(crate) instruction_pointer: u64,
    is_paused: bool,
}

impl AlgorithmExecutor {
    pub fn from(algorithm: Arc<Mutex<Algorithm>>) -> AlgorithmExecutor {
        AlgorithmExecutor {
            algorithm,
            instruction_pointer: 0,
            is_paused: true,
        }
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

        let next_effects = self
            .algorithm
            .lock()
            .unwrap()
            .instruction_effects
            .iter()
            .filter(|(instruction_count, _)| **instruction_count > self.instruction_pointer && self.instruction_pointer + tick_count >= **instruction_count)
            .map(|(_, effects)| effects.to_owned())
            .flatten()
            .collect::<Vec<AlgorithmEffect>>();

        self.instruction_pointer = (self.instruction_pointer + tick_count).min(*self.algorithm.lock().unwrap().instruction_count);

        next_effects
    }

    fn is_complete(&self) -> bool {
        self.instruction_pointer >= *self.algorithm.lock().unwrap().instruction_count
    }

    fn progress(&self) -> u64 {
        self.instruction_pointer
    }

    fn total_instructions(&self) -> u64 {
        *self.algorithm.lock().unwrap().instruction_count
    }
}