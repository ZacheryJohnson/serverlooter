use std::sync::{Arc, Mutex};
use bevy::prelude::{Commands, On, ResMut};
use crate::event::request_start_exploit::RequestStartExploitEvent;
use crate::player_state::state::{PlayerState, PlayerUnlock};
use crate::{lock_and_clone, TIME_BETWEEN_TICKS};
use crate::active_exploit::{ActiveExploit, ActiveExploitStatus};
use crate::algorithm::effect::{AlgorithmEffect, AlgorithmEffectApplication, AlgorithmEffectTarget};
use crate::algorithm::generator::AlgorithmGenerator;
use crate::event::exploit_event::ExploitEvent;
use crate::event::modify_credits::{ModificationSource, ModifyCreditsEvent};
use crate::event::request_pause_exploit::RequestPauseExploitEvent;
use crate::event::request_purchase_unlock::RequestPurchaseUnlockEvent;
use crate::event::request_restart_exploit::RequestRestartExploitEvent;
use crate::event::request_resume_exploit::RequestResumeExploitEvent;
use crate::event::request_stop_exploit::RequestStopExploitEvent;
use crate::inventory::{InventoryItem, InventoryItemAdded};
use crate::script::ScriptCreatedEvent;
use crate::server::{ServerStatInstance, ServerStatSource, ServerStatType, ServerStats};
use crate::tutorial::progression::TutorialProgression;
use crate::ui::state::UiState;
use crate::ui::window::active_exploit::ActiveExploitWindow;

pub(crate) fn on_request_start_exploit(
    evt: On<RequestStartExploitEvent>,
    mut player_state: ResMut<PlayerState>,
    mut ui_state: ResMut<UiState>,
) -> bevy::prelude::Result {
    let target = evt.target.clone();
    let script = evt.script.clone();
    let server = evt.server.clone();

    // ZJ-TODO: validate server can accommodate another process
    // ZJ-TODO: validate server can meets thread minimums

    // ZJ-TODO: actually implement way to shift resource allocation
    //          for now, just time share equally
    let target_server_name = lock_and_clone!(evt.server, name);
    let new_total_processes = player_state
        .active_exploits
        .iter()
        .filter(|exploit| lock_and_clone!(exploit, hosting_server, name) == target_server_name)
        .count() as u64 + 1;

    let new_clock_speed_per_process = lock_and_clone!(server, clock_speed_hz) / new_total_processes;

    for existing_exploit in &mut player_state.active_exploits {
        let mut existing_exploit = existing_exploit.lock().unwrap();
        existing_exploit.clock_allocation_hz = new_clock_speed_per_process;
    }

    let auto_reconnect = player_state.player_unlocks.exploit_auto_reconnect;
    let active_exploit = Arc::new(Mutex::new(ActiveExploit::new(
        target,
        script,
        server,
        new_clock_speed_per_process,
        auto_reconnect,
    )));

    ui_state.active_exploit_windows.push(ActiveExploitWindow::new(active_exploit.clone()));
    player_state.active_exploits.push(active_exploit);

    Ok(())
}

pub(crate) fn on_request_stop_exploit(
    evt: On<RequestStopExploitEvent>,
    mut player_state: ResMut<PlayerState>,
    mut ui_state: ResMut<UiState>,
) -> bevy::prelude::Result {
    player_state.active_exploits.retain(|exploit| {
        lock_and_clone!(exploit, id) != evt.exploit_id
    });

    ui_state.active_exploit_windows.retain(|window| {
        lock_and_clone!(window.active_exploit, id) != evt.exploit_id
    });

    Ok(())
}

pub(crate) fn on_request_pause_exploit(
    evt: On<RequestPauseExploitEvent>,
    mut player_state: ResMut<PlayerState>,
) -> bevy::prelude::Result {
    if let Some(exploit) = player_state
        .active_exploits
        .iter_mut()
        .find(|exploit| {
            lock_and_clone!(exploit, id) == evt.exploit_id
        })
    {
        exploit.lock().unwrap().stop_execution();
    }

    Ok(())
}

pub(crate) fn on_request_restart_exploit(
    evt: On<RequestRestartExploitEvent>,
    mut player_state: ResMut<PlayerState>,
) -> bevy::prelude::Result {
    if let Some(exploit) = player_state
        .active_exploits
        .iter_mut()
        .find(|exploit| {
            lock_and_clone!(exploit, id) == evt.exploit_id
        })
    {
        exploit.lock().unwrap().restart();
    }

    Ok(())
}

