pub enum TutorialProgression {
    /// The option to start with a tutorial hasn't been presented to the player yet.
    None,

    /// The player skipped the tutorial.
    /// This is equivalent to Complete, but allows us to be condescending if the player needs it.
    Skipped,

    /// The player has completed the tutorial.
    Complete,

    /// The player has started the tutorial, but has not yet completed any steps.
    Start,

    /// The server section is added to the menu sidebar,
    /// and the user is prompted to click the only server tab within.
    ServersTabIntroduced,

    ServerClicked,

    /// The develop section and scripts tab is added to the menu sidebar,
    /// and the user is prompted to click the "scripts" tab within.
    DevelopScriptsIntroduced,

    ScriptClicked,

    /// The market tab is added to the menu sidebar.
    MarketTabIntroduced,

    MarketTabClicked,

    /// The exploit tab is added to the menu sidebar.
    ExploitTabIntroduced,

    ExploitServersShown,

    ExploitCreditsReceived,

    // ZJ-TODO: finish tutorial
    // ExploitCorpASuccess,
    //
    // ExploitCorpBClicked,
    //
    // MarketAlgorithmPrompted,
}

impl TutorialProgression {
    pub(crate) fn show_servers_tab(&self) -> bool {
        match self {
            TutorialProgression::None => false,
            TutorialProgression::Start => false,
            _ => true,
        }
    }

    pub(crate) fn show_develop_tab(&self) -> bool {
        self.show_servers_tab() && match self {
            TutorialProgression::ServersTabIntroduced => false,
            TutorialProgression::ServerClicked => false,
            _ => true,
        }
    }

    pub(crate) fn show_market_tab(&self) -> bool {
        self.show_develop_tab() && match self {
            TutorialProgression::DevelopScriptsIntroduced => false,
            TutorialProgression::ScriptClicked => false,
            _ => true,
        }
    }

    pub(crate) fn show_exploit_tab(&self) -> bool {
        self.show_market_tab() && match self {
            TutorialProgression::MarketTabIntroduced => false,
            TutorialProgression::MarketTabClicked => false,
            _ => true,
        }
    }

    pub(crate) fn is_complete(&self) -> bool {
        match self {
            TutorialProgression::Complete | TutorialProgression::Skipped => true,
            _ => false,
        }
    }

    pub fn advance(&mut self) {
        match self {
            TutorialProgression::None => {},
            TutorialProgression::Skipped => {},
            TutorialProgression::Complete => {},
            TutorialProgression::Start => *self = TutorialProgression::ServersTabIntroduced,
            TutorialProgression::ServersTabIntroduced => *self = TutorialProgression::ServerClicked,
            TutorialProgression::ServerClicked => *self = TutorialProgression::DevelopScriptsIntroduced,
            TutorialProgression::DevelopScriptsIntroduced => *self = TutorialProgression::ScriptClicked,
            TutorialProgression::ScriptClicked => *self = TutorialProgression::MarketTabIntroduced,
            TutorialProgression::MarketTabIntroduced => *self = TutorialProgression::MarketTabClicked,
            TutorialProgression::MarketTabClicked => *self = TutorialProgression::ExploitTabIntroduced,
            TutorialProgression::ExploitTabIntroduced => *self = TutorialProgression::ExploitServersShown,
            TutorialProgression::ExploitServersShown => *self = TutorialProgression::ExploitCreditsReceived,
            TutorialProgression::ExploitCreditsReceived => *self = TutorialProgression::Complete,

            // ZJ-TODO: finish tutorial
            // TutorialProgression::ExploitCreditsReceived => *self = TutorialProgression::ExploitCorpASuccess,
            // TutorialProgression::ExploitCorpASuccess => *self = TutorialProgression::ExploitCorpBClicked,
            // TutorialProgression::ExploitCorpBClicked => *self = TutorialProgression::MarketAlgorithmPrompted,
            // TutorialProgression::MarketAlgorithmPrompted => *self = TutorialProgression::Complete,
        }
    }
}