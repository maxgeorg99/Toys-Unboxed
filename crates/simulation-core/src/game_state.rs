use crate::combat::calculate_damage;
use crate::formation::Formation;
use crate::types::{AttackType, GamePhase, PlayerId, SimUnitId};
use crate::unit_data::UnitsConfig;

const MELEE_RANGE_THRESHOLD: f32 = 40.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimState {
    Idle,
    Run,
    Attack,
    Death,
}

#[derive(Debug, Clone)]
pub struct SimUnit {
    pub id: SimUnitId,
    pub def_id: String,
    pub owner: PlayerId,
    pub health: f32,
    pub max_health: f32,
    pub x: f32,
    pub y: f32,
    pub is_alive: bool,
    pub animation_state: AnimState,
    pub attack_cooldown_remaining: f32,
    pub target: Option<SimUnitId>,
}

#[derive(Debug, Clone)]
pub struct SimProjectile {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub target_id: SimUnitId,
    pub speed: f32,
    pub damage: f32,
    pub attack_type: AttackType,
    pub owner: PlayerId,
    pub source_def_id: String,
}

pub enum Command {
    SpawnTroop(SpawnTroopCommand),
    StartBattle,
}

pub struct SpawnTroopCommand {
    pub unit_id: String,
    pub owner: PlayerId,
    pub center_x: f32,
    pub center_y: f32,
    pub spacing: f32,
}

