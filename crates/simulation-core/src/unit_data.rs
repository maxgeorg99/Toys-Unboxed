use serde::Deserialize;

use crate::types::{AttackType, DefenseType};

#[derive(Debug, Clone, Deserialize)]
pub struct UnitDef {
    pub id: String,
    pub name: String,
    pub sprite_path: String,
    pub idle_sprite_path: String,
    pub idle_frame_count: usize,
    pub attack_sprite_path: String,
    #[serde(default)]
    pub attack_sound: String,
    #[serde(default)]
    pub avatar_path: String,
    pub base_health: f32,
    pub base_speed: f32,
    #[serde(default)]
    pub damage_to_base: u32,
    #[serde(default)]
    pub gold_reward: u32,
    pub frame_count: usize,
    pub attack_frame_count: usize,
    pub frame_size: [u32; 2],
    pub attack_type: AttackType,
    pub defense_type: DefenseType,
    #[serde(default)]
    pub base_damage: f32,
    #[serde(default)]
    pub attack_range: f32,
    #[serde(default)]
    pub attack_cooldown: f32,
    #[serde(default)]
    pub meat_cost: u32,
    #[serde(default)]
    pub recruitable: bool,
    #[serde(default)]
    pub faction: String,
    #[serde(default = "default_one")]
    pub troops_width: u32,
    #[serde(default = "default_one")]
    pub troops_height: u32,
    #[serde(default)]
    pub death_sprite_path: String,
    #[serde(default)]
    pub death_frame_count: usize,
    #[serde(default)]
    pub death_frame_size: Option<[u32; 2]>,
    #[serde(default)]
    pub capacity_cost: Option<u32>,
}

fn default_one() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnitsConfig {
    pub units: Vec<UnitDef>,
}

impl UnitsConfig {
    pub fn load_from_str(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    pub fn find_by_id(&self, id: &str) -> Option<&UnitDef> {
        self.units.iter().find(|u| u.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SKULL_TOML: &str = r#"
[[units]]
id = "skull"
name = "Skull"
sprite_path = "Enemies/Skull/Skull_Run.png"
idle_sprite_path = "Enemies/Skull/Skull_Idle.png"
idle_frame_count = 4
attack_sprite_path = "Enemies/Skull/Skull_Attack.png"
attack_sound = "Sound Effects/Assasin.mp3"
avatar_path = "Enemies/Skull/Skull Avatar.png"
base_health = 70.0
base_speed = 45.0
damage_to_base = 2
gold_reward = 12
frame_count = 6
attack_frame_count = 7
frame_size = [192, 192]
attack_type = "magic"
defense_type = "mystical"
troops_width = 3
troops_height = 10
death_sprite_path = "Enemies/Skull/Skull_Death.png"
death_frame_count = 5
"#;

    #[test]
    fn parse_skull_unit() {
        let config = UnitsConfig::load_from_str(SKULL_TOML).unwrap();
        assert_eq!(config.units.len(), 1);

        let skull = &config.units[0];
        assert_eq!(skull.id, "skull");
        assert_eq!(skull.name, "Skull");
        assert_eq!(skull.troops_width, 3);
        assert_eq!(skull.troops_height, 10);
        assert_eq!(skull.base_health, 70.0);
        assert_eq!(skull.frame_size, [192, 192]);
        assert_eq!(skull.attack_type, AttackType::Magic);
        assert_eq!(skull.defense_type, DefenseType::Mystical);
        assert_eq!(skull.idle_frame_count, 4);
        assert_eq!(skull.frame_count, 6);
        assert_eq!(skull.attack_frame_count, 7);
        assert_eq!(skull.death_frame_count, 5);
    }

    #[test]
    fn find_by_id() {
        let config = UnitsConfig::load_from_str(SKULL_TOML).unwrap();
        assert!(config.find_by_id("skull").is_some());
        assert!(config.find_by_id("nonexistent").is_none());
    }

    #[test]
    fn parse_unit_with_optional_fields() {
        let toml = r#"
[[units]]
id = "warrior"
name = "Red Warrior"
sprite_path = "Units/Red Units/Warrior/Warrior_Run.png"
idle_sprite_path = "Units/Red Units/Warrior/Warrior_Idle.png"
idle_frame_count = 4
attack_sprite_path = "Units/Red Units/Warrior/Warrior_Attack1.png"
base_health = 50.0
base_speed = 50.0
frame_count = 6
attack_frame_count = 4
frame_size = [192, 192]
attack_type = "blunt"
defense_type = "armor"
base_damage = 20.0
attack_range = 22.0
attack_cooldown = 1.0
meat_cost = 5
recruitable = true
troops_width = 1
troops_height = 2
death_sprite_path = "Units/Red Units/Warrior/Warrior_Death.png"
death_frame_count = 4
death_frame_size = [128, 128]
"#;
        let config = UnitsConfig::load_from_str(toml).unwrap();
        let warrior = &config.units[0];
        assert_eq!(warrior.death_frame_size, Some([128, 128]));
        assert!(warrior.recruitable);
        assert_eq!(warrior.meat_cost, 5);
    }
}
