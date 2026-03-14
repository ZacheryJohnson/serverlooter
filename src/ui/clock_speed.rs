use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use fluent_templates::fluent_bundle::FluentValue;
use crate::l10n::Localizable;

#[derive(Clone)]
pub struct ClockSpeed {
    clock_speed_hz: u64,
}

impl Deref for ClockSpeed {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.clock_speed_hz
    }
}

impl DerefMut for ClockSpeed {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.clock_speed_hz
    }
}

impl From<u64> for ClockSpeed {
    fn from(clock_speed: u64) -> Self {
        Self { clock_speed_hz: clock_speed }
    }
}

impl ClockSpeed {
    pub fn new(clock_speed_hz: u64) -> Self {
        Self { clock_speed_hz }
    }
}

impl Localizable for ClockSpeed {
    fn loc_key(&self) -> &'static str {
        "ui_server_clock_speed"
    }

    fn loc_args(&self) -> HashMap<&'static str, FluentValue<'_>> {
        let server_speed_digits = self.clock_speed_hz.ilog10();
        let (unit, clock_speed) = match server_speed_digits {
            0..3 => ("hz", self.clock_speed_hz as f32),
            3..6 => ("khz", self.clock_speed_hz as f32 / 1_000.0),
            6..9 => ("mhz", self.clock_speed_hz as f32 / 1_000_000.0),
            _ => ("ghz", self.clock_speed_hz as f32 / 1_000_000_000.0)
        };

        [
            ("unit", unit.into()),
            ("clock_speed", clock_speed.into())
        ].into()
    }
}