use bevy::prelude::*;

use crate::grid::{Direction, GridPos};
use crate::npc::NpcId;
use crate::player::Player;
use crate::states::InputLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorTarget {
    EnterHouse,
    ExitHouse,
}

#[derive(Component, Debug, Clone, Copy)]
pub enum Interactable {
    Npc(NpcId),
    Door(DoorTarget),
    GunHidingSpot,
    Sign(&'static str),
}

/// Fired when the player successfully interacts with something in front of
/// them. Consumed by whichever system (dialogue, transitions, ...) cares
/// about that particular [`Interactable`] variant.
#[derive(Message, Clone, Copy)]
pub struct InteractionEvent(pub Interactable);

pub fn interact_key_system(
    keys: Res<ButtonInput<KeyCode>>,
    input_lock: Res<InputLock>,
    player_q: Query<(&GridPos, &crate::grid::Facing), With<Player>>,
    interactables: Query<(&GridPos, &Interactable)>,
    mut events: MessageWriter<InteractionEvent>,
) {
    if input_lock.0 || !keys.just_pressed(KeyCode::Space) {
        return;
    }
    let Ok((player_pos, facing)) = player_q.single() else {
        return;
    };

    // Prefer whatever the player is actually facing, but fall back to
    // checking every adjacent tile - standing right next to something
    // should be enough to interact with it even if the facing direction
    // isn't pixel-perfect.
    let facing_first = [facing.dir, Direction::Up, Direction::Down, Direction::Left, Direction::Right];
    for dir in facing_first {
        let target = player_pos.stepped(dir);
        if let Some((_, interactable)) = interactables.iter().find(|(pos, _)| **pos == target) {
            events.write(InteractionEvent(*interactable));
            return;
        }
    }
}
