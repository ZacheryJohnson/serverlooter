use std::sync::{Arc, Mutex};
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::id::AlgorithmId;
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
            .all(|proc| proc.lock().unwrap().iterator().next().is_none())
    }

    pub fn add_algorithm(&mut self, algorithm: Arc<Mutex<Algorithm>>) {
        // ZJ-TODO: handle multiple procedures
        match self.script.procedures.first() {
            Some(procedure) => procedure.lock().unwrap().add_algorithm(algorithm),
            None => self.script.procedures.push(Arc::new(Mutex::new(
                AlgorithmProcedure::from(&[algorithm]))
            )),
        }
    }

    pub fn remove_algorithm(&mut self, algorithm_id: AlgorithmId) {
        for procedure in self.script.procedures.iter_mut() {
            let mut procedure = procedure.lock().unwrap();
            procedure.remove_algorithm(algorithm_id.clone());
        }
    }

    pub fn current_script(&self) -> &Script {
        &self.script
    }

    pub fn finish(self) -> Script {
        self.script
    }
}