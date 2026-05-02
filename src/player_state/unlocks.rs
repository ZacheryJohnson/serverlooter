use crate::l10n::Localizable;
use crate::l10n::message_id::MessageId;

#[repr(u8)] // this means we can only have 256 unlocks
#[derive(Debug, Clone, Copy)]
pub enum PlayerUnlock {
    ExploitAutoReconnect,
}

impl PlayerUnlock {
    /// Returns an ordered vector of market-purchasable unlocks and their credit costs.
    pub fn market_unlockable_unlocks() -> Vec<(PlayerUnlock, u128)> {
        vec![
            (PlayerUnlock::ExploitAutoReconnect, 150)
        ]
    }

    pub fn description(&self) -> PlayerUnlockDescription {
        PlayerUnlockDescription(*self)
    }
}

impl Localizable for PlayerUnlock {
    fn loc_key(&self) -> MessageId {
        match self {
            PlayerUnlock::ExploitAutoReconnect => MessageId::PlayerUnlockExploitAutoreconnectTitle,
        }
    }
}

pub struct PlayerUnlockDescription(PlayerUnlock);

impl Localizable for PlayerUnlockDescription {
    fn loc_key(&self) -> MessageId {
        match self.0 {
            PlayerUnlock::ExploitAutoReconnect => MessageId::PlayerUnlockExploitAutoreconnectDescription,
        }
    }
}

pub struct PlayerUnlocks {
    unlock_bitfield: u128,
}

impl PlayerUnlocks {
    pub fn empty() -> Self {
        PlayerUnlocks {
            unlock_bitfield: 0,
        }
    }

    pub fn unlock(&mut self, unlock: PlayerUnlock) {
        let bit_mask = (1 << unlock as u8) as u128;
        self.unlock_bitfield |= bit_mask;
    }

    pub fn is_unlocked(&self, unlock: PlayerUnlock) -> bool {
        let bit_mask = (1 << unlock as u8) as u128;
        self.unlock_bitfield & bit_mask == bit_mask
    }
}