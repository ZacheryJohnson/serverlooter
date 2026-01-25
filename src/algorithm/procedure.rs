// ZJ-TODO: figure out how to join/split procedures
// eg       x-x-x
//         /     \
//    x-x-x       x-x-x
//         \     /
//          x-x-/

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::algorithm::algorithm::Algorithm;
use crate::script::AlgorithmExecutor;

#[derive(Clone)]
pub struct AlgorithmProcedure {
    algorithms: VecDeque<Arc<Mutex<Algorithm>>>,
}

impl AlgorithmProcedure {
    /// Creates a new procedure from a slice of algorithms.
    /// The algorithms will be executed such that the first element of the slice is executed
    /// before the second element of the slice, and so on.
    pub fn from(algorithms: &[Arc<Mutex<Algorithm>>]) -> Self {
        let mut algorithm_vec = algorithms.to_vec();
        algorithm_vec.reverse();

        AlgorithmProcedure {
            algorithms: VecDeque::from(algorithm_vec),
        }
    }

    /// Returns an iterator of algorithms in the procedure,
    /// where the first algorithm returned is the first to be executed.
    pub fn algorithms(&self) -> impl Iterator<Item = &Arc<Mutex<Algorithm>>> {
        self.algorithms.iter().rev()
    }

    /// Adds an algorithm to the end of the procedure.
    pub fn add_algorithm(&mut self, algorithm: Arc<Mutex<Algorithm>>) {
        self.algorithms.push_front(algorithm);
    }

    /// Removes an algorithm from the procedure, if it exists within.
    /// No error is returned if the element does not exist.
    pub fn remove_algorithm(&mut self, algorithm: Arc<Mutex<Algorithm>>) {
        self.algorithms.retain(|algo| {
            let a_id = { algo.lock().unwrap().id.clone() };
            let b_id = { algorithm.lock().unwrap().id.clone() };
            a_id != b_id
        });
    }

    /// Returns true when there are no further algorithms in the procedure.
    pub fn is_complete(&self) -> bool {
        self.algorithms.is_empty()
    }

    /// Pops the next algorithm and returns an executor for that algorithm.
    /// If None, there were no more algorithms in the procedure prior to the next call.
    pub fn next(&mut self) -> Option<AlgorithmExecutor> {
        let Some(next_algorithm) = self.algorithms.pop_back() else {
            return None
        };

        let executor = AlgorithmExecutor::from(next_algorithm);
        Some(executor)
    }

    /// Returns the number of instructions algorithms in the procedure have remaining.
    /// This returns a current count, meaning calls to [next()](AlgorithmProcedure::next) will reduce this value.
    pub fn instruction_count(&self) -> u64 {
        self
            .algorithms
            .iter()
            .map(|algo| algo.lock().unwrap().instruction_count)
            .sum::<u64>()
    }
}