use bevy::prelude::*;

use crate::components::{Selected, TroopUnitId};
use crate::recruitment_ui::{attack_icon_path, defense_icon_path};
use crate::troop_spawner::UnitConfigRes;

#[derive(Component)]
pub struct DetailsPanel;

#[derive(Component)]
pub struct DetailsAvatar;

#[derive(Component)]
pub struct DetailsName;

#[derive(Component)]
pub struct DetailsStats;

#[derive(Component)]
pub struct DetailsAttackIcon;

#[derive(Component)]
pub struct DetailsDefenseIcon;

const DETAIL_ICON_SZ: f32 = 16.0;
const LABEL_COLOR: Color = Color::srgba(0.6, 0.6, 0.6, 1.0);

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

            for (label, marker_is_atk) in [("Attack:", true), ("Defense:", false)] {
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new(label),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(LABEL_COLOR),
                        ));
                        let icon_bundle = (
                            ImageNode::default(),
                            Node {
                                width: Val::Px(DETAIL_ICON_SZ),
                                height: Val::Px(DETAIL_ICON_SZ),
                                ..default()
                            },
                        );
                        if marker_is_atk {
                            row.spawn((icon_bundle.0, icon_bundle.1, DetailsAttackIcon));
                        } else {
                            row.spawn((icon_bundle.0, icon_bundle.1, DetailsDefenseIcon));
                        }
                    });
            }
        });
}

pub fn update_details_ui(
    selected: Query<&TroopUnitId, With<Selected>>,
    config: Res<UnitConfigRes>,
    asset_server: Res<AssetServer>,
    mut panel_q: Query<&mut Visibility, With<DetailsPanel>>,
    mut name_q: Query<&mut Text, (With<DetailsName>, Without<DetailsStats>)>,
    mut stats_q: Query<&mut Text, (With<DetailsStats>, Without<DetailsName>)>,
    mut avatar_q: Query<&mut ImageNode, (With<DetailsAvatar>, Without<DetailsAttackIcon>, Without<DetailsDefenseIcon>)>,
    mut atk_icon_q: Query<&mut ImageNode, (With<DetailsAttackIcon>, Without<DetailsAvatar>, Without<DetailsDefenseIcon>)>,
    mut def_icon_q: Query<&mut ImageNode, (With<DetailsDefenseIcon>, Without<DetailsAvatar>, Without<DetailsAttackIcon>)>,
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

    if let Ok(mut icon) = atk_icon_q.single_mut() {
        *icon = ImageNode::new(asset_server.load(attack_icon_path(unit.attack_type)));
    }
    if let Ok(mut icon) = def_icon_q.single_mut() {
        *icon = ImageNode::new(asset_server.load(defense_icon_path(unit.defense_type)));
    }

    if let Ok(mut stats_text) = stats_q.single_mut() {
        **stats_text = format!(
            "HP: {}\nDMG: {}\nSpeed: {}\nRange: {}\nCooldown: {:.1}s\nFormation: {}x{}\nCost: {}",
            unit.base_health,
            unit.base_damage,
            unit.base_speed,
            unit.attack_range,
            unit.attack_cooldown,
            unit.troops_width,
            unit.troops_height,
            unit.meat_cost,
        );
    }
}
