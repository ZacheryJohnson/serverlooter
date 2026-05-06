// ZJ-TODO: figure out how to join/split procedures
// eg       x-x-x
//         /     \
//    x-x-x       x-x-x
//         \     /
//          x-x-/

pub mod executor;

use std::sync::{Arc, Mutex, Weak};
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::id::AlgorithmId;

#[derive(Clone)]
pub struct AlgorithmProcedure {
    algorithms: Vec<Arc<Mutex<Algorithm>>>,
}

impl AlgorithmProcedure {
    /// Creates a new procedure from a slice of algorithms.
    /// The algorithms will be executed such that the first element of the slice is executed
    /// before the second element of the slice, and so on.
    pub fn from(algorithms: &[Arc<Mutex<Algorithm>>]) -> Self {
        AlgorithmProcedure {
            algorithms: algorithms.to_vec(),
        }
    }

    /// Adds an algorithm to the end of the procedure.
    pub fn add_algorithm(&mut self, algorithm: Arc<Mutex<Algorithm>>) {
        // We execute algorithms from end of the vec to front
        self.algorithms.push(algorithm);
    }

    /// Removes an algorithm from the procedure, if it exists within.
    /// No error is returned if the element does not exist.
    pub fn remove_algorithm(&mut self, algorithm_id: AlgorithmId) {
        self.algorithms.retain(|algo| {
            algo.lock().unwrap().id != algorithm_id
        });
    }

    pub fn iterator(&self) -> AlgorithmProcedureIterator {
        AlgorithmProcedureIterator::new(self.algorithms.clone())
    }

    /// Returns the number of instructions algorithms in the procedure have remaining.
    /// This returns a current count, meaning calls to [next()](AlgorithmProcedure::next) will reduce this value.
    pub fn instruction_count(&self) -> u64 {
        self
            .algorithms
            .iter()
            .map(|algo| *algo.lock().unwrap().instruction_count)
            .sum::<u64>()
    }
}

#[derive(Clone)]
pub struct AlgorithmProcedureIterator {
    algorithms: Vec<Arc<Mutex<Algorithm>>>,
    index: usize,
}

impl AlgorithmProcedureIterator {
    pub fn new(algorithms: Vec<Arc<Mutex<Algorithm>>>) -> Self {
        AlgorithmProcedureIterator {
            algorithms,
            index: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.index >= self.algorithms.len()
    }
}

impl Iterator for AlgorithmProcedureIterator {
    type Item = Weak<Mutex<Algorithm>>;

    fn next(&mut self) -> Option<Self::Item> {
        let maybe_algorithm = match self.algorithms.get(self.index) {
            None => None,
            Some(algo) => Some(Arc::downgrade(algo)),
        };

        self.index += 1;

        maybe_algorithm
    }
}
