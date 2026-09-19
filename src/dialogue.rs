use bevy::prelude::*;

use crate::interaction::{Interactable, InteractionEvent};
use crate::npc::NpcId;
use crate::quest::{advance_quest_on_talk, npc_lines, QuestState};
use crate::states::InputLock;

#[derive(Resource, Default)]
pub struct DialogueState {
    pub active: bool,
    /// True for exactly one frame after opening, so the key press that
    /// opened the dialogue can't also immediately advance/close it.
    just_opened: bool,
    pub speaker: Option<NpcId>,
    pub lines: Vec<String>,
    pub line_index: usize,
}

impl DialogueState {
    fn open(&mut self, speaker: Option<NpcId>, lines: Vec<String>) {
        self.active = true;
        self.just_opened = true;
        self.speaker = speaker;
        self.lines = lines;
        self.line_index = 0;
    }
}

#[derive(Component)]
pub(crate) struct DialogueRoot;

#[derive(Component)]
pub(crate) struct DialogueSpeakerText;

#[derive(Component)]
pub(crate) struct DialogueBodyText;

pub fn setup_dialogue_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(24.0),
                left: Val::Px(48.0),
                right: Val::Px(48.0),
                padding: UiRect::all(Val::Px(16.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.9)),
            Visibility::Hidden,
            DialogueRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont { font_size: 20.0.into(), ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.4)),
                DialogueSpeakerText,
            ));
            parent.spawn((
                Text::new(""),
                TextFont { font_size: 18.0.into(), ..default() },
                TextColor(Color::WHITE),
                DialogueBodyText,
            ));
        });
}

pub fn handle_interaction_for_dialogue(
    mut events: MessageReader<InteractionEvent>,
    quest: Res<QuestState>,
    mut dialogue: ResMut<DialogueState>,
    mut input_lock: ResMut<InputLock>,
) {
    for event in events.read() {
        match event.0 {
            Interactable::Npc(id) => {
                dialogue.open(Some(id), npc_lines(id, &quest));
                input_lock.0 = true;
            }
            Interactable::Sign(text) => {
                dialogue.open(None, vec![text.to_string()]);
                input_lock.0 = true;
            }
            Interactable::Door(_) | Interactable::GunHidingSpot => {
                // Handled by transitions.rs
            }
        }
    }
}

pub fn dialogue_advance_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut dialogue: ResMut<DialogueState>,
    mut quest: ResMut<QuestState>,
    mut input_lock: ResMut<InputLock>,
) {
    if !dialogue.active {
        return;
    }
    if dialogue.just_opened {
        dialogue.just_opened = false;
        return;
    }
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    dialogue.line_index += 1;
    if dialogue.line_index >= dialogue.lines.len() {
        if let Some(speaker) = dialogue.speaker {
            advance_quest_on_talk(&mut quest, speaker);
        }
        dialogue.active = false;
        dialogue.lines.clear();
        dialogue.line_index = 0;
        dialogue.speaker = None;
        input_lock.0 = false;
    }
}

pub fn dialogue_render_system(
    dialogue: Res<DialogueState>,
    mut root_q: Query<&mut Visibility, With<DialogueRoot>>,
    mut speaker_q: Query<&mut Text, (With<DialogueSpeakerText>, Without<DialogueBodyText>)>,
    mut body_q: Query<&mut Text, (With<DialogueBodyText>, Without<DialogueSpeakerText>)>,
) {
    if !dialogue.is_changed() {
        return;
    }
    let Ok(mut visibility) = root_q.single_mut() else {
        return;
    };
    *visibility = if dialogue.active { Visibility::Visible } else { Visibility::Hidden };
    if !dialogue.active {
        return;
    }

    if let Ok(mut speaker_text) = speaker_q.single_mut() {
        speaker_text.0 = dialogue.speaker.map(|s| s.name().to_string()).unwrap_or_default();
    }
    if let Ok(mut body_text) = body_q.single_mut() {
        body_text.0 = dialogue.lines.get(dialogue.line_index).cloned().unwrap_or_default();
    }
}
