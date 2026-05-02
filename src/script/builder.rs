use std::sync::{Arc, Mutex};
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::procedure::AlgorithmProcedure;
use crate::script::Script;

pub struct ScriptBuilder {
    script: Script,
}

impl ScriptBuilder {
    pub fn new() -> ScriptBuilder {
        ScriptBuilder {
            script: Script::empty(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self
            .script
            .procedures
            .iter()
            .all(|proc| proc.algorithms().count() == 0)
    }

    pub fn add_algorithm(&mut self, algorithm: Arc<Mutex<Algorithm>>) {
        // ZJ-TODO: handle multiple procedures
        match self.script.procedures.first_mut() {
            Some(procedure) => procedure.add_algorithm(algorithm),
            None => self.script.procedures.push(AlgorithmProcedure::from(&[algorithm])),
        }
    }

    pub fn remove_algorithm(&mut self, algorithm: Arc<Mutex<Algorithm>>) {
        for procedure in self.script.procedures.iter_mut() {
            procedure.remove_algorithm(algorithm.clone());
        }
    }

    pub fn current_script(&self) -> &Script {
        &self.script
    }

    pub fn finish(self) -> Script {
        self.script
    }
}