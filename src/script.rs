use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use bevy::prelude::Event;
use rand::Rng;

#[derive(Event)]
pub struct ScriptCreatedEvent {
    pub script: Script
}

// ZJ-TODO: figure out how to join/split procedures
// eg       x-x-x
//         /     \
//    x-x-x       x-x-x
//         \     /
//          x-x-/

#[derive(Clone)]
pub struct AlgorithmProcedure {
    pub algorithms: VecDeque<Algorithm>,
}

impl AlgorithmProcedure {
    pub fn is_complete(&self) -> bool {
        self.algorithms.is_empty()
    }

    pub fn next(&mut self) -> Option<AlgorithmExecutor> {
        let Some(next_algorithm) = self.algorithms.pop_back() else {
            return None
        };

        let executor = AlgorithmExecutor::from(next_algorithm);
        Some(executor)
    }

    pub fn instruction_count(&self) -> u64 {
        self
            .algorithms
            .iter()
            .map(|algo| algo.instruction_count)
            .sum::<u64>()
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum ScriptId {
    Invalid,
    Id(u64),
}

impl Display for ScriptId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptId::Invalid => write!(f, "Invalid"),
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
        Script { id: ScriptId::Invalid, procedures: Vec::new() }
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
            .all(|proc| proc.algorithms.is_empty())
    }

    pub fn add_algorithm(&mut self, algorithm: Algorithm) {
        // ZJ-TODO: handle multiple procedures
        match self.script.procedures.first_mut() {
            Some(procedure) => {
                procedure.algorithms.push_front(algorithm)
            },
            None => self.script.procedures.push(AlgorithmProcedure {
                algorithms: vec![algorithm].into(),
            })
        }
    }

    pub fn remove_algorithm(&mut self, algorithm: Algorithm) {
        for procedure in self.script.procedures.iter_mut() {
            procedure.algorithms.retain(|algo| algo != &algorithm);
        }
    }

    pub fn current_script(&self) -> &Script {
        &self.script
    }

    pub fn finish(self) -> Script {
        self.script
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Algorithm {
    /// How many instructions does this algorithm contain?
    /// Once all instructions are executed, the algorithm is considered complete
    pub instruction_count: u64,

    /// What effects are applied on what instruction?
    /// This could be a hashmap, but with so few effects per algorithm this is plenty efficient
    pub instruction_effects: Vec<(u64, Vec<AlgorithmEffect>)>,
}

impl Default for Algorithm {
    fn default() -> Self {
        Algorithm {
            instruction_count: 0,
            instruction_effects: vec![],
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AlgorithmEffectValue {
    /// This value will always be a single value (the provided `i32`).
    Static(i32),

    /// This value will be any integer between the lower and upper `i32` values, inclusive.
    /// Will panic if lower is greater than upper.
    RangeInclusive(i32, i32),
}

impl Display for AlgorithmEffectValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlgorithmEffectValue::Static(val) => {
                write!(f, "{val}")
            }
            AlgorithmEffectValue::RangeInclusive(low, upp) => {
                write!(f, "{low}-{upp}")
            }
        }
    }
}

impl AlgorithmEffectValue {
    /// Gets or generates a value.
    /// Repeated calls may result in different values in the case of range values (such as [RangeInclusive](AlgorithmEffectValue::RangeInclusive)).
    pub fn make_value(&self) -> i32 {
        let rng = &mut rand::rng();
        match self {
            Self::Static(v) => *v,
            Self::RangeInclusive(min, max) => {
                assert!(min <= max);
                rng.sample(
                    rand::distr::Uniform::new(*min, *max + 1).unwrap()
                )
            },
        }
    }
}

impl From<i32> for AlgorithmEffectValue {
    fn from(value: i32) -> Self {
        AlgorithmEffectValue::Static(value)
    }
}

impl From<Range<i32>> for AlgorithmEffectValue {
    fn from(value: Range<i32>) -> Self {
        AlgorithmEffectValue::RangeInclusive(value.start, value.end)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AlgorithmEffect {
    Extract { potency: AlgorithmEffectValue, }
}

impl Display for AlgorithmEffect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlgorithmEffect::Extract { potency } => {
                write!(f, "Extract {potency}")
            },
        }
    }
}

pub trait Executor {
    fn start_execution(&mut self);
    fn stop_execution(&mut self);
    // ZJ-TODO: this doesn't handle partial ticks
    //          for example, if tick_count = 5, and we're on the last instruction
    //          of an algorithm, the next algorithm in the procedure won't be ticked for 4
    //          We'll instead currently "lose" 4 ticks.
    //          The ticks should instead be re-applied to the next algorithm
    fn tick_execution(&mut self, tick_count: u64) -> Vec<AlgorithmEffect>;
    fn is_complete(&self) -> bool;
    fn progress(&self) -> u64;
    fn total_instructions(&self) -> u64;
}

#[derive(Clone)]
pub struct AlgorithmExecutor {
    algorithm: Algorithm,
    instruction_pointer: u64,
    is_paused: bool,
}

impl AlgorithmExecutor {
    pub fn from(algorithm: Algorithm) -> AlgorithmExecutor {
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
            .instruction_effects
            .iter()
            .filter(|(instruction_count, _)| *instruction_count > self.instruction_pointer && self.instruction_pointer + tick_count >= *instruction_count)
            .map(|(_, effects)| effects.to_owned())
            .flatten()
            .collect::<Vec<AlgorithmEffect>>();

        self.instruction_pointer = (self.instruction_pointer + tick_count).min(self.algorithm.instruction_count);

        next_effects
    }

