use bevy::prelude::*;

use crate::components::{Selected, TroopUnitId};
use crate::troop_spawner::UnitConfigRes;

#[derive(Component)]
pub struct DetailsPanel;

#[derive(Component)]
pub struct DetailsAvatar;

#[derive(Component)]
pub struct DetailsName;

#[derive(Component)]
pub struct DetailsStats;

pub fn setup_details_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect {
                    top: Val::Px(8.0),
                    bottom: Val::Px(6.0),
                    left: Val::Px(12.0),
                    right: Val::Px(12.0),
                },
                row_gap: Val::Px(6.0),
                width: Val::Px(220.0),
                border_radius: BorderRadius {
                    top_left: Val::Px(0.0),
                    top_right: Val::Px(10.0),
                    bottom_left: Val::Px(0.0),
                    bottom_right: Val::Px(0.0),
                },
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.09, 0.16, 0.95)),
            Visibility::Hidden,
            DetailsPanel,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(""),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.82, 0.4)),
                DetailsName,
            ));

            panel.spawn((
                Node {
                    height: Val::Px(1.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
            ));

            panel.spawn((
                ImageNode::default(),
                Node {
                    width: Val::Px(64.0),
                    height: Val::Px(64.0),
                    align_self: AlignSelf::Center,
                    ..default()
                },
                DetailsAvatar,
            ));

            panel.spawn((
                Text::new(""),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
                DetailsStats,
            ));
        });
}

pub fn update_details_ui(
    selected: Query<&TroopUnitId, With<Selected>>,
    config: Res<UnitConfigRes>,
    asset_server: Res<AssetServer>,
    mut panel_q: Query<&mut Visibility, With<DetailsPanel>>,
    mut name_q: Query<&mut Text, (With<DetailsName>, Without<DetailsStats>)>,
    mut stats_q: Query<&mut Text, (With<DetailsStats>, Without<DetailsName>)>,
    mut avatar_q: Query<&mut ImageNode, With<DetailsAvatar>>,
) {
    let Ok(mut vis) = panel_q.single_mut() else {
        return;
    };

    let Ok(troop_id) = selected.single() else {
        *vis = Visibility::Hidden;
        return;
    };

    let Some(unit) = config.0.find_by_id(&troop_id.0) else {
        *vis = Visibility::Hidden;
        return;
    };

    *vis = Visibility::Inherited;

    if let Ok(mut name_text) = name_q.single_mut() {
        **name_text = unit.name.clone();
    }

    if let Ok(mut avatar) = avatar_q.single_mut() {
        *avatar = ImageNode::new(asset_server.load(&unit.avatar_path));
    }

    if let Ok(mut stats_text) = stats_q.single_mut() {
        **stats_text = format!(
            "HP: {}\nDMG: {}\nSpeed: {}\nRange: {}\nCooldown: {:.1}s\nAttack: {:?}\nDefense: {:?}\nFormation: {}x{}\nMeat Cost: {}",
            unit.base_health,
            unit.base_damage,
            unit.base_speed,
            unit.attack_range,
            unit.attack_cooldown,
            unit.attack_type,
            unit.defense_type,
            unit.troops_width,
            unit.troops_height,
            unit.meat_cost,
        );
    }
}