pub(crate) fn on_request_resume_exploit(
    evt: On<RequestResumeExploitEvent>,
    mut player_state: ResMut<PlayerState>,
) -> bevy::prelude::Result {
    if let Some(exploit) = player_state
        .active_exploits
        .iter_mut()
        .find(|exploit| {
            lock_and_clone!(exploit, id) == evt.exploit_id
        })
    {
        exploit.lock().unwrap().start_execution();
    }

    Ok(())
}

pub(crate) fn on_modify_credits(
    evt: On<ModifyCreditsEvent>,
    mut player_state: ResMut<PlayerState>,
) -> bevy::prelude::Result {
    player_state.credits = player_state.credits.saturating_add_signed(evt.credits as i128);

    if matches!(player_state.progression, TutorialProgression::ExploitServersShown) {
        player_state.progression.advance();
    }

    Ok(())
}

pub(crate) fn on_request_purchase_unlock(
    evt: On<RequestPurchaseUnlockEvent>,
    mut player_state: ResMut<PlayerState>,
) -> bevy::prelude::Result {
    if evt.credit_cost >= player_state.credits {
        // ZJ-TODO: error messaging
        return Ok(());
    }

    player_state.credits -= evt.credit_cost;

    // ZJ-TODO: rather than having a separate list of bools,
    //          it'd be nice to just have a collection of enums

    match evt.unlock {
        PlayerUnlock::ExploitAutoReconnect => {
            player_state.player_unlocks.exploit_auto_reconnect = true;
        }
    }

    Ok(())
}

pub(crate) fn on_script_created(
    evt: On<ScriptCreatedEvent>,
    mut player_state: ResMut<PlayerState>,
) -> bevy::prelude::Result {
    let script = evt.script.to_owned();
    player_state.scripts.push(Arc::new(Mutex::new(script)));

    Ok(())
}

pub(crate) fn tick_active_exploits(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
) {
    // Only tick at a fixed rate
    let time_since_last_tick = web_time::Instant::now().duration_since(player_state.last_tick);
    if time_since_last_tick < TIME_BETWEEN_TICKS {
        return;
    }

    player_state.last_tick = web_time::Instant::now();

    let mut pending_effects = vec![];
    for active_exploit in &player_state.active_exploits {
        let (new_host_effects, new_target_effects) = {
            let mut active_exploit = active_exploit.lock().unwrap();
            // ZJ-TODO: compared allocated speed vs server's current capacity
            //          this should probably be refactored
            let server_speed = active_exploit.clock_allocation_hz;
            let ticks_since_last = (server_speed as f64 * time_since_last_tick.as_secs_f64()).floor() as u64;

            let target_server_speed = active_exploit.target.lock().unwrap().server.lock().unwrap().clock_speed_hz;
            let target_ticks_since_last = (target_server_speed as f64 * time_since_last_tick.as_secs_f64()).floor() as u64;

            active_exploit.tick(&mut commands, ticks_since_last, target_ticks_since_last)
        };

        pending_effects.push((active_exploit.clone(), new_host_effects));
        pending_effects.push((active_exploit.clone(), new_target_effects));
    }

    for (active_exploit, pending_effects) in pending_effects {
        for pending_effect in pending_effects {
            process_algorithm_effect_application(
                &mut commands,
                pending_effect,
                active_exploit.clone(),
            );
        }
    }

    for active_exploit in &player_state.active_exploits {
        let active_exploit = active_exploit.lock().unwrap();
        if matches!(active_exploit.status(), ActiveExploitStatus::Disconnected) {
            if *active_exploit.auto_reconnect.lock().unwrap() {
                commands.trigger(RequestRestartExploitEvent { exploit_id: active_exploit.id })
            }
        }
    }
}

