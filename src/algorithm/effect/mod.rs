pub mod application;
pub mod target;
pub mod value;

use std::collections::HashMap;
use std::fmt::Debug;
use fluent_templates::fluent_bundle::FluentValue;
use crate::algorithm::effect::target::AlgorithmEffectTarget;
use crate::algorithm::effect::value::AlgorithmEffectValue;
use crate::l10n::Localizable;
use crate::server::ServerStatType;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AlgorithmEffect {
    /// `Terminate` damages the connection health between the two servers.
    /// This is primarily used by opposing servers to disconnect the player's exploits.
    Terminate { potency: AlgorithmEffectValue, },

    /// `Siphon` steals credits from the target machine.
    /// The higher the `potency`, the more credits will be stolen from the target.
    Siphon { potency: AlgorithmEffectValue, },

    /// `Exfil` steals algorithms from the target machine.
    /// The higher the `potency`, the stronger algorithms will be stolen from the target.
    Exfil { potency: AlgorithmEffectValue, },

    /// `Modify` alters the stats of a server.
    /// This can be used to buff or debuff a `target`, either the hosting server or remote target server.
    /// The higher the `potency`, the stronger the effect on `stat`.
    Modify { target: AlgorithmEffectTarget, stat: ServerStatType, potency: AlgorithmEffectValue },

    /// `Purge` removes negative modifications of the type `stat` from `target`.
    /// Unlike [Modify](AlgorithmEffect::Modify), `Purge` will only return a stat to its baseline level.
    /// The higher the `potency`, the more negative modifications on self will be removed,
    /// or positive modifications on target will be removed.
    Purge { target: AlgorithmEffectTarget, stat: ServerStatType, potency: AlgorithmEffectValue },
}

impl Localizable for AlgorithmEffect {
    fn loc_key(&self) -> &'static str {
        match self {
            AlgorithmEffect::Terminate { .. } => "algorithm_effect_terminate_instance",
            AlgorithmEffect::Siphon { .. } => "algorithm_effect_siphon_instance",
            AlgorithmEffect::Exfil { .. } => "algorithm_effect_exfil_instance",
            AlgorithmEffect::Modify { .. } => "algorithm_effect_modify_instance",
            AlgorithmEffect::Purge { .. } => "algorithm_effect_purge_instance",
        }
    }

    fn loc_args(&self) -> HashMap<&'static str, FluentValue<'_>> {
        match self {
            AlgorithmEffect::Terminate { potency }
            | AlgorithmEffect::Siphon { potency }
            | AlgorithmEffect::Exfil { potency } => {
                HashMap::from([
                  ("potency", format!("{potency}").into())
                ])
            }
            AlgorithmEffect::Modify { target, stat, potency }
            | AlgorithmEffect::Purge { target, stat, potency } => {
                HashMap::from([
                    ("target", format!("{target:?}").into()),
                    ("stat", format!("{stat:?}").into()),
                    ("potency", format!("{potency}").into())
                ])
            }
        }
    }
}