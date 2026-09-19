use bevy::prelude::*;

/// Size in pixels of one logical tile. Placeholder art uses flat-colored
/// squares at this size; swapping in a real Kenney tileset later only
/// requires changing this constant to match that tileset's native size.
pub const TILE_SIZE: f32 = 32.0;

/// How long a single grid-step move animation takes.
pub const MOVE_DURATION_SECS: f32 = 0.12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn offset(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, 1),
            Direction::Down => (0, -1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

/// Authoritative logical tile position of an entity (player, NPC, ...).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn stepped(self, dir: Direction) -> Self {
        let (dx, dy) = dir.offset();
        Self { x: self.x + dx, y: self.y + dy }
    }

    pub fn to_world(self) -> Vec2 {
        Vec2::new(self.x as f32 * TILE_SIZE, self.y as f32 * TILE_SIZE)
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Facing {
    pub dir: Direction,
}

/// Present on an entity while it is animating between two grid cells.
/// Removed automatically once the tween completes.
#[derive(Component)]
pub struct MoveTween {
    pub start: Vec2,
    pub end: Vec2,
    pub timer: Timer,
}

impl MoveTween {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self {
            start,
            end,
            timer: Timer::from_seconds(MOVE_DURATION_SECS, TimerMode::Once),
        }
    }
}

/// Advances any in-flight [`MoveTween`], lerping the entity's on-screen
/// position toward its logical [`GridPos`]. Runs unconditionally so it can
/// also drive the cutscene's Ben-walks-in movement.
pub fn move_tween_system(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Transform, &mut MoveTween)>,
) {
    for (entity, mut transform, mut tween) in &mut q {
        tween.timer.tick(time.delta());
        let t = tween.timer.fraction();
        let pos = tween.start.lerp(tween.end, t);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
        transform.translation.z = y_sort_z(pos.y, transform.translation.z);
        if tween.timer.is_finished() {
            commands.entity(entity).remove::<MoveTween>();
        }
    }
}

/// Base Z layer for a translation, offset by a small Y-sort term so entities
/// lower on screen (visually "closer") draw above ones further back.
pub fn y_sort_z(world_y: f32, base_z: f32) -> f32 {
    let layer = base_z.trunc();
    layer + (1000.0 - world_y) * 0.0001
}
