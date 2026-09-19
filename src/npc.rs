use bevy::prelude::*;

use crate::grid::{Facing, GridPos, Direction};
use crate::interaction::Interactable;
use crate::tilemap::InteriorEntity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcId {
    Mom,
    Dad,
    OlderBrother,
    YoungerBrother,
    Ben,
}

impl NpcId {
    /// Speaker label shown in the dialogue box. Spelled out as "Ben's ..."
    /// since this is Ben's family, not the player's.
    pub fn name(self) -> &'static str {
        match self {
            NpcId::Mom => "Ben's Mom",
            NpcId::Dad => "Ben's Dad",
            NpcId::OlderBrother => "Ben's Older Brother",
            NpcId::YoungerBrother => "Ben's Younger Brother",
            NpcId::Ben => "Ben",
        }
    }

    pub fn color(self) -> Color {
        match self {
            NpcId::Mom => Color::srgb(0.85, 0.45, 0.65),
            NpcId::Dad => Color::srgb(0.35, 0.45, 0.85),
            NpcId::OlderBrother => Color::srgb(0.45, 0.75, 0.45),
            NpcId::YoungerBrother => Color::srgb(0.85, 0.75, 0.35),
            NpcId::Ben => Color::srgb(0.75, 0.55, 0.45),
        }
    }

    /// Head color, drawn as a separate small square above the body so NPCs
    /// read as people rather than plain colored tiles.
    pub fn head_color(self) -> Color {
        match self {
            // Ben has red hair.
            NpcId::Ben => Color::srgb(0.8, 0.2, 0.1),
            _ => Color::srgb(0.93, 0.78, 0.63),
        }
    }
}

/// Marks an entity as an NPC, e.g. so player movement treats its tile as
/// occupied. Which specific character it is lives on the co-located
/// [`Interactable::Npc`] component instead of being duplicated here.
#[derive(Component)]
pub struct Npc;

/// Spawns an NPC as a small outlined body + head so it reads as a person at
/// a glance, distinct from the flat, unoutlined squares used for walls and
/// furniture.
pub fn spawn_npc(commands: &mut Commands, id: NpcId, pos: GridPos, facing: Direction) -> Entity {
    let world = pos.to_world();
    let size = crate::grid::TILE_SIZE;
    commands
        .spawn((
            Sprite::from_color(Color::srgb(0.05, 0.05, 0.05), Vec2::splat(size * 0.8)),
            Transform::from_xyz(world.x, world.y, 2.0),
            pos,
            Facing { dir: facing },
            Npc,
            Interactable::Npc(id),
            InteriorEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite::from_color(id.color(), Vec2::splat(size * 0.62)),
                Transform::from_xyz(0.0, -size * 0.08, 0.05),
            ));
            parent.spawn((
                Sprite::from_color(id.head_color(), Vec2::splat(size * 0.34)),
                Transform::from_xyz(0.0, size * 0.32, 0.06),
            ));
        })
        .id()
}
