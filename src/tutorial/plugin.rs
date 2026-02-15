use bevy::app::{App, Plugin};
use bevy_egui::EguiPrimaryContextPass;
use crate::tutorial::systems::{tutorial_on_script_created, tutorial_ui_system};

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(tutorial_on_script_created)
            .add_systems(EguiPrimaryContextPass, tutorial_ui_system);
    }
}