pub(crate) fn process_algorithm_effect_application(
    commands: &mut Commands,
    application: AlgorithmEffectApplication,
    active_exploit: Arc<Mutex<ActiveExploit>>,
) {
    let mut active_exploit = active_exploit.lock().unwrap();
    let from_player_server = {
        let host_server_name = active_exploit.hosting_server.lock().unwrap().name.clone();
        let application_server_name = application.host_server.lock().unwrap().name.clone();

        host_server_name == application_server_name
    };
    match application.effect {
        AlgorithmEffect::Terminate { potency }  => {
            let value = potency.make_value();
            let old_health = active_exploit.connection_current_health.lock().unwrap().clone();
            let new_health = old_health.saturating_sub(value.abs() as u32);
            *active_exploit.connection_current_health.lock().unwrap() = new_health;

            if new_health == 0 {
                active_exploit.stop_execution();
            }
        }
        AlgorithmEffect::Siphon { potency } => {
            let value = potency.make_value();
            let target_server = application.target_server.lock().unwrap();
            let target_stats = &target_server.stats;

            let target_defense = target_stats.value_of(ServerStatType::SiphonResist);
            let siphon_value = (value - target_defense).max(0) as i64;

            commands.trigger(ModifyCreditsEvent {
                credits: siphon_value,
                source: ModificationSource::Script(application.script.lock().unwrap().id.clone()),
            });

            commands.trigger(ExploitEvent {
                active_exploit_id: active_exploit.id,
                from_player_server,
                algorithm_effect: AlgorithmEffect::Siphon { potency },
                potency_roll: Some(value),
                value_after_modification: Some(siphon_value as i32),
            });
        }
        AlgorithmEffect::Exfil { potency } => {
            let value = potency.make_value();
            let target_server = application.target_server.lock().unwrap();
            let target_stats = &target_server.stats;
            let target_defense = target_stats.value_of(ServerStatType::ExfilResist);
            let exfil_value = value - target_defense;
            if exfil_value <= 0 {
                return;
            }

            // ZJ-TODO: pass potency to generator
            let algorithm = AlgorithmGenerator::generate();

            commands.trigger(InventoryItemAdded {
                item: InventoryItem::Algorithm(algorithm),
            });

            commands.trigger(ExploitEvent {
                active_exploit_id: active_exploit.id,
                from_player_server,
                algorithm_effect: AlgorithmEffect::Exfil { potency },
                potency_roll: Some(value),
                value_after_modification: Some(exfil_value),
            });
        }
        AlgorithmEffect::Modify { target, stat, potency } => {
            let server = match target {
                AlgorithmEffectTarget::SelfServer => application.host_server,
                AlgorithmEffectTarget::TargetServer => application.target_server,
            };

            let script_id = application.script.lock().unwrap().id.clone();

            let potency_roll = potency.make_value();

            let purged_stats = server.lock().unwrap().stats.apply_and_purge(
                ServerStatInstance::new(
                    ServerStatSource::Script(script_id),
                    stat.to_owned(),
                    potency_roll
                )
            );

            let new_value = server.lock().unwrap().stats.value_of(stat.to_owned());

            commands.trigger(ExploitEvent {
                active_exploit_id: active_exploit.id,
                from_player_server,
                algorithm_effect: AlgorithmEffect::Modify { target, stat, potency },
                potency_roll: Some(potency_roll),
                value_after_modification: Some(new_value),
            });
        }
        AlgorithmEffect::Purge { target, stat, potency } => {
            let (server, is_self) = match target {
                AlgorithmEffectTarget::SelfServer => (application.host_server, true),
                AlgorithmEffectTarget::TargetServer => (application.target_server, false),
            };

            let mut server = server.lock().unwrap();
            let script_id = active_exploit.script.lock().unwrap().id.clone();

            let server_stat_value = server.stats.modification_of(stat.to_owned());
            if server_stat_value >= 0 && is_self || server_stat_value <= 0 && !is_self {
                return;
            }

            let potency_roll = {
                if is_self {
                    potency.make_value().max(0)
                } else {
                    potency.make_value().min(0)
                }
            };

            let purged_stats = server.stats.apply_and_purge(
                ServerStatInstance::new(
                    ServerStatSource::Script(script_id),
                    stat.to_owned(),
                    potency_roll
                )
            );

            let new_value = server.stats.value_of(stat.to_owned());

            commands.trigger(ExploitEvent {
                active_exploit_id: active_exploit.id,
                from_player_server,
                algorithm_effect: AlgorithmEffect::Purge { target, stat, potency },
                potency_roll: Some(potency_roll),
                value_after_modification: Some(new_value),
            });
        }
    }
}