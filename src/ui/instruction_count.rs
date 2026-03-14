use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use fluent_templates::fluent_bundle::FluentValue;
use crate::l10n::Localizable;

#[derive(Clone)]
pub struct InstructionCount {
    instruction_count: u64,
}

impl Deref for InstructionCount {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.instruction_count
    }
}

impl DerefMut for InstructionCount {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instruction_count
    }
}

impl From<u64> for InstructionCount {
    fn from(instruction_count: u64) -> Self {
        Self { instruction_count }
    }
}

impl InstructionCount {
    pub fn new(instruction_count: u64) -> Self {
        Self { instruction_count }
    }
}

impl Localizable for InstructionCount {
    fn loc_key(&self) -> &'static str {
        "ui_algorithm_instruction_count"
    }

    fn loc_args(&self) -> HashMap<&'static str, FluentValue<'_>> {
        [("instruction_count", self.instruction_count.into())].into()
    }
}