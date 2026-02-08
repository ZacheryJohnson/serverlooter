use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};
use bevy::prelude::Event;
use crate::algorithm::algorithm::Algorithm;
use crate::algorithm::effect::AlgorithmEffect;
use crate::algorithm::procedure::AlgorithmProcedure;
use crate::executor::Executor;

#[derive(Event)]
pub struct ScriptCreatedEvent {
    pub script: Script
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum ScriptId {
    Invalid,
    Id(u64),
}

impl Display for ScriptId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptId::Invalid => write!(f, "Invalid"), // ZJ-TODO: localize
            ScriptId::Id(id) => write!(f, "{id}"),
        }
    }
}

#[derive(Clone)]
pub struct Script {
    pub id: ScriptId,
    pub procedures: Vec<AlgorithmProcedure>,
}

impl Script {
    pub fn empty() -> Script {
        Script::new(ScriptId::Invalid, Vec::new())
    }

    pub fn new(id: ScriptId, procedures: Vec<AlgorithmProcedure>) -> Script {
        Script { id, procedures }
    }

    /// Clones self to create an executor.
    pub fn executor(&self) -> ScriptExecutor {
        let clone = self.clone();
        ScriptExecutor::from(clone)
    }

    pub fn into_executor(self) -> ScriptExecutor {
        ScriptExecutor::from(self)
    }
}

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

#[derive(Clone)]
pub struct AlgorithmExecutor {
    algorithm: Arc<Mutex<Algorithm>>,
    instruction_pointer: u64,
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
            .filter(|(instruction_count, _)| *instruction_count > self.instruction_pointer && self.instruction_pointer + tick_count >= *instruction_count)
            .map(|(_, effects)| effects.to_owned())
            .flatten()
            .collect::<Vec<AlgorithmEffect>>();

        self.instruction_pointer = (self.instruction_pointer + tick_count).min(self.algorithm.lock().unwrap().instruction_count);

        next_effects
    }

    fn is_complete(&self) -> bool {
        self.instruction_pointer >= self.algorithm.lock().unwrap().instruction_count
    }

    fn progress(&self) -> u64 {
        self.instruction_pointer
    }

    fn total_instructions(&self) -> u64 {
        self.algorithm.lock().unwrap().instruction_count
    }
}

#[derive(Clone)]
struct AlgorithmProcedureExecutor {
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
            .map(|algorithm| algorithm.lock().unwrap().instruction_count)
            .sum();

        let current_instruction_pointer = self.algorithm_executor.instruction_pointer;

        completed_instruction_count + current_instruction_pointer
    }

    fn total_instructions(&self) -> u64 {
        self.total_expected_instructions
    }
}

#[derive(Clone, Default)]
pub struct ScriptExecutor {
    algorithm_procedure_executors: Vec<AlgorithmProcedureExecutor>,
    is_paused: bool,
}

impl ScriptExecutor {
    pub fn from(script: Script) -> ScriptExecutor {
        ScriptExecutor {
            algorithm_procedure_executors: script
                .procedures
                .into_iter()
                .filter_map(AlgorithmProcedureExecutor::from) // ZJ-TODO: explicitly return errors?
                .collect(),
            is_paused: false,
        }
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use uuid::Uuid;
    use crate::algorithm::id::AlgorithmId;

    fn make_id() -> AlgorithmId {
        Uuid::new_v4().into()
    }

    #[test]
    fn algorithm_executor_can_complete() {
        let algorithm = Arc::new(Mutex::new(Algorithm {
            id: make_id(),
            instruction_count: 3,
            instruction_effects: Default::default(),
        }));

        let mut executor = AlgorithmExecutor::from(algorithm);

        assert_eq!(executor.is_complete(), false);
        executor.start_execution();
        assert_eq!(executor.is_complete(), false);

        executor.tick_execution(1);
        assert_eq!(executor.is_complete(), false);
        executor.tick_execution(1);
        assert_eq!(executor.is_complete(), false);
        executor.tick_execution(1);
        assert_eq!(executor.is_complete(), true);
    }

    #[test]
    fn algorithm_procedure_executor_can_complete() {
        let algorithm1 = Arc::new(Mutex::new(Algorithm {
            id: make_id(),
            instruction_count: 3,
            instruction_effects: Default::default(),
        }));

        let algorithm2 = Arc::new(Mutex::new(Algorithm {
            id: make_id(),
            instruction_count: 3,
            instruction_effects: Default::default(),
        }));

        let procedure = AlgorithmProcedure::from(&[algorithm1, algorithm2]);
        let mut executor = AlgorithmProcedureExecutor::from(procedure).unwrap();

        assert_eq!(executor.is_complete(), false);
        executor.start_execution();
        assert_eq!(executor.is_complete(), false);

        for i in 1..=6 {
            executor.tick_execution(1);
            assert_eq!(executor.is_complete(), i == 6);
        }
    }

    #[test]
    fn script_executor_can_complete() {
        let algorithm1 = Arc::new(Mutex::new(Algorithm {
            id: make_id(),
            instruction_count: 5,
            instruction_effects: vec![
                (1, vec![
                    AlgorithmEffect::Siphon { potency: 1.into(), }
                ]),
            ],
        }));

        let algorithm2 = Arc::new(Mutex::new(Algorithm {
            id: make_id(),
            instruction_count: 10,
            instruction_effects: vec![
                (5, vec![
                    AlgorithmEffect::Siphon { potency: 2.into(), }
                ]),
            ],
        }));

        let procedure = AlgorithmProcedure::from(&[algorithm1, algorithm2]);

        let script = Script::new(
            ScriptId::Id(1),
            vec![procedure],
        );

        let expected_effects_on_tick = BTreeMap::from([
            // Algorithm 1 = first algorithm, so no delay
            (1, vec![
                AlgorithmEffect::Siphon { potency: 1.into(), }
            ]),
            // Algorithm 2 = second algorithm, so 5 instructions for algorithm 1 then 5 instructions until the effect
            (10, vec![
                AlgorithmEffect::Siphon { potency: 2.into(), }
            ]),
        ]);

        let mut executor = script.into_executor();
        assert_eq!(executor.is_complete(), false);
        executor.start_execution();
        assert_eq!(executor.is_complete(), false);

        let mut current_tick = 0;
        const MAX_TICKS_RUNAWAY_LOOP: usize = 1000;
        while !executor.is_complete() {
            if current_tick >= MAX_TICKS_RUNAWAY_LOOP {
                panic!("script executor never completing!");
            }

            current_tick += 1;

            let expected_effects = expected_effects_on_tick.get(&current_tick).unwrap_or(&vec![]).to_owned();
            let actual_effects = executor.tick_execution(1);

            assert_eq!(expected_effects, actual_effects, "Tick {current_tick}");
        }

        assert!(executor.is_complete());
    }
}