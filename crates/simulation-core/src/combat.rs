use crate::types::{AttackType, DefenseType};

/// Returns the damage multiplier for a given attack type vs defense type.
///
/// |            | Armor | Agility | Mystical |
/// |------------|-------|---------|----------|
/// | **Blunt**  | 1.25  | 0.85    | 1.10     |
/// | **Pierce** | 0.80  | 1.25    | 0.90     |
/// | **Magic**  | 0.85  | 1.10    | 1.25     |
/// | **Divine** | 1.00  | 0.90    | 1.30     |
pub fn damage_multiplier(attack: AttackType, defense: DefenseType) -> f32 {
    match (attack, defense) {
        (AttackType::Blunt, DefenseType::Armor) => 1.25,
        (AttackType::Blunt, DefenseType::Agility) => 0.85,
        (AttackType::Blunt, DefenseType::Mystical) => 1.10,

        (AttackType::Pierce, DefenseType::Armor) => 0.80,
        (AttackType::Pierce, DefenseType::Agility) => 1.25,
        (AttackType::Pierce, DefenseType::Mystical) => 0.90,

        (AttackType::Magic, DefenseType::Armor) => 0.85,
        (AttackType::Magic, DefenseType::Agility) => 1.10,
        (AttackType::Magic, DefenseType::Mystical) => 1.25,

        (AttackType::Divine, DefenseType::Armor) => 1.00,
        (AttackType::Divine, DefenseType::Agility) => 0.90,
        (AttackType::Divine, DefenseType::Mystical) => 1.30,
    }
}

/// Calculates final damage from base damage, attack type, and target defense type.
pub fn calculate_damage(base_damage: f32, attack: AttackType, defense: DefenseType) -> f32 {
    base_damage * damage_multiplier(attack, defense)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blunt_vs_all_defenses() {
        assert_eq!(damage_multiplier(AttackType::Blunt, DefenseType::Armor), 1.25);
        assert_eq!(damage_multiplier(AttackType::Blunt, DefenseType::Agility), 0.85);
        assert_eq!(damage_multiplier(AttackType::Blunt, DefenseType::Mystical), 1.10);
    }

    #[test]
    fn pierce_vs_all_defenses() {
        assert_eq!(damage_multiplier(AttackType::Pierce, DefenseType::Armor), 0.80);
        assert_eq!(damage_multiplier(AttackType::Pierce, DefenseType::Agility), 1.25);
        assert_eq!(damage_multiplier(AttackType::Pierce, DefenseType::Mystical), 0.90);
    }

    #[test]
    fn magic_vs_all_defenses() {
        assert_eq!(damage_multiplier(AttackType::Magic, DefenseType::Armor), 0.85);
        assert_eq!(damage_multiplier(AttackType::Magic, DefenseType::Agility), 1.10);
        assert_eq!(damage_multiplier(AttackType::Magic, DefenseType::Mystical), 1.25);
    }

    #[test]
    fn divine_vs_all_defenses() {
        assert_eq!(damage_multiplier(AttackType::Divine, DefenseType::Armor), 1.00);
        assert_eq!(damage_multiplier(AttackType::Divine, DefenseType::Agility), 0.90);
        assert_eq!(damage_multiplier(AttackType::Divine, DefenseType::Mystical), 1.30);
    }

    #[test]
    fn calculate_damage_applies_multiplier() {
        let dmg = calculate_damage(100.0, AttackType::Blunt, DefenseType::Armor);
        assert!((dmg - 125.0).abs() < 0.001);

        let dmg = calculate_damage(50.0, AttackType::Pierce, DefenseType::Armor);
        assert!((dmg - 40.0).abs() < 0.001);
    }

    #[test]
    fn calculate_damage_zero_base() {
        let dmg = calculate_damage(0.0, AttackType::Magic, DefenseType::Mystical);
        assert_eq!(dmg, 0.0);
    }
}
