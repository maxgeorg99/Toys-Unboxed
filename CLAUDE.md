# Toys Unboxed - Claude Code Guide

## Project Overview
Tactical autobattler for the Bad Ideas Game Jam 2026. Built with Bevy 0.18 + bevy_ecs_tiled.

## Architecture
- **Workspace**: 2 crates under `/crates/`
  - `simulation-core` — Pure Rust game logic (no Bevy dependency): types, unit_data, formation, game_state
  - `game` — Bevy 0.18 application: rendering, UI, input, animation
- **Assets**: `/assets/` (units.toml, TMX maps, sprite sheets, avatars)
- Asset path override in game crate: `file_path: "../../assets"`

## Bevy 0.18 API Notes (IMPORTANT)
- **Events are now called "Messages"**:
  - `#[derive(Event)]` → `#[derive(Message)]` for buffered frame-to-frame events
  - `EventReader<T>` → `MessageReader<T>`
  - `EventWriter<T>` → `MessageWriter<T>`
  - `app.add_event::<T>()` → `app.add_message::<T>()`
  - `Events<T>` → `Messages<T>`
- **`Event` trait** is now exclusively for observer/trigger pattern (immediate dispatch via `world.trigger()` / `Commands::trigger()`)
- **`EntityEvent`** is for entity-targeted observer events
- Query single: `query.single()` returns `Result`, use `.ok()` or `let Ok(..) = .. else { return };`

## Key Patterns
- Unit definitions loaded from `assets/units.toml` via `simulation_core::unit_data::UnitsConfig`
- `UnitConfigRes(UnitsConfig)` is a Bevy `Resource` wrapper (inserted at startup)
- Troop spawning uses `SpawnTroopEvent` message → `handle_spawn_troop_events` system
- Formations use `simulation_core::formation::Formation` for grid positioning
- Sprites use texture atlases with `TextureAtlasLayout::from_grid`
- Animation system: `AnimationState` component + `UnitAnimations` + `AnimationTimer`
- Drag system: `Draggable` marker + `Dragging { offset }` component
- Rotation: middle-click rotates parent, child sprites counter-rotate

## Build & Run
```sh
cargo run -p game
```

## Unit Types
- **Player (recruitable=true)**: warrior, archer, lancer, monk
- **Player (other)**: dragon, mage, knight, dwarf
- **Enemies**: skull, paddle_fish, harpoon_fish, goblin_lancer, shaman, thief, snake, turtle, minotaur, gnoll, spider, panda, lizard, bear, gnome, ogre, werewolf, demon, stone_golem, lizardman, cerberus, skeleton_mage, gryphon, headless_horseman, pyromancer, satyr_archer, centaur, gargoyle

## Reference Project
- `/Users/maximilian.georg/RustroverProjects/tower-defense-with-friends/` — Sister project with island_ui.rs boat recruitment panel pattern