    fn is_complete(&self) -> bool {
        self.instruction_pointer >= self.algorithm.instruction_count
    }

    fn progress(&self) -> u64 {
        self.instruction_pointer
    }

    fn total_instructions(&self) -> u64 {
        self.algorithm.instruction_count
    }
}

#[derive(Clone)]
struct AlgorithmProcedureExecutor {
    procedure: AlgorithmProcedure,
    finished_algorithms: Vec<Algorithm>,
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
            .map(|algorithm| algorithm.instruction_count)
            .sum();

        let current_instruction_pointer = self.algorithm_executor.instruction_pointer;

        completed_instruction_count + current_instruction_pointer
    }

    fn total_instructions(&self) -> u64 {
        self.total_expected_instructions
    }
}

#[derive(Clone)]
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

    #[test]
    fn algorithm_executor_can_complete() {
        let algorithm = Algorithm {
            instruction_count: 3,
            instruction_effects: Default::default(),
        };

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
        let algorithm1 = Algorithm {
            instruction_count: 3,
            instruction_effects: Default::default(),
        };

        let algorithm2 = Algorithm {
            instruction_count: 3,
            instruction_effects: Default::default(),
        };

        let procedure = AlgorithmProcedure {
            algorithms: vec![algorithm2, algorithm1].into(),
        };

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
        let algorithm1 = Algorithm {
            instruction_count: 5,
            instruction_effects: vec![
                (1, vec![
                    AlgorithmEffect::Extract { potency: 1.into(), }
                ]),
            ],
        };

        let algorithm2 = Algorithm {
            instruction_count: 10,
            instruction_effects: vec![
                (5, vec![
                    AlgorithmEffect::Extract { potency: 2.into(), }
                ]),
            ],
        };

        let procedure = AlgorithmProcedure {
            algorithms: vec![algorithm2, algorithm1].into(),
        };

        let script = Script::new(
            ScriptId::Id(1),
            vec![procedure],
        );

        let expected_effects_on_tick = BTreeMap::from([
            // Algorithm 1 = first algorithm, so no delay
            (1, vec![
                AlgorithmEffect::Extract { potency: 1.into(), }
            ]),
            // Algorithm 2 = second algorithm, so 5 instructions for algorithm 1 then 5 instructions until the effect
            (10, vec![
                AlgorithmEffect::Extract { potency: 2.into(), }
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