mod collision;
mod cutscene;
mod dialogue;
mod end_screen;
mod grid;
mod interaction;
mod npc;
mod player;
mod quest;
mod states;
mod tilemap;
mod transitions;

use bevy::prelude::*;

use cutscene::CutsceneState;
use dialogue::DialogueState;
use interaction::InteractionEvent;
use quest::QuestState;
use states::{AppState, InputLock};
use transitions::FadeState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "The Golden (Water) Gun".to_string(),
                resolution: bevy::window::WindowResolution::new(960, 640),
                // In a browser, fill whatever element the canvas is embedded
                // in (e.g. itch.io's HTML5 embed frame) instead of a fixed
                // native window size.
                canvas: Some("#bevy".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_message::<InteractionEvent>()
        .init_resource::<InputLock>()
        .init_resource::<QuestState>()
        .init_resource::<DialogueState>()
        .init_resource::<FadeState>()
        .init_resource::<CutsceneState>()
        .add_systems(
            Startup,
            (
                player::spawn_player_system,
                dialogue::setup_dialogue_ui,
                transitions::setup_fade_overlay,
                quest::spawn_quest_hint_ui,
            ),
        )
        .add_systems(
            OnEnter(AppState::Outdoor),
            (tilemap::spawn_outdoor_map_system, tilemap::despawn_interior_map_system),
        )
        .add_systems(OnExit(AppState::Outdoor), tilemap::despawn_outdoor_map_system)
        .add_systems(OnEnter(AppState::HouseInterior), tilemap::spawn_interior_map_system)
        .add_systems(OnEnter(AppState::Cutscene), cutscene::start_cutscene_system)
        .add_systems(OnEnter(AppState::EndScreen), end_screen::setup_end_screen_system)
        .add_systems(OnExit(AppState::EndScreen), end_screen::despawn_end_screen_system)
        .add_systems(
            Update,
            (
                grid::move_tween_system,
                transitions::fade_update_system,
                transitions::handle_scene_transition_interactions,
                dialogue::handle_interaction_for_dialogue,
                dialogue::dialogue_advance_system,
                dialogue::dialogue_render_system,
                quest::quest_hint_text_system,
                tilemap::refresh_gun_spot_system,
                player::camera_follow_system,
                player::update_facing_indicator_system,
            ),
        )
        .add_systems(
            Update,
            (player::player_input_system, interaction::interact_key_system)
                .run_if(|state: Res<State<AppState>>| {
                    matches!(state.get(), AppState::Outdoor | AppState::HouseInterior)
                }),
        )
        .add_systems(Update, cutscene::cutscene_sequencer_system.run_if(in_state(AppState::Cutscene)))
        .add_systems(Update, end_screen::restart_system.run_if(in_state(AppState::EndScreen)))
        .run();
}
