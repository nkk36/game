use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Outdoor,
    HouseInterior,
    Cutscene,
    EndScreen,
}

/// Blocks player movement/interaction input, e.g. while a dialogue box or
/// the cutscene is playing.
#[derive(Resource, Default)]
pub struct InputLock(pub bool);
