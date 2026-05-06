use std::sync::{Arc, Mutex, Weak};
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::effect::AlgorithmEffect;
use crate::algorithm::executor::AlgorithmExecutor;
use crate::algorithm::procedure::{AlgorithmProcedure, AlgorithmProcedureIterator};
use crate::executor::Executor;

#[derive(Clone)]
pub struct AlgorithmProcedureExecutor {
    procedure: Weak<Mutex<AlgorithmProcedure>>,
    procedure_iterator: AlgorithmProcedureIterator,
    finished_algorithms: Vec<Weak<Mutex<Algorithm>>>,
    algorithm_executor: AlgorithmExecutor,

    total_expected_instructions: u64,
    is_paused: bool,
}

impl AlgorithmProcedureExecutor {
    /// Creates an AlgorithmProcedureExecutor from an AlgorithmProcedure.
    /// If the procedure contains no algorithms, returns None.
    pub fn from(algorithm_procedure: &Arc<Mutex<AlgorithmProcedure>>) -> Option<AlgorithmProcedureExecutor> {
        let procedure_inner = algorithm_procedure.lock().unwrap();
        let total_expected_instructions = procedure_inner.instruction_count();

        let mut procedure_iterator = procedure_inner.iterator();

        let Some(algorithm) = procedure_iterator.next() else {
            return None;
        };

        Some(AlgorithmProcedureExecutor {
            procedure: Arc::downgrade(algorithm_procedure),
            procedure_iterator,
            finished_algorithms: Vec::new(),
            algorithm_executor: AlgorithmExecutor::from(algorithm),
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
            let Some(next_algorithm_executor) = self.procedure_iterator.next() else {
                return vec![];
            };

            self.algorithm_executor = AlgorithmExecutor::from(next_algorithm_executor);
            self.algorithm_executor.start_execution();
        }

        self.algorithm_executor.tick_execution(tick_count)
    }

    fn is_complete(&self) -> bool {
        self.procedure_iterator.is_empty() && self.algorithm_executor.is_complete()
    }

    fn progress(&self) -> u64 {
        let completed_instruction_count: u64 = self
            .finished_algorithms
            .iter()
            .map(|algorithm| *algorithm.upgrade().unwrap().lock().unwrap().instruction_count)
            .sum();

        let current_instruction_pointer = self.algorithm_executor.instruction_pointer;

        completed_instruction_count + current_instruction_pointer
    }

    fn total_instructions(&self) -> u64 {
        self.total_expected_instructions
    }
}