pub struct GameState {
    pub phase: GamePhase,
    pub units: Vec<SimUnit>,
    pub projectiles: Vec<SimProjectile>,
    next_unit_id: u64,
    next_projectile_id: u64,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            phase: GamePhase::Placement,
            units: Vec::new(),
            projectiles: Vec::new(),
            next_unit_id: 0,
            next_projectile_id: 0,
        }
    }

    pub fn add_unit(&mut self, def_id: &str, owner: PlayerId, x: f32, y: f32, config: &UnitsConfig) -> SimUnitId {
        let def = config.find_by_id(def_id).expect("unknown unit id");
        let id = SimUnitId(self.next_unit_id);
        self.next_unit_id += 1;
        self.units.push(SimUnit {
            id,
            def_id: def_id.to_string(),
            owner,
            health: def.base_health,
            max_health: def.base_health,
            x,
            y,
            is_alive: true,
            animation_state: AnimState::Idle,
            attack_cooldown_remaining: 0.0,
            target: None,
        });
        id
    }

    pub fn execute(&mut self, command: Command, config: &UnitsConfig) {
        match command {
            Command::SpawnTroop(cmd) => self.spawn_troop(cmd, config),
            Command::StartBattle => {
                if self.phase == GamePhase::Placement {
                    self.phase = GamePhase::Battle;
                }
            }
        }
    }

    fn spawn_troop(&mut self, cmd: SpawnTroopCommand, config: &UnitsConfig) {
        let def = config
            .find_by_id(&cmd.unit_id)
            .expect("unknown unit id");

        let formation = Formation::new(def.troops_width, def.troops_height, cmd.spacing);

        for (ox, oy) in formation.positions() {
            let id = SimUnitId(self.next_unit_id);
            self.next_unit_id += 1;

            self.units.push(SimUnit {
                id,
                def_id: cmd.unit_id.clone(),
                owner: cmd.owner,
                health: def.base_health,
                max_health: def.base_health,
                x: cmd.center_x + ox,
                y: cmd.center_y + oy,
                is_alive: true,
                animation_state: AnimState::Idle,
                attack_cooldown_remaining: 0.0,
                target: None,
            });
        }
    }

    pub fn tick(&mut self, dt: f32, config: &UnitsConfig) {
        if self.phase != GamePhase::Battle {
            return;
        }

        let unit_count = self.units.len();

        // Step 1: Decrement cooldowns for all alive units
        for unit in self.units.iter_mut() {
            if unit.is_alive {
                unit.attack_cooldown_remaining = (unit.attack_cooldown_remaining - dt).max(0.0);
            }
        }

        // Step 2 & 3: Target selection and movement
        // We need to find the nearest enemy for each unit, then decide attack vs move.
        // Collect decisions first to avoid borrow conflicts.
        struct UnitAction {
            target: Option<SimUnitId>,
            move_dx: f32,
            move_dy: f32,
            anim: AnimState,
        }

        let mut actions: Vec<UnitAction> = Vec::with_capacity(unit_count);

        for i in 0..unit_count {
            let unit = &self.units[i];
            if !unit.is_alive {
                actions.push(UnitAction {
                    target: None,
                    move_dx: 0.0,
                    move_dy: 0.0,
                    anim: AnimState::Death,
                });
                continue;
            }

            let def = config.find_by_id(&unit.def_id).expect("unknown unit def");

            // Find nearest alive enemy (different owner)
            let mut nearest: Option<(usize, f32)> = None;
            for j in 0..unit_count {
                if i == j {
                    continue;
                }
                let other = &self.units[j];
                if !other.is_alive || other.owner == unit.owner {
                    continue;
                }
                let dx = other.x - unit.x;
                let dy = other.y - unit.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if let Some((_, best_dist)) = nearest {
                    if dist < best_dist {
                        nearest = Some((j, dist));
                    }
                } else {
                    nearest = Some((j, dist));
                }
            }

            match nearest {
                Some((target_idx, dist)) if dist <= def.attack_range => {
                    // In range — attack
                    actions.push(UnitAction {
                        target: Some(self.units[target_idx].id),
                        move_dx: 0.0,
                        move_dy: 0.0,
                        anim: AnimState::Attack,
                    });
                }
                Some((target_idx, dist)) => {
                    // Out of range — move toward enemy
                    let enemy = &self.units[target_idx];
                    let dx = enemy.x - unit.x;
                    let dy = enemy.y - unit.y;
                    let move_amount = (def.base_speed * dt).min(dist);
                    let move_dx = (dx / dist) * move_amount;
                    let move_dy = (dy / dist) * move_amount;
                    actions.push(UnitAction {
                        target: None,
                        move_dx,
                        move_dy,
                        anim: AnimState::Run,
                    });
                }
                None => {
                    // No enemies at all
                    actions.push(UnitAction {
                        target: None,
                        move_dx: 0.0,
                        move_dy: 0.0,
                        anim: AnimState::Idle,
                    });
                }
            }
        }

        // Apply movement and targeting
        for (i, action) in actions.iter().enumerate() {
            let unit = &mut self.units[i];
            if !unit.is_alive {
                continue;
            }
            unit.x += action.move_dx;
            unit.y += action.move_dy;
            unit.target = action.target;
            unit.animation_state = action.anim;
        }

        // Step 4: Attack execution — melee applies damage, ranged spawns projectiles
        struct DamageEvent {
            target_id: SimUnitId,
            damage: f32,
        }

        let mut damage_events: Vec<DamageEvent> = Vec::new();

        for i in 0..unit_count {
            let unit = &self.units[i];
            if !unit.is_alive || unit.attack_cooldown_remaining > 0.0 {
                continue;
            }

            let Some(target_id) = unit.target else {
                continue;
            };

            let attacker_def = config.find_by_id(&unit.def_id).expect("unknown unit def");
            let is_ranged = attacker_def.attack_range > MELEE_RANGE_THRESHOLD;

            let target_def_id = self.units.iter()
                .find(|u| u.id == target_id)
                .map(|u| u.def_id.clone());

            let Some(target_def_id) = target_def_id else {
                continue;
            };

            let target_def = config.find_by_id(&target_def_id).expect("unknown unit def");

            let dmg = calculate_damage(
                attacker_def.base_damage,
                attacker_def.attack_type,
                target_def.defense_type,
            );

            if is_ranged && attacker_def.projectile_speed > 0.0 {
                self.projectiles.push(SimProjectile {
                    id: self.next_projectile_id,
                    x: unit.x,
                    y: unit.y,
                    target_id,
                    speed: attacker_def.projectile_speed,
                    damage: dmg,
                    attack_type: attacker_def.attack_type,
                    owner: unit.owner,
                    source_def_id: unit.def_id.clone(),
                });
                self.next_projectile_id += 1;
            } else {
                damage_events.push(DamageEvent {
                    target_id,
                    damage: dmg,
                });
            }
        }

        // Reset cooldowns for units that attacked
        for i in 0..unit_count {
            let unit = &self.units[i];
            if !unit.is_alive || unit.attack_cooldown_remaining > 0.0 {
                continue;
            }
            if unit.target.is_some() {
                let def = config.find_by_id(&unit.def_id).expect("unknown unit def");
                self.units[i].attack_cooldown_remaining = def.attack_cooldown;
            }
        }

        // Step 4b: Move projectiles toward their targets
        let mut projectile_hits: Vec<DamageEvent> = Vec::new();
        let mut arrived_ids: Vec<u64> = Vec::new();

        self.projectiles.retain_mut(|proj| {
            let target = self.units.iter().find(|u| u.id == proj.target_id);
            let Some(target) = target else {
                return false;
            };
            if !target.is_alive {
                return false;
            }

            let dx = target.x - proj.x;
            let dy = target.y - proj.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 4.0 {
                projectile_hits.push(DamageEvent {
                    target_id: proj.target_id,
                    damage: proj.damage,
                });
                arrived_ids.push(proj.id);
                return false;
            }

            let move_amount = (proj.speed * dt).min(dist);
            proj.x += (dx / dist) * move_amount;
            proj.y += (dy / dist) * move_amount;
            true
        });

        damage_events.extend(projectile_hits);

        // Step 5: Apply damage and handle deaths
        for event in &damage_events {
            if let Some(target) = self.units.iter_mut().find(|u| u.id == event.target_id) {
                if target.is_alive {
                    target.health = (target.health - event.damage).max(0.0);
                    if target.health <= 0.0 {
                        target.is_alive = false;
                        target.animation_state = AnimState::Death;
                    }
                }
            }
        }

        // Clear stale target references (targets that just died)
        // Collect dead unit IDs first to avoid borrow conflicts
        let dead_ids: Vec<SimUnitId> = self.units.iter()
            .filter(|u| !u.is_alive)
            .map(|u| u.id)
            .collect();

        for unit in self.units.iter_mut() {
            if let Some(target_id) = unit.target {
                if dead_ids.contains(&target_id) {
                    unit.target = None;
                }
            }
        }

        // Step 6: Resolution check — if only one (or zero) players have alive units
        let mut alive_owners = std::collections::HashSet::new();
        for unit in &self.units {
            if unit.is_alive {
                alive_owners.insert(unit.owner);
            }
        }

        if alive_owners.len() <= 1 {
            self.phase = GamePhase::Resolution;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PlayerId;
    use crate::unit_data::UnitsConfig;

    const WARRIOR_TOML: &str = r#"
[[units]]
id = "warrior"
name = "Warrior"
sprite_path = ""
idle_sprite_path = ""
idle_frame_count = 1
attack_sprite_path = ""
base_health = 100.0
base_speed = 50.0
frame_count = 1
attack_frame_count = 1
frame_size = [64, 64]
attack_type = "blunt"
defense_type = "armor"
base_damage = 20.0
attack_range = 30.0
attack_cooldown = 1.0
troops_width = 1
troops_height = 1
"#;

    const TWO_UNITS_TOML: &str = r#"
[[units]]
id = "warrior"
name = "Warrior"
sprite_path = ""
idle_sprite_path = ""
idle_frame_count = 1
attack_sprite_path = ""
base_health = 100.0
base_speed = 50.0
frame_count = 1
attack_frame_count = 1
frame_size = [64, 64]
attack_type = "blunt"
defense_type = "armor"
base_damage = 20.0
attack_range = 30.0
attack_cooldown = 1.0
troops_width = 1
troops_height = 1

[[units]]
id = "mage"
name = "Mage"
sprite_path = ""
idle_sprite_path = ""
idle_frame_count = 1
attack_sprite_path = ""
base_health = 60.0
base_speed = 40.0
frame_count = 1
attack_frame_count = 1
frame_size = [64, 64]
attack_type = "magic"
defense_type = "mystical"
base_damage = 30.0
attack_range = 50.0
attack_cooldown = 1.5
troops_width = 1
troops_height = 1
"#;

    fn make_config(toml: &str) -> UnitsConfig {
        UnitsConfig::load_from_str(toml).unwrap()
    }

    fn spawn_one(state: &mut GameState, config: &UnitsConfig, unit_id: &str, owner: PlayerId, x: f32, y: f32) {
        state.execute(
            Command::SpawnTroop(SpawnTroopCommand {
                unit_id: unit_id.into(),
                owner,
                center_x: x,
                center_y: y,
                spacing: 32.0,
            }),
            config,
        );
    }

    // =========================================================================
    // Spawn tests
    // =========================================================================

    #[test]
    fn spawn_troop_creates_correct_count() {
        let toml = r#"
[[units]]
id = "skull"
name = "Skull"
sprite_path = ""
idle_sprite_path = ""
idle_frame_count = 4
attack_sprite_path = ""
base_health = 70.0
base_speed = 45.0
frame_count = 6
attack_frame_count = 7
frame_size = [192, 192]
attack_type = "magic"
defense_type = "mystical"
troops_width = 3
troops_height = 10
"#;
        let config = make_config(toml);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "skull", PlayerId(1), 0.0, 0.0);
        assert_eq!(state.units.len(), 30);
    }

    #[test]
    fn spawned_units_have_correct_health_and_max_health() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        assert_eq!(state.units[0].health, 100.0);
        assert_eq!(state.units[0].max_health, 100.0);
        assert_eq!(state.units[0].attack_cooldown_remaining, 0.0);
        assert!(state.units[0].target.is_none());
    }

    #[test]
    fn spawned_units_have_unique_ids() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 100.0, 0.0);
        let mut ids: Vec<u64> = state.units.iter().map(|u| u.id.0).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 2);
    }

    // =========================================================================
    // Phase transition tests
    // =========================================================================

    #[test]
    fn start_battle_transitions_phase() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        assert_eq!(state.phase, GamePhase::Placement);
        state.execute(Command::StartBattle, &config);
        assert_eq!(state.phase, GamePhase::Battle);
    }

    #[test]
    fn tick_is_noop_during_placement() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 100.0, 0.0);
        let x_before = state.units[0].x;
        state.tick(1.0, &config);
        assert_eq!(state.units[0].x, x_before);
        assert_eq!(state.phase, GamePhase::Placement);
    }

    // =========================================================================
    // Target selection tests
    // =========================================================================

    #[test]
    fn nearest_enemy_is_picked_as_target() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0); // within range (30)
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        assert_eq!(state.units[0].target, Some(state.units[1].id));
    }

    #[test]
    fn allies_are_not_targeted() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 20.0, 0.0); // same owner
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        // No enemies, should go to resolution
        assert_eq!(state.phase, GamePhase::Resolution);
    }

    #[test]
    fn dead_units_are_not_targeted() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 200.0, 0.0);
        // Kill the close enemy
        state.units[1].is_alive = false;
        state.units[1].health = 0.0;
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        // Should target the far enemy (unit 2), not the dead one (unit 1)
        assert_eq!(state.units[0].target, None); // out of range, so no target — should be moving
        assert_eq!(state.units[0].animation_state, AnimState::Run);
    }

    #[test]
    fn target_set_when_in_range() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 25.0, 0.0); // within 30 range
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        assert!(state.units[0].target.is_some());
        assert_eq!(state.units[0].animation_state, AnimState::Attack);
    }

    #[test]
    fn no_target_when_out_of_range() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 500.0, 0.0); // way out of range
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        assert!(state.units[0].target.is_none());
        assert_eq!(state.units[0].animation_state, AnimState::Run);
    }

    // =========================================================================
    // Movement tests
    // =========================================================================

    #[test]
    fn moves_toward_enemy() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 500.0, 0.0);
        state.execute(Command::StartBattle, &config);
        state.tick(1.0, &config);
        // Speed is 50, dt=1.0, so should move 50 units toward enemy
        assert!((state.units[0].x - 50.0).abs() < 0.01);
        assert!((state.units[0].y - 0.0).abs() < 0.01);
    }

    #[test]
    fn speed_scales_with_dt() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 500.0, 0.0);
        state.execute(Command::StartBattle, &config);
        state.tick(0.5, &config);
        // Speed is 50, dt=0.5, so should move 25 units
        assert!((state.units[0].x - 25.0).abs() < 0.01);
    }

    #[test]
    fn no_overshoot_movement() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 10.0, 0.0); // Very close
        state.execute(Command::StartBattle, &config);
        // With speed=50 and dt=1.0, would move 50 units, but distance is only 10.
        // However, 10 < attack_range (30), so it should attack instead of move.
        state.tick(1.0, &config);
        assert_eq!(state.units[0].animation_state, AnimState::Attack);
    }

    #[test]
    fn no_overshoot_when_moving_to_close_target() {
        // Place enemy just outside range so unit must move, but move distance > remaining distance
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 35.0, 0.0); // just outside 30 range
        state.execute(Command::StartBattle, &config);
        // dt=1.0, speed=50, distance=35. Move is min(50, 35)=35. Should not overshoot.
        state.tick(1.0, &config);
        assert!((state.units[0].x - 35.0).abs() < 0.01);
    }

    #[test]
    fn no_movement_while_attacking() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0); // within range
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        // Position should not change when attacking
        assert!((state.units[0].x - 0.0).abs() < 0.01);
    }

    // =========================================================================
    // Attack tests
    // =========================================================================

    #[test]
    fn damage_applied_with_type_multiplier() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        // Both are warriors: blunt attack vs armor defense = 1.25x
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        // Warrior: 20 base dmg * 1.25 (blunt vs armor) = 25 damage
        assert!((state.units[1].health - 75.0).abs() < 0.01);
    }

    #[test]
    fn cooldown_respected_and_reset() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        state.execute(Command::StartBattle, &config);

        // First tick — attack fires (cooldown was 0)
        state.tick(0.1, &config);
        let health_after_first = state.units[1].health;
        assert!((health_after_first - 75.0).abs() < 0.01);

        // Second tick — cooldown should prevent attack (1.0s cooldown, only 0.1s elapsed)
        state.tick(0.1, &config);
        assert!((state.units[1].health - health_after_first).abs() < 0.01);

        // Fast forward past cooldown
        state.tick(0.9, &config);
        // Now cooldown should be 0, attack fires again
        assert!((state.units[1].health - 50.0).abs() < 0.01);
    }

    #[test]
    fn unit_dies_at_zero_health() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        // Lower the target's health so one hit kills
        state.units[1].health = 20.0;
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        // 20 * 1.25 = 25 damage, but health clamped at 0
        assert_eq!(state.units[1].health, 0.0);
        assert!(!state.units[1].is_alive);
    }

    // =========================================================================
    // Death handling tests
    // =========================================================================

    #[test]
    fn death_sets_animation() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        state.units[1].health = 1.0;
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        assert_eq!(state.units[1].animation_state, AnimState::Death);
    }

    #[test]
    fn health_clamped_at_zero() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        state.units[1].health = 1.0;
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        assert_eq!(state.units[1].health, 0.0);
    }

    #[test]
    fn stale_targets_cleared_on_death() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        state.units[1].health = 1.0; // Will die in one hit
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        // Unit 0's target should be cleared since unit 1 is dead
        assert!(state.units[0].target.is_none());
    }

    // =========================================================================
    // Phase resolution tests
    // =========================================================================

    #[test]
    fn resolution_when_one_side_eliminated() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        state.units[1].health = 1.0;
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        assert_eq!(state.phase, GamePhase::Resolution);
    }

    #[test]
    fn no_resolution_while_both_sides_alive() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        assert_eq!(state.phase, GamePhase::Battle);
    }

    #[test]
    fn resolution_with_no_units() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);
        assert_eq!(state.phase, GamePhase::Resolution);
    }

    // =========================================================================
    // Full scenario tests
    // =========================================================================

    #[test]
    fn two_warriors_fight_to_death() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        state.execute(Command::StartBattle, &config);

        // Both have 100 HP, deal 25 dmg/hit (blunt vs armor = 1.25x on 20 base).
        // Cooldown is 1.0s. Both attack simultaneously each second.
        // After 4 hits: 100 - 4*25 = 0 HP
        for _ in 0..100 {
            state.tick(0.1, &config);
            if state.phase == GamePhase::Resolution {
                break;
            }
        }

        assert_eq!(state.phase, GamePhase::Resolution);
        // Both should be dead (simultaneous kills)
        assert!(!state.units[0].is_alive);
        assert!(!state.units[1].is_alive);
    }

    #[test]
    fn stronger_unit_wins() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        // Give player 1 extra health advantage
        state.units[0].health = 200.0;
        state.units[0].max_health = 200.0;
        state.execute(Command::StartBattle, &config);

        for _ in 0..200 {
            state.tick(0.1, &config);
            if state.phase == GamePhase::Resolution {
                break;
            }
        }

        assert_eq!(state.phase, GamePhase::Resolution);
        assert!(state.units[0].is_alive);
        assert!(!state.units[1].is_alive);
    }

    #[test]
    fn type_advantage_decides_fight() {
        let config = make_config(TWO_UNITS_TOML);
        let mut state = GameState::new();
        // Mage (magic/mystical) vs Warrior (blunt/armor)
        // Magic vs Armor = 0.85x: mage deals 30*0.85 = 25.5 dmg
        // Blunt vs Mystical = 1.10x: warrior deals 20*1.10 = 22.0 dmg
        // Mage HP=60, Warrior HP=100
        // Mage dies in ceil(60/22) = 3 hits
        // Warrior dies in ceil(100/25.5) = 4 hits
        // Mage cooldown=1.5, Warrior cooldown=1.0
        // Warrior attacks at t=0, 1.0, 2.0 -> mage takes 22*3=66 > 60, dies at t=2.0
        // Mage attacks at t=0, 1.5 -> warrior takes 25.5*2=51, still alive
        spawn_one(&mut state, &config, "mage", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        state.execute(Command::StartBattle, &config);

        for _ in 0..300 {
            state.tick(0.1, &config);
            if state.phase == GamePhase::Resolution {
                break;
            }
        }

        assert_eq!(state.phase, GamePhase::Resolution);
        assert!(!state.units[0].is_alive); // mage dies
        assert!(state.units[1].is_alive);  // warrior survives
    }

    #[test]
    fn simultaneous_kills_both_die() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 20.0, 0.0);
        // Set both to exactly lethal health: 25 damage kills both
        state.units[0].health = 25.0;
        state.units[1].health = 25.0;
        state.execute(Command::StartBattle, &config);
        state.tick(0.1, &config);

        assert!(!state.units[0].is_alive);
        assert!(!state.units[1].is_alive);
        assert_eq!(state.phase, GamePhase::Resolution);
    }

    const RPS_TOML: &str = r#"
