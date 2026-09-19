use bevy::prelude::*;

use crate::grid::{Direction, Facing, GridPos};
use crate::interaction::{DoorTarget, Interactable, InteractionEvent};
use crate::player::{teleport_player, Player};
use crate::states::{AppState, InputLock};
use crate::tilemap::{INTERIOR_SPAWN, INTERIOR_SPAWN_FACING, OUTDOOR_RETURN_FACING, OUTDOOR_RETURN_POS};

const FADE_SECONDS: f32 = 0.25;

#[derive(PartialEq, Eq, Clone, Copy)]
enum FadePhase {
    Idle,
    Out,
    In,
}

#[derive(Resource)]
pub struct FadeState {
    phase: FadePhase,
    timer: Timer,
    target_state: Option<AppState>,
    target_player: Option<(GridPos, Direction)>,
}

impl Default for FadeState {
    fn default() -> Self {
        Self {
            phase: FadePhase::Idle,
            timer: Timer::from_seconds(FADE_SECONDS, TimerMode::Once),
            target_state: None,
            target_player: None,
        }
    }
}

impl FadeState {
    pub fn start(&mut self, target_state: AppState, target_player: Option<(GridPos, Direction)>) {
        if self.phase != FadePhase::Idle {
            return;
        }
        self.phase = FadePhase::Out;
        self.timer = Timer::from_seconds(FADE_SECONDS, TimerMode::Once);
        self.target_state = Some(target_state);
        self.target_player = target_player;
    }
}

#[derive(Component)]
pub struct FadeOverlay;

pub fn setup_fade_overlay(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        FadeOverlay,
        // Render above gameplay UI (dialogue box, quest hint) as well.
        GlobalZIndex(100),
    ));
}

/// Listens for door/gun-spot interactions and kicks off the appropriate
/// scene transition.
pub fn handle_scene_transition_interactions(
    mut events: MessageReader<InteractionEvent>,
    mut fade: ResMut<FadeState>,
) {
    for event in events.read() {
        match event.0 {
            Interactable::Door(DoorTarget::EnterHouse) => {
                fade.start(AppState::HouseInterior, Some((INTERIOR_SPAWN, INTERIOR_SPAWN_FACING)));
            }
            Interactable::Door(DoorTarget::ExitHouse) => {
                fade.start(AppState::Outdoor, Some((OUTDOOR_RETURN_POS, OUTDOOR_RETURN_FACING)));
            }
            Interactable::GunHidingSpot => {
                fade.start(AppState::Cutscene, None);
            }
            Interactable::Npc(_) | Interactable::Sign(_) => {}
        }
    }
}

pub fn fade_update_system(
    time: Res<Time>,
    mut fade: ResMut<FadeState>,
    mut overlay_q: Query<&mut BackgroundColor, With<FadeOverlay>>,
    mut next_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
    mut input_lock: ResMut<InputLock>,
    mut player_q: Query<(&mut GridPos, &mut Facing, &mut Transform), With<Player>>,
) {
    let Ok(mut background) = overlay_q.single_mut() else {
        return;
    };

    match fade.phase {
        FadePhase::Idle => {}
        FadePhase::Out => {
            input_lock.0 = true;
            fade.timer.tick(time.delta());
            background.0 = Color::srgba(0.0, 0.0, 0.0, fade.timer.fraction());
            if fade.timer.is_finished() {
                if let Some(target) = fade.target_state.take() {
                    next_state.set(target);
                }
                if let Some((pos, dir)) = fade.target_player.take() {
                    teleport_player(&mut player_q, pos, dir);
                }
                fade.phase = FadePhase::In;
                fade.timer = Timer::from_seconds(FADE_SECONDS, TimerMode::Once);
            }
        }
        FadePhase::In => {
            fade.timer.tick(time.delta());
            background.0 = Color::srgba(0.0, 0.0, 0.0, 1.0 - fade.timer.fraction());
            if fade.timer.is_finished() {
                fade.phase = FadePhase::Idle;
                if matches!(current_state.get(), AppState::Outdoor | AppState::HouseInterior) {
                    input_lock.0 = false;
                }
            }
        }
    }
}
