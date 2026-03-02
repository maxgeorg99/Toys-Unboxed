use crate::formation::Formation;
use crate::types::{GamePhase, PlayerId, SimUnitId};
use crate::unit_data::UnitsConfig;

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
    pub x: f32,
    pub y: f32,
    pub is_alive: bool,
    pub animation_state: AnimState,
}

/// Command pattern: all mutations go through commands.
pub enum Command {
    SpawnTroop(SpawnTroopCommand),
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
    next_unit_id: u64,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            phase: GamePhase::Placement,
            units: Vec::new(),
            next_unit_id: 0,
        }
    }

    pub fn execute(&mut self, command: Command, config: &UnitsConfig) {
        match command {
            Command::SpawnTroop(cmd) => self.spawn_troop(cmd, config),
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
                x: cmd.center_x + ox,
                y: cmd.center_y + oy,
                is_alive: true,
                animation_state: AnimState::Idle,
            });
        }
    }

    pub fn tick(&mut self, _dt: f32) {
        // Future: movement, combat, etc.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit_data::UnitsConfig;

    const SKULL_TOML: &str = r#"
[[units]]
id = "skull"
name = "Skull"
sprite_path = "Enemies/Skull/Skull_Run.png"
idle_sprite_path = "Enemies/Skull/Skull_Idle.png"
idle_frame_count = 4
attack_sprite_path = "Enemies/Skull/Skull_Attack.png"
base_health = 70.0
base_speed = 45.0
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
    fn spawn_troop_creates_correct_count() {
        let config = UnitsConfig::load_from_str(SKULL_TOML).unwrap();
        let mut state = GameState::new();

        state.execute(
            Command::SpawnTroop(SpawnTroopCommand {
                unit_id: "skull".into(),
                owner: PlayerId(1),
                center_x: 0.0,
                center_y: 0.0,
                spacing: 32.0,
            }),
            &config,
        );

        assert_eq!(state.units.len(), 30);
    }

    #[test]
    fn spawned_units_have_correct_health() {
        let config = UnitsConfig::load_from_str(SKULL_TOML).unwrap();
        let mut state = GameState::new();

        state.execute(
            Command::SpawnTroop(SpawnTroopCommand {
                unit_id: "skull".into(),
                owner: PlayerId(1),
                center_x: 0.0,
                center_y: 0.0,
                spacing: 32.0,
            }),
            &config,
        );

        for unit in &state.units {
            assert_eq!(unit.health, 70.0);
            assert_eq!(unit.def_id, "skull");
            assert!(unit.is_alive);
            assert_eq!(unit.animation_state, AnimState::Idle);
            assert_eq!(unit.owner, PlayerId(1));
        }
    }

    #[test]
    fn spawned_units_have_unique_ids() {
        let config = UnitsConfig::load_from_str(SKULL_TOML).unwrap();
        let mut state = GameState::new();

        state.execute(
            Command::SpawnTroop(SpawnTroopCommand {
                unit_id: "skull".into(),
                owner: PlayerId(1),
                center_x: 0.0,
                center_y: 0.0,
                spacing: 32.0,
            }),
            &config,
        );

        let mut ids: Vec<u64> = state.units.iter().map(|u| u.id.0).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 30);
    }

    #[test]
    fn spawned_units_centered_on_origin() {
        let config = UnitsConfig::load_from_str(SKULL_TOML).unwrap();
        let mut state = GameState::new();

        state.execute(
            Command::SpawnTroop(SpawnTroopCommand {
                unit_id: "skull".into(),
                owner: PlayerId(1),
                center_x: 100.0,
                center_y: 200.0,
                spacing: 32.0,
            }),
            &config,
        );

        let avg_x: f32 = state.units.iter().map(|u| u.x).sum::<f32>() / 30.0;
        let avg_y: f32 = state.units.iter().map(|u| u.y).sum::<f32>() / 30.0;

        assert!((avg_x - 100.0).abs() < 0.001);
        assert!((avg_y - 200.0).abs() < 0.001);
    }
}
