//! hcscoder Buddy System Module
//!
//! Gacha/companion system with deterministic PRNG.
//! Zero telemetry, no phone-home logic.

pub mod companion;
pub mod gacha;

use crate::hcscoder_buddy::companion::BuddyRarity;
use crate::hcscoder_buddy::companion::HcscoderCompanion;
use crate::hcscoder_buddy::gacha::HcscoderGacha;
use anyhow::Result;

/// Summon a new Buddy companion
pub async fn summon_buddy() -> Result<HcscoderCompanion> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Use current time as seed for randomness
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow::anyhow!("system clock: {}", e))?
        .as_millis() as u64;

    let mut gacha = HcscoderGacha::new(seed);
    let companion = gacha.summon();

    println!("\n🎉 Summoning Result!");
    println!("{}", "=".repeat(40));
    println!("✨ Name:     {}", companion.name);
    println!("🏷️  Rarity:   {:?}", companion.rarity);
    println!("📊 Power:     {}", companion.power);
    println!("💫 Personality: {}", companion.personality);
    println!("📝 Bio:       {}", companion.bio);
    println!("{}", "=".repeat(40));

    // Save to buddy collection
    save_companion(&companion)?;

    Ok(companion)
}

/// List all owned Buddies
pub fn list_buddies() -> Result<()> {
    let buddies = load_all_companions()?;

    if buddies.is_empty() {
        println!("\n📭 No Buddies yet! Use 'hcscoder buddy summon' to get your first companion.");
        return Ok(());
    }

    println!("\n🎒 Your Buddy Collection");
    println!("{}", "=".repeat(50));

    for (i, buddy) in buddies.iter().enumerate() {
        let rarity_icon = match buddy.rarity {
            BuddyRarity::Common => "⚪",
            BuddyRarity::Rare => "🔵",
            BuddyRarity::Epic => "🟣",
            BuddyRarity::Legendary => "🟡",
        };

        println!(
            "{}. {} {} (Power: {})",
            i + 1,
            rarity_icon,
            buddy.name,
            buddy.power
        );
    }

    println!("{}", "=".repeat(50));
    println!("Total: {} Buddies", buddies.len());

    Ok(())
}

/// Show details of a specific Buddy
pub fn show_buddy(name: &str) -> Result<()> {
    let buddies = load_all_companions()?;

    let buddy = buddies
        .iter()
        .find(|b| b.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| anyhow::anyhow!("Buddy '{}' not found", name))?;

    println!("\n📋 Buddy Details");
    println!("{}", "=".repeat(40));
    println!("Name:         {}", buddy.name);
    println!("Rarity:       {:?}", buddy.rarity);
    println!("Power:        {}", buddy.power);
    println!("Personality:  {}", buddy.personality);
    println!("Bio:          {}", buddy.bio);
    println!("Created:      {:?}", buddy.created_at);
    println!("{}", "=".repeat(40));

    Ok(())
}

/// Release a Buddy from collection
pub fn release_buddy(name: &str) -> Result<()> {
    let mut buddies = load_all_companions()?;

    let initial_len = buddies.len();
    buddies.retain(|b| !b.name.eq_ignore_ascii_case(name));

    if buddies.len() == initial_len {
        anyhow::bail!("Buddy '{}' not found", name);
    }

    save_all_companions(&buddies)?;
    println!("✅ Released {}. Farewell!", name);

    Ok(())
}

/// Buddy storage path
fn buddy_storage_path() -> Result<std::path::PathBuf> {
    let config_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?
        .join(".hcscoder")
        .join("buddies");

    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("collection.json"))
}

/// Save a single companion
fn save_companion(companion: &HcscoderCompanion) -> Result<()> {
    use std::fs;

    let path = buddy_storage_path()?;
    let mut buddies = load_all_companions()?;

    // Check for duplicates
    if buddies.iter().any(|b| b.name == companion.name) {
        return Err(anyhow::anyhow!("Buddy '{}' already exists", companion.name));
    }

    buddies.push(companion.clone());

    let json = serde_json::to_string_pretty(&buddies)?;
    fs::write(path, json)?;

    Ok(())
}

/// Load all companions
fn load_all_companions() -> Result<Vec<HcscoderCompanion>> {
    let path = buddy_storage_path()?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(path)?;
    let buddies: Vec<HcscoderCompanion> = serde_json::from_str(&content)?;

    Ok(buddies)
}

/// Save all companions
fn save_all_companions(buddies: &[HcscoderCompanion]) -> Result<()> {
    use std::fs;

    let path = buddy_storage_path()?;
    let json = serde_json::to_string_pretty(buddies)?;
    fs::write(path, json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gacha_determinism() {
        let mut gacha1 = HcscoderGacha::new(12345);
        let mut gacha2 = HcscoderGacha::new(12345);

        let buddy1 = gacha1.summon();
        let buddy2 = gacha2.summon();

        assert_eq!(buddy1.name, buddy2.name);
        assert_eq!(buddy1.rarity, buddy2.rarity);
        assert_eq!(buddy1.power, buddy2.power);
    }
}
