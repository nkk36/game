use bevy::camera::ScalingMode;
use bevy::image::{TextureAtlas, TextureAtlasLayout};
use bevy::prelude::*;

use crate::collision::is_move_blocked;
use crate::grid::{Direction, Facing, GridPos, MoveTween, TILE_SIZE};
use crate::ldtk::{CurrentLevel, GameMaps};
use crate::npc::Npc;
use crate::states::InputLock;
use crate::tilemap::ActiveCollision;

#[derive(Component)]
pub struct Player;

/// A small marker glued to the front of the player showing which way
/// they're currently facing, so it's clear what "space" will interact with
/// before pressing it.
#[derive(Component)]
pub struct FacingIndicator;

/// Idle.png is a single row of 6 128x128 frames (5 near-identical poses
/// plus one blink), looped continuously regardless of movement state.
const IDLE_FRAME_SIZE: UVec2 = UVec2::new(128, 128);
const IDLE_FRAME_COUNT: u32 = 6;
/// The character only occupies this part of each frame (measured from
/// Idle.png's alpha; the rest is transparent padding), so the atlas is
/// cropped to it - otherwise the player renders at a quarter of a tile.
const IDLE_CHARACTER_RECT: URect = URect { min: UVec2::new(40, 84), max: UVec2::new(75, 128) };
const IDLE_FRAME_SECONDS: f32 = 0.15;

#[derive(Component)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component)]
pub struct AnimationTimer(Timer);

pub fn spawn_player_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    maps: Res<GameMaps>,
) {
    let (spawn_pos, spawn_facing) = maps.player_start;
    let world = spawn_pos.to_world();
    let texture = asset_server.load("Idle.png");
    let mut layout = TextureAtlasLayout::new_empty(UVec2::new(IDLE_FRAME_SIZE.x * IDLE_FRAME_COUNT, IDLE_FRAME_SIZE.y));
    for i in 0..IDLE_FRAME_COUNT {
        let offset = UVec2::new(i * IDLE_FRAME_SIZE.x, 0);
        layout.add_texture(URect { min: IDLE_CHARACTER_RECT.min + offset, max: IDLE_CHARACTER_RECT.max + offset });
    }
    let layout = atlas_layouts.add(layout);
    // One tile tall, keeping the character's aspect ratio.
    let character_size = IDLE_CHARACTER_RECT.size().as_vec2();
    commands
        .spawn((
            Sprite {
                custom_size: Some(character_size * (TILE_SIZE / character_size.y)),
                ..Sprite::from_atlas_image(texture, TextureAtlas { layout, index: 0 })
            },
            Transform::from_xyz(world.x, world.y, 3.0),
            spawn_pos,
            Facing { dir: spawn_facing },
            Player,
            AnimationIndices { first: 0, last: (IDLE_FRAME_COUNT - 1) as usize },
            AnimationTimer(Timer::from_seconds(IDLE_FRAME_SECONDS, TimerMode::Repeating)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite::from_color(Color::srgb(1.0, 0.95, 0.3), Vec2::splat(TILE_SIZE * 0.22)),
                Transform::from_xyz(0.0, TILE_SIZE * 0.38, 0.1),
                FacingIndicator,
            ));
        });

    // Framed to the current level by `camera_fit_level_system`.
    commands.spawn(Camera2d);
}

/// Keeps the facing indicator glued to whichever side of the player they're
/// currently facing.
pub fn update_facing_indicator_system(
    player_q: Query<(&Facing, &Children), (With<Player>, Changed<Facing>)>,
    mut indicator_q: Query<&mut Transform, With<FacingIndicator>>,
) {
    let Ok((facing, children)) = player_q.single() else {
        return;
    };
    let (dx, dy) = facing.dir.offset();
    for child in children.iter() {
        if let Ok(mut transform) = indicator_q.get_mut(child) {
            transform.translation.x = dx as f32 * TILE_SIZE * 0.38;
            transform.translation.y = dy as f32 * TILE_SIZE * 0.38;
        }
    }
}

/// Loops the player's sprite through its idle animation frames.
pub fn player_animation_system(
    time: Res<Time>,
    mut player_q: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite), With<Player>>,
) {
    let Ok((indices, mut timer, mut sprite)) = player_q.single_mut() else {
        return;
    };
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = if atlas.index >= indices.last { indices.first } else { atlas.index + 1 };
        }
    }
}

pub fn player_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    input_lock: Res<InputLock>,
    collision: Res<ActiveCollision>,
    npc_positions: Query<&GridPos, With<Npc>>,
    mut commands: Commands,
    mut player_q: Query<(Entity, &mut GridPos, &mut Facing), (With<Player>, Without<MoveTween>, Without<Npc>)>,
) {
    if input_lock.0 {
        return;
    }
    let Ok((entity, mut pos, mut facing)) = player_q.single_mut() else {
        return;
    };

    let dir = if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        Some(Direction::Up)
    } else if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        Some(Direction::Down)
    } else if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        Some(Direction::Left)
    } else if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        Some(Direction::Right)
    } else {
        None
    };

    let Some(dir) = dir else {
        return;
    };
    facing.dir = dir;

    let target = pos.stepped(dir);
    if is_move_blocked(target, &collision, &npc_positions) {
        return;
    }

    let start = pos.to_world();
    *pos = target;
    commands.entity(entity).insert(MoveTween::new(start, target.to_world()));
}

/// Directly repositions the player, bypassing tweening. Used by scene
/// transitions when moving the player between the outdoor map and the
/// house interior.
pub fn teleport_player(
    player_q: &mut Query<(&mut GridPos, &mut Facing, &mut Transform), With<Player>>,
    pos: GridPos,
    facing: Direction,
) {
    if let Ok((mut grid_pos, mut f, mut transform)) = player_q.single_mut() {
        *grid_pos = pos;
        f.dir = facing;
        let world = pos.to_world();
        transform.translation.x = world.x;
        transform.translation.y = world.y;
    }
}

/// Centers the camera on the current level and zooms so the whole level
/// (plus a half-tile margin) is visible. `AutoMin` keeps it fitted as the
/// window resizes, so this only needs to run when the level changes.
pub fn camera_fit_level_system(
    maps: Res<GameMaps>,
    current_level: Res<CurrentLevel>,
    mut camera_q: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    if !current_level.is_changed() {
        return;
    }
    let Ok((mut cam_t, mut projection)) = camera_q.single_mut() else {
        return;
    };
    let grid = &maps.levels[current_level.0].grid;
    let (w, h) = (grid.width as f32, grid.height as f32);
    // Tiles are centered on their grid position, so the level spans
    // -TILE_SIZE/2 ..= (w - 0.5) * TILE_SIZE horizontally (same vertically).
    cam_t.translation.x = (w - 1.0) * TILE_SIZE / 2.0;
    cam_t.translation.y = (h - 1.0) * TILE_SIZE / 2.0;
    *projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::AutoMin { min_width: (w + 1.0) * TILE_SIZE, min_height: (h + 1.0) * TILE_SIZE },
        ..OrthographicProjection::default_2d()
    });
}
