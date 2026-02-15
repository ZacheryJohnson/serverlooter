use bevy::prelude::Resource;
use crate::ui::panel::exploit::ExploitPanel;
use crate::ui::panel::market::MarketPanel;
use crate::ui::panel::script::ScriptsPanel;
use crate::ui::panel::server::ServersPanel;
use crate::ui::window::active_exploit::ActiveExploitWindow;

pub(crate) enum ActivePanel {
    Home,
    Market,
    Servers,
    Scripts,
    Exploit,
}

#[derive(Resource)]
pub(crate) struct UiState {
    pub active_panel: ActivePanel,
    pub market_panel_state: MarketPanel,
    pub server_panel_state: ServersPanel,
    pub scripts_panel_state: ScriptsPanel,
    pub exploit_panel_state: ExploitPanel,
    pub active_exploit_windows: Vec<ActiveExploitWindow>,
}