use std::collections::HashSet;
use std::sync::{Mutex, Weak};
use bevy_egui::egui::text::LayoutJob;
use crate::{PlayerState};
use crate::script::{Algorithm, AlgorithmEffect, Script, ScriptId};
use crate::ui::hover_text::OnHoverText;

pub enum InventoryGridItemDisplay {
    Text(String),
}

pub trait InventoryGridItem {
    /// Displayed as the "icon" of the grid item.
    /// This can be stylized text, or in the future (ZJ-TODO) an image.
    fn display(&self) -> InventoryGridItemDisplay;

    /// Displayed when the grid item is hovered in the inventory.
    fn hover_text(&self) -> &LayoutJob;
}

pub struct AlgorithmGridItem {
    pub algorithm: Weak<Mutex<Algorithm>>,

    hover_text: LayoutJob,
}

impl AlgorithmGridItem {
    pub fn from(
        algorithm: Weak<Mutex<Algorithm>>,
        player_state: &PlayerState
    ) -> AlgorithmGridItem {
        let hover_text = {
            let hover_text = algorithm.upgrade().unwrap().lock().unwrap().on_hover_text(player_state).clone();
            hover_text
        };

        AlgorithmGridItem {
            algorithm,
            hover_text,
        }
    }
}

impl InventoryGridItem for AlgorithmGridItem {
    fn display(&self) -> InventoryGridItemDisplay {
        let algorithm = self.algorithm.upgrade().unwrap();
        let algorithm = algorithm.lock().unwrap();

        let mut display_str = String::new();
        let effect_set = algorithm
            .instruction_effects
            .iter()
            .map(|(_, effects)| effects.to_owned())
            .flatten()
            .collect::<HashSet<_>>();

        // ZJ-TODO: rather than a HashSet, we should consider a BTreeSet, and sort by highest-potency first
        //          this would order the display string to show more clearly what an algorithm does best
        for effect in effect_set {
            match effect {
                AlgorithmEffect::Siphon { .. } => display_str.push_str("$"),
                AlgorithmEffect::Exfil { .. } => display_str.push_str("X"),
                AlgorithmEffect::Modify { .. } => display_str.push_str("~"),
            }
        }

        InventoryGridItemDisplay::Text(display_str)
    }

    fn hover_text(&self) -> &LayoutJob {
        &self.hover_text
    }
}

pub struct ScriptGridItem {
    pub script: Weak<Mutex<Script>>,

    hover_text: LayoutJob,
}

impl ScriptGridItem {
    pub fn from(
        script: Weak<Mutex<Script>>,
    ) -> ScriptGridItem {
        let hover_text = {
            let hover_text = script.upgrade().unwrap().lock().unwrap().on_hover_text(&()).clone();
            hover_text
        };

        ScriptGridItem {
            script,
            hover_text,
        }
    }
}

impl InventoryGridItem for ScriptGridItem {
    fn display(&self) -> InventoryGridItemDisplay {
        let script_id = {
            let script = self.script.upgrade();
            let script_id = script.unwrap().lock().unwrap().id.clone();
            script_id
        };

        match script_id {
            ScriptId::Invalid => {
                InventoryGridItemDisplay::Text("!".to_string())
            }
            ScriptId::Id(id) => {
                InventoryGridItemDisplay::Text(id.to_string())
            }
        }
    }

    fn hover_text(&self) -> &LayoutJob {
        &self.hover_text
    }
}