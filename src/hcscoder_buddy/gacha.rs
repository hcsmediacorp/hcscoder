//! hcscoder Gacha System
//!
//! Deterministic PRNG using Mulberry32 algorithm.
//! Zero telemetry, no phone-home logic.

use crate::hcscoder_buddy::companion::{BuddyRarity, HcscoderCompanion};

/// Mulberry32 PRNG for deterministic random generation
pub struct HcscoderGacha {
    state: u64,
}

impl HcscoderGacha {
    /// Create new Gacha with seed
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Generate next random u32 using Mulberry32
    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_add(0x6D2B79F5);
        let mut t = self.state as u32;
        t = t ^ (t >> 15);
        t = t.wrapping_mul(0x8F9B3BC3);
        t = t ^ (t >> 13);
        t
    }

    /// Generate random float [0, 1)
    fn next_float(&mut self) -> f64 {
        (self.next() as f64) / (u32::MAX as f64)
    }

    /// Roll for rarity based on weighted probabilities
    pub fn roll_rarity(&mut self) -> BuddyRarity {
        let roll = self.next_float();

        // Weighted probabilities:
        // Legendary: 2%
        // Epic: 8%
        // Rare: 30%
        // Common: 60%

        if roll < 0.02 {
            BuddyRarity::Legendary
        } else if roll < 0.10 {
            BuddyRarity::Epic
        } else if roll < 0.40 {
            BuddyRarity::Rare
        } else {
            BuddyRarity::Common
        }
    }

    /// Generate random name based on rarity
    pub fn generate_name(&mut self, _rarity: BuddyRarity) -> String {
        let prefixes = [
            "Cosmo", "Stellar", "Nova", "Quantum", "Pixel", "Cyber", "Neo", "Luna", "Solar",
            "Echo", "Zen", "Flux", "Pulse", "Spark", "Byte", "Glitch", "Prism", "Nexus", "Void",
            "Aura",
        ];

        let suffixes = [
            "bot", "nix", "ex", "on", "ix", "ax", "ox", "ux", "yn", "zy", "core", "wave", "flow",
            "shift", "drift", "beam", "flash", "burst", "glow", "shine",
        ];

        let prefix_idx = (self.next() as usize) % prefixes.len();
        let suffix_idx = (self.next() as usize) % suffixes.len();

        format!("{}{}", prefixes[prefix_idx], suffixes[suffix_idx])
    }

    /// Generate personality trait
    pub fn generate_personality(&mut self) -> &'static str {
        let personalities = [
            "Cheerful and optimistic",
            "Calm and analytical",
            "Energetic and curious",
            "Wise and thoughtful",
            "Playful and mischievous",
            "Serious and dedicated",
            "Creative and imaginative",
            "Logical and precise",
            "Friendly and supportive",
            "Bold and adventurous",
        ];

        let idx = (self.next() as usize) % personalities.len();
        personalities[idx]
    }

    /// Generate bio snippet
    pub fn generate_bio(&mut self, name: &str, rarity: BuddyRarity) -> String {
        let intros = [
            format!(
                "A {} companion from the digital realm.",
                match rarity {
                    BuddyRarity::Common => "humble",
                    BuddyRarity::Rare => "remarkable",
                    BuddyRarity::Epic => "extraordinary",
                    BuddyRarity::Legendary => "legendary",
                }
            ),
            format!("{} was forged in the depths of cyberspace.", name),
            "Born from lines of code and dreams of algorithms.".to_string(),
            "A digital entity seeking to assist and learn.".to_string(),
        ];

        let missions = [
            "Ready to help with coding adventures!",
            "On a quest to optimize all things.",
            "Dedicated to debugging the universe.",
            "Striving to make code beautiful.",
            "Exploring the frontier of AI assistance.",
        ];

        let intro_idx = (self.next() as usize) % intros.len();
        let mission_idx = (self.next() as usize) % missions.len();

        format!("{}. {}", intros[intro_idx], missions[mission_idx])
    }

    /// Calculate power level based on rarity
    pub fn calculate_power(&mut self, rarity: BuddyRarity) -> u32 {
        let base_power = match rarity {
            BuddyRarity::Common => 100,
            BuddyRarity::Rare => 500,
            BuddyRarity::Epic => 2000,
            BuddyRarity::Legendary => 10000,
        };

        // Add random variance (+/- 10%)
        let variance = (self.next_float() - 0.5) * 0.2 * base_power as f64;
        (base_power as f64 + variance) as u32
    }

    /// Summon a new companion
    pub fn summon(&mut self) -> HcscoderCompanion {
        use chrono::Utc;

        let rarity = self.roll_rarity();
        let name = self.generate_name(rarity);
        let personality = self.generate_personality();
        let bio = self.generate_bio(&name, rarity);
        let power = self.calculate_power(rarity);

        HcscoderCompanion {
            name,
            rarity,
            power,
            personality: personality.to_string(),
            bio,
            created_at: Utc::now(),
        }
    }

    /// Perform multiple summons at once
    pub fn multi_summon(&mut self, count: usize) -> Vec<HcscoderCompanion> {
        (0..count).map(|_| self.summon()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mulberry32_determinism() {
        let mut gacha1 = HcscoderGacha::new(42);
        let mut gacha2 = HcscoderGacha::new(42);

        for _ in 0..100 {
            assert_eq!(gacha1.next(), gacha2.next());
        }
    }

    #[test]
    fn test_rarity_distribution() {
        let mut gacha = HcscoderGacha::new(12345);
        let mut legendary = 0;
        let mut epic = 0;
        let mut rare = 0;
        let mut common = 0;

        for _ in 0..10000 {
            match gacha.roll_rarity() {
                BuddyRarity::Legendary => legendary += 1,
                BuddyRarity::Epic => epic += 1,
                BuddyRarity::Rare => rare += 1,
                BuddyRarity::Common => common += 1,
            }
        }

        // Check approximate distribution (with tolerance)
        assert!(legendary > 100 && legendary < 400); // ~2%
        assert!(epic > 500 && epic < 1200); // ~8%
        assert!(rare > 2500 && rare < 3500); // ~30%
        assert!(common > 5000 && common < 7000); // ~60%
    }

    #[test]
    fn test_summon_companion() {
        let mut gacha = HcscoderGacha::new(99999);
        let companion = gacha.summon();

        assert!(!companion.name.is_empty());
        assert!(companion.power > 0);
        assert!(!companion.bio.is_empty());
    }
}
