use std::sync::{Arc, Mutex, Weak};
use crate::algorithm::effect::AlgorithmEffect;
use crate::algorithm::procedure::executor::AlgorithmProcedureExecutor;
use crate::executor::Executor;
use crate::script::Script;

#[derive(Clone, Default)]
pub struct ScriptExecutor {
    script: Weak<Mutex<Script>>,
    algorithm_procedure_executors: Vec<AlgorithmProcedureExecutor>,
    is_paused: bool,
}

impl ScriptExecutor {
    pub fn from(script: Weak<Mutex<Script>>) -> ScriptExecutor {
        let script_arc = script.clone().upgrade().unwrap();
        let script_inner = script_arc.lock().unwrap();
        let algorithm_procedure_executors = script_inner
            .procedures
            .iter()
            .filter_map(AlgorithmProcedureExecutor::from) // ZJ-TODO: explicitly return errors?
            .collect();

        ScriptExecutor {
            script,
            algorithm_procedure_executors,
            is_paused: false,
        }
    }

    pub fn from_arc(script: &Arc<Mutex<Script>>) -> ScriptExecutor {
        ScriptExecutor::from(Arc::downgrade(script))
    }
}

impl Executor for ScriptExecutor {
    fn start_execution(&mut self) {
        self.is_paused = false;
        for procedure_executor in &mut self.algorithm_procedure_executors {
            procedure_executor.start_execution();
        }
    }

    fn stop_execution(&mut self) {
        self.is_paused = true;
        for procedure_executor in &mut self.algorithm_procedure_executors {
            procedure_executor.stop_execution();
        }
    }

    fn tick_execution(&mut self, tick_count: u64) -> Vec<AlgorithmEffect> {
        if self.is_paused {
            return vec![];
        }

        let mut new_effects = vec![];

        for procedure_executor in &mut self.algorithm_procedure_executors {
            new_effects.extend(procedure_executor.tick_execution(tick_count));
        }

        new_effects
    }

    fn is_complete(&self) -> bool {
        self.algorithm_procedure_executors.iter().all(|exec| exec.is_complete())
    }

    fn progress(&self) -> u64 {
        self
            .algorithm_procedure_executors
            .iter()
            .map(|exec| exec.progress())
            .sum()
    }

    fn total_instructions(&self) -> u64 {
        self
            .algorithm_procedure_executors
            .iter()
            .map(|exec| exec.total_instructions())
            .sum()
    }
}