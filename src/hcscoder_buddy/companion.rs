//! hcscoder Companion Module
//!
//! Buddy companion data structures.
//! Zero telemetry, no phone-home logic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Buddy rarity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuddyRarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl std::fmt::Display for BuddyRarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Common => write!(f, "Common"),
            Self::Rare => write!(f, "Rare"),
            Self::Epic => write!(f, "Epic"),
            Self::Legendary => write!(f, "Legendary"),
        }
    }
}

/// Buddy Companion structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HcscoderCompanion {
    pub name: String,
    pub rarity: BuddyRarity,
    pub power: u32,
    pub personality: String,
    pub bio: String,
    pub created_at: DateTime<Utc>,
}

impl HcscoderCompanion {
    /// Get rarity icon
    pub fn rarity_icon(&self) -> &'static str {
        match self.rarity {
            BuddyRarity::Common => "⚪",
            BuddyRarity::Rare => "🔵",
            BuddyRarity::Epic => "🟣",
            BuddyRarity::Legendary => "🟡",
        }
    }

    /// Get power tier description
    pub fn power_tier(&self) -> &'static str {
        match self.power {
            p if p < 200 => "Novice",
            p if p < 600 => "Apprentice",
            p if p < 1500 => "Adept",
            p if p < 3000 => "Expert",
            p if p < 8000 => "Master",
            _ => "Grand Master",
        }
    }

    /// Check if this is a special edition companion
    pub fn is_special(&self) -> bool {
        matches!(self.rarity, BuddyRarity::Epic | BuddyRarity::Legendary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rarity_display() {
        assert_eq!(BuddyRarity::Common.to_string(), "Common");
        assert_eq!(BuddyRarity::Legendary.to_string(), "Legendary");
    }

    #[test]
    fn test_rarity_icon() {
        let common = HcscoderCompanion {
            name: "Test".to_string(),
            rarity: BuddyRarity::Common,
            power: 100,
            personality: "Test".to_string(),
            bio: "Test".to_string(),
            created_at: Utc::now(),
        };

        assert_eq!(common.rarity_icon(), "⚪");
    }

    #[test]
    fn test_power_tier() {
        let mut companion = HcscoderCompanion {
            name: "Test".to_string(),
            rarity: BuddyRarity::Common,
            power: 100,
            personality: "Test".to_string(),
            bio: "Test".to_string(),
            created_at: Utc::now(),
        };

        assert_eq!(companion.power_tier(), "Novice");

        companion.power = 500;
        assert_eq!(companion.power_tier(), "Apprentice");

        companion.power = 10000;
        assert_eq!(companion.power_tier(), "Grand Master");
    }
}
