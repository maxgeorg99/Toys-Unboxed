use bevy::prelude::*;

#[derive(Component)]
pub struct MenuUI;

#[derive(Component)]
pub struct PlayButton;

pub fn setup_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
            MenuUI,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("TOYS UNBOXED"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.4)),
            ));

            root.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(48.0), Val::Px(16.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.7, 0.3)),
                PlayButton,
            ))
            .with_child((
                Text::new("PLAY"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn handle_play_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    mut next_state: ResMut<NextState<super::AppState>>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            next_state.set(super::AppState::Lobby);
        }
    }
}

pub fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