[[units]]
id = "warrior"
name = "Warrior"
sprite_path = ""
idle_sprite_path = ""
idle_frame_count = 1
attack_sprite_path = ""
base_health = 50.0
base_speed = 50.0
frame_count = 1
attack_frame_count = 1
frame_size = [64, 64]
attack_type = "blunt"
defense_type = "armor"
base_damage = 20.0
attack_range = 22.0
attack_cooldown = 1.0
troops_width = 1
troops_height = 1

[[units]]
id = "archer"
name = "Archer"
sprite_path = ""
idle_sprite_path = ""
idle_frame_count = 1
attack_sprite_path = ""
base_health = 30.0
base_speed = 60.0
frame_count = 1
attack_frame_count = 1
frame_size = [64, 64]
attack_type = "pierce"
defense_type = "agility"
base_damage = 12.0
attack_range = 120.0
attack_cooldown = 0.8
projectile_speed = 200.0
troops_width = 1
troops_height = 1

[[units]]
id = "lancer"
name = "Lancer"
sprite_path = ""
idle_sprite_path = ""
idle_frame_count = 1
attack_sprite_path = ""
base_health = 80.0
base_speed = 40.0
frame_count = 1
attack_frame_count = 1
frame_size = [64, 64]
attack_type = "blunt"
defense_type = "agility"
base_damage = 15.0
attack_range = 30.0
attack_cooldown = 1.2
troops_width = 1
troops_height = 1
"#;

    fn run_duel(config: &UnitsConfig, unit_a: &str, unit_b: &str) -> (bool, bool) {
        let mut state = GameState::new();
        spawn_one(&mut state, config, unit_a, PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, config, unit_b, PlayerId(2), 200.0, 0.0);
        state.execute(Command::StartBattle, config);
        for _ in 0..1000 {
            state.tick(0.1, config);
            if state.phase == GamePhase::Resolution {
                break;
            }
        }
        assert_eq!(state.phase, GamePhase::Resolution);
        (state.units[0].is_alive, state.units[1].is_alive)
    }

    // =========================================================================
    // Rock-Paper-Scissors triangle tests
    // =========================================================================

    #[test]
    fn rps_warrior_beats_archer() {
        let config = make_config(RPS_TOML);
        let (warrior_alive, archer_alive) = run_duel(&config, "warrior", "archer");
        assert!(warrior_alive, "Warrior should survive vs Archer");
        assert!(!archer_alive, "Archer should die vs Warrior");
    }

    #[test]
    fn rps_lancer_beats_warrior() {
        let config = make_config(RPS_TOML);
        let (lancer_alive, warrior_alive) = run_duel(&config, "lancer", "warrior");
        assert!(lancer_alive, "Lancer should survive vs Warrior");
        assert!(!warrior_alive, "Warrior should die vs Lancer");
    }

    #[test]
    fn rps_archer_beats_lancer() {
        let config = make_config(RPS_TOML);
        let (archer_alive, lancer_alive) = run_duel(&config, "archer", "lancer");
        assert!(archer_alive, "Archer should survive vs Lancer");
        assert!(!lancer_alive, "Lancer should die vs Archer");
    }

    #[test]
    fn units_move_then_fight() {
        let config = make_config(WARRIOR_TOML);
        let mut state = GameState::new();
        spawn_one(&mut state, &config, "warrior", PlayerId(1), 0.0, 0.0);
        spawn_one(&mut state, &config, "warrior", PlayerId(2), 200.0, 0.0);
        state.execute(Command::StartBattle, &config);

        // First tick: out of range, both should move
        state.tick(1.0, &config);
        assert_eq!(state.units[0].animation_state, AnimState::Run);
        assert_eq!(state.units[1].animation_state, AnimState::Run);

        // Keep ticking until they engage
        for _ in 0..100 {
            state.tick(0.1, &config);
            if state.units[0].animation_state == AnimState::Attack {
                break;
            }
        }

        assert_eq!(state.units[0].animation_state, AnimState::Attack);

        // Keep fighting until resolution
        for _ in 0..200 {
            state.tick(0.1, &config);
            if state.phase == GamePhase::Resolution {
                break;
            }
        }

        assert_eq!(state.phase, GamePhase::Resolution);
    }
}
