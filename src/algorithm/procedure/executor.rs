use std::sync::{Arc, Mutex};
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::effect::AlgorithmEffect;
use crate::algorithm::executor::AlgorithmExecutor;
use crate::algorithm::procedure::AlgorithmProcedure;
use crate::executor::Executor;

#[derive(Clone)]
pub struct AlgorithmProcedureExecutor {
    procedure: AlgorithmProcedure,
    finished_algorithms: Vec<Arc<Mutex<Algorithm>>>,
    algorithm_executor: AlgorithmExecutor,

    total_expected_instructions: u64,
    is_paused: bool,
}

impl AlgorithmProcedureExecutor {
    /// Creates an AlgorithmProcedureExecutor from an AlgorithmProcedure.
    /// If the procedure contains no algorithms, returns None.
    pub fn from(mut algorithm_procedure: AlgorithmProcedure) -> Option<AlgorithmProcedureExecutor> {
        let total_expected_instructions = algorithm_procedure.instruction_count();

        let Some(executor) = algorithm_procedure.next() else {
            return None;
        };

        Some(AlgorithmProcedureExecutor {
            procedure: algorithm_procedure,
            finished_algorithms: Vec::new(),
            algorithm_executor: executor,
            total_expected_instructions,
            is_paused: true,
        })
    }
}

impl Executor for AlgorithmProcedureExecutor {
    fn start_execution(&mut self) {
        self.is_paused = false;
        self.algorithm_executor.start_execution();
    }

    fn stop_execution(&mut self) {
        self.is_paused = true;
        self.algorithm_executor.stop_execution();
    }

    fn tick_execution(&mut self, tick_count: u64) -> Vec<AlgorithmEffect> {
        if self.is_paused {
            return vec![];
        }

        if self.algorithm_executor.is_complete() {
            // Take old algorithm and store it so we can track overall progress
            let finished_algorithm = std::mem::take(
                &mut self.algorithm_executor.algorithm
            );

            self.finished_algorithms.push(finished_algorithm);

            // Pop next algorithm and begin executing it
            let Some(next_algorithm_executor) = self.procedure.next() else {
                return vec![];
            };

            self.algorithm_executor = next_algorithm_executor;
            self.algorithm_executor.start_execution();
        }

        self.algorithm_executor.tick_execution(tick_count)
    }

    fn is_complete(&self) -> bool {
        self.procedure.is_complete() && self.algorithm_executor.is_complete()
    }

    fn progress(&self) -> u64 {
        let completed_instruction_count: u64 = self
            .finished_algorithms
            .iter()
            .map(|algorithm| *algorithm.lock().unwrap().instruction_count)
            .sum();

        let current_instruction_pointer = self.algorithm_executor.instruction_pointer;

        completed_instruction_count + current_instruction_pointer
    }

    fn total_instructions(&self) -> u64 {
        self.total_expected_instructions
    }
}