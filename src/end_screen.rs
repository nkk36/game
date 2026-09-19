use bevy::prelude::*;

use crate::cutscene::CutsceneState;
use crate::dialogue::DialogueState;
use crate::quest::QuestState;
use crate::states::AppState;
use crate::tilemap::{OUTDOOR_SPAWN, OUTDOOR_SPAWN_FACING};
use crate::transitions::FadeState;

#[derive(Component)]
pub(crate) struct EndScreenEntity;

pub fn setup_end_screen_system(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            EndScreenEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Ben shot you with the golden gun... again!"),
                TextFont {
                    font_size: 34.0.into(),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.3)),
            ));
            parent.spawn((
                Text::new("He always finds a way to grab it first."),
                TextFont {
                    font_size: 18.0.into(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Press R to try again"),
                TextFont {
                    font_size: 16.0.into(),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

pub fn despawn_end_screen_system(mut commands: Commands, q: Query<Entity, With<EndScreenEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

pub fn restart_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut quest: ResMut<QuestState>,
    mut dialogue: ResMut<DialogueState>,
    mut cutscene: ResMut<CutsceneState>,
    mut fade: ResMut<FadeState>,
) {
    if !keys.just_pressed(KeyCode::KeyR) {
        return;
    }
    *quest = QuestState::default();
    *dialogue = DialogueState::default();
    *cutscene = CutsceneState::default();
    fade.start(
        AppState::Outdoor,
        Some((OUTDOOR_SPAWN, OUTDOOR_SPAWN_FACING)),
    );
}
