use crate::algorithm::effect::AlgorithmEffect;

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