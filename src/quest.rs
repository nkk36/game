use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::npc::NpcId;
use crate::states::AppState;

pub const CLUE_ORDER: [NpcId; 4] =
    [NpcId::Mom, NpcId::Dad, NpcId::OlderBrother, NpcId::YoungerBrother];

#[derive(Resource)]
pub struct QuestState {
    pub talked_to: HashSet<NpcId>,
    pub current_step: usize,
    pub gun_unlocked: bool,
}

impl Default for QuestState {
    fn default() -> Self {
        Self { talked_to: HashSet::default(), current_step: 0, gun_unlocked: false }
    }
}

#[derive(PartialEq, Eq)]
enum Stage {
    NotDue,
    Due,
    Done,
}

fn stage_for(npc: NpcId, quest: &QuestState) -> Stage {
    if quest.talked_to.contains(&npc) {
        Stage::Done
    } else if CLUE_ORDER.get(quest.current_step) == Some(&npc) {
        Stage::Due
    } else {
        Stage::NotDue
    }
}

/// The dialogue lines to show for talking to `npc` right now, given the
/// current quest progress.
pub fn npc_lines(npc: NpcId, quest: &QuestState) -> Vec<String> {
    let stage = stage_for(npc, quest);
    match (npc, stage) {
        (NpcId::Mom, Stage::Due) => vec![
            "The \"golden gun\"? Oh, you mean that dollar-store squirt gun Ben's obsessed with.".into(),
            "Ask his dad — he was helping Ben hide it earlier.".into(),
        ],
        (NpcId::Mom, Stage::Done) => vec!["Did you find Ben yet? He's been giggling all day.".into()],
        (NpcId::Mom, Stage::NotDue) => vec!["Hi there! Ben's around here somewhere.".into()],

        (NpcId::Dad, Stage::Due) => vec![
            "Ha! Yeah, I saw him sneaking around with it.".into(),
            "I think he roped his older brother into \"guarding\" it. Go ask him.".into(),
        ],
        (NpcId::Dad, Stage::Done) => vec!["Careful in there. Ben's fast.".into()],
        (NpcId::Dad, Stage::NotDue) => vec!["Ask around, I'm sure someone knows where Ben hid it.".into()],

        (NpcId::OlderBrother, Stage::Due) => vec![
            "Ben told me not to say anything...".into(),
            "...but he definitely made our younger brother help him hide it. Good luck getting HIM to talk.".into(),
        ],
        (NpcId::OlderBrother, Stage::Done) => vec!["Don't say I told you anything.".into()],
        (NpcId::OlderBrother, Stage::NotDue) => vec!["Go away, I'm busy.".into()],

        (NpcId::YoungerBrother, Stage::Due) => vec![
            "Fine, I'll tell you. It's under his bed.".into(),
            "But if I were you, I'd watch my back — Ben always finds a way to grab it first.".into(),
        ],
        (NpcId::YoungerBrother, Stage::Done) => vec!["Good luck. You'll need it.".into()],
        (NpcId::YoungerBrother, Stage::NotDue) => vec!["What do you want?".into()],

        (NpcId::Ben, _) => vec!["Looking for something?".into()],
    }
}

/// Called when a dialogue with `npc` finishes. Advances the clue chain if
/// this NPC was the one currently "due", and unlocks the gun's hiding spot
/// once every NPC has been talked to.
pub fn advance_quest_on_talk(quest: &mut QuestState, npc: NpcId) {
    if stage_for(npc, quest) != Stage::Due {
        return;
    }
    quest.talked_to.insert(npc);
    quest.current_step += 1;
    if quest.current_step >= CLUE_ORDER.len() {
        quest.gun_unlocked = true;
    }
}

#[derive(Component)]
pub struct QuestHintText;

pub fn spawn_quest_hint_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(""),
            TextFont { font_size: 18.0.into(), ..default() },
            TextColor(Color::WHITE),
            QuestHintText,
        ));
    });
}

pub fn quest_hint_text_system(
    app_state: Res<State<AppState>>,
    quest: Res<QuestState>,
    mut text_q: Query<&mut Text, With<QuestHintText>>,
) {
    if !quest.is_changed() && !app_state.is_changed() {
        return;
    }
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };
    let hint = match app_state.get() {
        AppState::Outdoor => "Find Ben's house and go inside.",
        AppState::HouseInterior => match quest.current_step {
            0 => "Ask Ben's Mom about the golden gun.",
            1 => "Ask Ben's Dad about the golden gun.",
            2 => "Ask Ben's Older Brother about the golden gun.",
            3 => "Ask Ben's Younger Brother about the golden gun.",
            _ => "You know where it is. Check Ben's room!",
        },
        AppState::Cutscene | AppState::EndScreen => "",
    };
    text.0 = hint.to_string();
}
