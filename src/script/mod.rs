pub mod event;
pub mod id;
pub mod builder;
pub mod executor;

use std::sync::{Arc, Mutex};
use crate::algorithm::procedure::AlgorithmProcedure;
use crate::script::id::ScriptId;

#[derive(Clone)]
pub struct Script {
    pub id: ScriptId,
    pub procedures: Vec<Arc<Mutex<AlgorithmProcedure>>>,
}

impl Script {
    pub fn empty() -> Script {
        Script::new(ScriptId::Invalid, Vec::new())
    }

    pub fn new(id: ScriptId, procedures: Vec<Arc<Mutex<AlgorithmProcedure>>>) -> Script {
        Script { id, procedures }
    }

    pub fn instruction_count(&self) -> u64 {
        self
            .procedures
            .iter()
            .map(|procedure| procedure.lock().unwrap().instruction_count())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;
    use crate::algorithm::algorithm::Algorithm;
    use crate::algorithm::effect::AlgorithmEffect;
    use crate::algorithm::executor::AlgorithmExecutor;
    use crate::algorithm::id::AlgorithmId;
    use crate::algorithm::procedure::executor::AlgorithmProcedureExecutor;
    use crate::executor::Executor;
    use crate::script::executor::ScriptExecutor;

    fn make_id() -> AlgorithmId {
        Uuid::new_v4().into()
    }

    #[test]
    fn algorithm_executor_can_complete() {
        let algorithm = Arc::new(Mutex::new(Algorithm {
            id: make_id(),
            instruction_count: 3.into(),
            instruction_effects: Default::default(),
        }));

        let mut executor = AlgorithmExecutor::from(Arc::downgrade(&algorithm));

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
            instruction_count: 3.into(),
            instruction_effects: Default::default(),
        }));

        let algorithm2 = Arc::new(Mutex::new(Algorithm {
            id: make_id(),
            instruction_count: 3.into(),
            instruction_effects: Default::default(),
        }));

        let procedure = Arc::new(Mutex::new(
            AlgorithmProcedure::from(&[algorithm1, algorithm2])
        ));
        let mut executor = AlgorithmProcedureExecutor::from(&procedure).unwrap();

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
            instruction_count: 5.into(),
            instruction_effects: vec![
                (1.into(), vec![
                    AlgorithmEffect::Siphon { potency: 1.into(), }
                ]),
            ],
        }));

        let algorithm2 = Arc::new(Mutex::new(Algorithm {
            id: make_id(),
            instruction_count: 10.into(),
            instruction_effects: vec![
                (5.into(), vec![
                    AlgorithmEffect::Siphon { potency: 2.into(), }
                ]),
            ],
        }));

        let procedure = Arc::new(Mutex::new(
            AlgorithmProcedure::from(&[algorithm1, algorithm2])
        ));

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

        let script_arc = Arc::new(Mutex::new(script));
        let script_weak = Arc::downgrade(&script_arc);

        let mut executor = ScriptExecutor::from(script_weak);
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