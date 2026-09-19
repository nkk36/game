use bevy::prelude::*;

use crate::grid::GridPos;
use crate::npc::Npc;
use crate::tilemap::ActiveCollision;

/// True if `target` is blocked by static map geometry or by an NPC
/// currently standing there.
pub fn is_move_blocked(
    target: GridPos,
    collision: &ActiveCollision,
    npc_positions: &Query<&GridPos, With<Npc>>,
) -> bool {
    if collision.is_blocked(target.x, target.y) {
        return true;
    }
    npc_positions.iter().any(|pos| *pos == target)
}
