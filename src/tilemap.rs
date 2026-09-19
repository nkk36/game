use bevy::prelude::*;

use crate::grid::{Direction, GridPos, TILE_SIZE};
use crate::interaction::{DoorTarget, Interactable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Grass,
    Path,
    HouseWall,
    DoorClosed,
    DoorBen,
    SignPost,
    Floor,
    Wall,
    ExitDoor,
    FurnitureCouch,
    FurnitureTv,
    FurnitureTable,
    FurnitureCounter,
    FurnitureBed,
    FurnitureBedGun,
}

impl TileKind {
    pub fn is_blocking(self) -> bool {
        use TileKind::*;
        matches!(
            self,
            HouseWall
                | DoorClosed
                | DoorBen
                | SignPost
                | Wall
                | ExitDoor
                | FurnitureCouch
                | FurnitureTv
                | FurnitureTable
                | FurnitureCounter
                | FurnitureBed
                | FurnitureBedGun
        )
    }

    pub fn color(self) -> Color {
        use TileKind::*;
        match self {
            Grass => Color::srgb(0.30, 0.55, 0.25),
            Path => Color::srgb(0.75, 0.72, 0.60),
            HouseWall => Color::srgb(0.55, 0.35, 0.25),
            DoorClosed => Color::srgb(0.35, 0.22, 0.15),
            DoorBen => Color::srgb(0.75, 0.6, 0.15),
            SignPost => Color::srgb(0.6, 0.5, 0.35),
            Floor => Color::srgb(0.80, 0.68, 0.50),
            Wall => Color::srgb(0.45, 0.42, 0.40),
            ExitDoor => Color::srgb(0.75, 0.6, 0.15),
            FurnitureCouch => Color::srgb(0.6, 0.25, 0.25),
            FurnitureTv => Color::srgb(0.1, 0.1, 0.1),
            FurnitureTable => Color::srgb(0.5, 0.35, 0.2),
            FurnitureCounter => Color::srgb(0.7, 0.7, 0.75),
            FurnitureBed => Color::srgb(0.4, 0.55, 0.8),
            FurnitureBedGun => Color::srgb(0.4, 0.55, 0.8),
        }
    }
}

pub struct TileGrid {
    pub width: i32,
    pub height: i32,
    tiles: Vec<TileKind>,
}

impl TileGrid {
    pub fn filled(width: i32, height: i32, fill: TileKind) -> Self {
        Self {
            width,
            height,
            tiles: vec![fill; (width * height) as usize],
        }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            None
        } else {
            Some((y * self.width + x) as usize)
        }
    }

    pub fn get(&self, x: i32, y: i32) -> TileKind {
        self.index(x, y).map(|i| self.tiles[i]).unwrap_or(TileKind::Wall)
    }

    pub fn set(&mut self, x: i32, y: i32, kind: TileKind) {
        if let Some(i) = self.index(x, y) {
            self.tiles[i] = kind;
        }
    }

    pub fn fill_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, kind: TileKind) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.set(x, y, kind);
            }
        }
    }

    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        self.get(x, y).is_blocking()
    }
}

// ---------------------------------------------------------------------
// Outdoor map
// ---------------------------------------------------------------------

pub const OUTDOOR_WIDTH: i32 = 17;
pub const OUTDOOR_HEIGHT: i32 = 7;
pub const OUTDOOR_PATH_Y: i32 = 2;
pub const OUTDOOR_DOOR_Y: i32 = 4;
pub const OUTDOOR_ROOF_Y: i32 = 5;

pub const NEIGHBOR1_X: i32 = 2;
pub const NEIGHBOR2_X: i32 = 7;
pub const BEN_HOUSE_X: i32 = 12;

pub const BEN_DOOR_POS: GridPos = GridPos { x: BEN_HOUSE_X + 1, y: OUTDOOR_DOOR_Y };
pub const OUTDOOR_SPAWN: GridPos = GridPos { x: 1, y: OUTDOOR_PATH_Y };
pub const OUTDOOR_SPAWN_FACING: Direction = Direction::Up;
/// Where the player lands after walking back out of the house.
pub const OUTDOOR_RETURN_POS: GridPos = GridPos { x: BEN_HOUSE_X + 1, y: OUTDOOR_DOOR_Y - 1 };
pub const OUTDOOR_RETURN_FACING: Direction = Direction::Down;

fn place_house(grid: &mut TileGrid, base_x: i32, door_kind: TileKind) {
    grid.fill_rect(base_x, OUTDOOR_ROOF_Y, base_x + 2, OUTDOOR_ROOF_Y, TileKind::HouseWall);
    grid.set(base_x, OUTDOOR_DOOR_Y, TileKind::HouseWall);
    grid.set(base_x + 1, OUTDOOR_DOOR_Y, door_kind);
    grid.set(base_x + 2, OUTDOOR_DOOR_Y, TileKind::HouseWall);
}

pub fn build_outdoor_grid() -> TileGrid {
    let mut grid = TileGrid::filled(OUTDOOR_WIDTH, OUTDOOR_HEIGHT, TileKind::Grass);
    grid.fill_rect(0, OUTDOOR_PATH_Y, OUTDOOR_WIDTH - 1, OUTDOOR_PATH_Y, TileKind::Path);

    place_house(&mut grid, NEIGHBOR1_X, TileKind::DoorClosed);
    place_house(&mut grid, NEIGHBOR2_X, TileKind::DoorClosed);
    place_house(&mut grid, BEN_HOUSE_X, TileKind::DoorBen);

    grid.set(NEIGHBOR1_X + 1, 1, TileKind::SignPost);
    grid.set(NEIGHBOR2_X + 1, 1, TileKind::SignPost);

    grid
}

pub const OUTDOOR_SIGNS: [(GridPos, &str); 2] = [
    (GridPos { x: NEIGHBOR1_X + 1, y: 1 }, "A weathered sign: \"The Johnsons\""),
    (GridPos { x: NEIGHBOR2_X + 1, y: 1 }, "A weathered sign: \"The Millers\""),
];

// ---------------------------------------------------------------------
// House interior map
// ---------------------------------------------------------------------

pub const INTERIOR_WIDTH: i32 = 21;
pub const INTERIOR_HEIGHT: i32 = 15;

pub const EXIT_DOOR_POS: GridPos = GridPos { x: 10, y: 0 };
pub const INTERIOR_SPAWN: GridPos = GridPos { x: 10, y: 1 };
pub const INTERIOR_SPAWN_FACING: Direction = Direction::Up;

pub const MOM_POS: GridPos = GridPos { x: 5, y: 4 };
pub const DAD_POS: GridPos = GridPos { x: 15, y: 4 };
pub const OLDER_BROTHER_POS: GridPos = GridPos { x: 3, y: 11 };
pub const YOUNGER_BROTHER_POS: GridPos = GridPos { x: 10, y: 11 };

/// Ben's bed - the gun hiding spot. Player must stand just south of it,
/// facing north, to interact with it.
pub const GUN_SPOT_POS: GridPos = GridPos { x: 17, y: 11 };
/// Where Ben is placed at the start of the ambush cutscene, and the tile
/// he walks to (the gun spot itself) during it.
pub const BEN_AMBUSH_ENTRY_POS: GridPos = GridPos { x: 19, y: 10 };

pub fn build_interior_grid() -> TileGrid {
    let mut grid = TileGrid::filled(INTERIOR_WIDTH, INTERIOR_HEIGHT, TileKind::Floor);

    // Outer walls.
    grid.fill_rect(0, 0, INTERIOR_WIDTH - 1, 0, TileKind::Wall);
    grid.fill_rect(0, INTERIOR_HEIGHT - 1, INTERIOR_WIDTH - 1, INTERIOR_HEIGHT - 1, TileKind::Wall);
    grid.fill_rect(0, 0, 0, INTERIOR_HEIGHT - 1, TileKind::Wall);
    grid.fill_rect(INTERIOR_WIDTH - 1, 0, INTERIOR_WIDTH - 1, INTERIOR_HEIGHT - 1, TileKind::Wall);
    grid.set(EXIT_DOOR_POS.x, EXIT_DOOR_POS.y, TileKind::ExitDoor);

    // Horizontal partition between bottom rooms (Living Room/Kitchen) and
    // top rooms (the two brothers' rooms and Ben's), with doorless gaps.
    grid.fill_rect(1, 7, INTERIOR_WIDTH - 2, 7, TileKind::Wall);
    for gap_x in [5, 10, 15] {
        grid.set(gap_x, 7, TileKind::Floor);
    }

    // Vertical partition separating Living Room from Kitchen. Starts at
    // y=2 (not y=1) so the entrance row stays clear - otherwise it walls
    // off the tile the player spawns on right after walking in.
    grid.fill_rect(10, 2, 10, 6, TileKind::Wall);
    grid.set(10, 3, TileKind::Floor);

    // Vertical partitions separating the three top rooms.
    grid.fill_rect(7, 8, 7, 13, TileKind::Wall);
    grid.set(7, 10, TileKind::Floor);
    grid.fill_rect(14, 8, 14, 13, TileKind::Wall);
    grid.set(14, 10, TileKind::Floor);

    // Furniture.
    grid.set(3, 3, TileKind::FurnitureCouch);
    grid.set(7, 5, TileKind::FurnitureTv);
    grid.set(17, 3, TileKind::FurnitureCounter);
    grid.set(13, 5, TileKind::FurnitureTable);
    grid.set(2, 12, TileKind::FurnitureBed);
    grid.set(12, 12, TileKind::FurnitureBed);
    grid.set(GUN_SPOT_POS.x, GUN_SPOT_POS.y, TileKind::FurnitureBedGun);

    grid
}

// ---------------------------------------------------------------------
// Spawning / despawning
// ---------------------------------------------------------------------

#[derive(Component, Clone)]
pub struct OutdoorEntity;

#[derive(Component, Clone)]
pub struct InteriorEntity;

/// The water-gun prop entity sitting at [`GUN_SPOT_POS`]. Gains an
/// [`Interactable::GunHidingSpot`] once the quest unlocks it, and is
/// despawned by the cutscene when Ben grabs it.
#[derive(Component)]
pub struct GunPickupProp;

#[derive(Resource, Default)]
pub struct ActiveCollision {
    pub grid_width: i32,
    pub grid_height: i32,
    pub blocked: std::collections::HashSet<(i32, i32)>,
}

impl ActiveCollision {
    pub fn from_grid(grid: &TileGrid) -> Self {
        let mut blocked = std::collections::HashSet::new();
        for y in 0..grid.height {
            for x in 0..grid.width {
                if grid.is_blocked(x, y) {
                    blocked.insert((x, y));
                }
            }
        }
        Self { grid_width: grid.width, grid_height: grid.height, blocked }
    }

    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.grid_width || y >= self.grid_height {
            return true;
        }
        self.blocked.contains(&(x, y))
    }
}

fn spawn_tile_sprites(commands: &mut Commands, grid: &TileGrid, marker: impl Component + Clone, z: f32) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            let kind = grid.get(x, y);
            let world = GridPos::new(x, y).to_world();
            commands.spawn((
                Sprite::from_color(kind.color(), Vec2::splat(TILE_SIZE)),
                Transform::from_xyz(world.x, world.y, z),
                marker.clone(),
            ));
        }
    }
}

pub fn spawn_outdoor_map_system(mut commands: Commands) {
    let grid = build_outdoor_grid();
    commands.insert_resource(ActiveCollision::from_grid(&grid));
    spawn_tile_sprites(&mut commands, &grid, OutdoorEntity, 0.0);

    commands.spawn((
        BEN_DOOR_POS,
        Interactable::Door(DoorTarget::EnterHouse),
        OutdoorEntity,
    ));
    for (pos, text) in OUTDOOR_SIGNS {
        commands.spawn((pos, Interactable::Sign(text), OutdoorEntity));
    }
}

pub fn despawn_outdoor_map_system(mut commands: Commands, q: Query<Entity, With<OutdoorEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

pub fn spawn_interior_map_system(mut commands: Commands) {
    let grid = build_interior_grid();
    commands.insert_resource(ActiveCollision::from_grid(&grid));
    spawn_tile_sprites(&mut commands, &grid, InteriorEntity, 0.0);

    commands.spawn((EXIT_DOOR_POS, Interactable::Door(DoorTarget::ExitHouse), InteriorEntity));

    let gun_world = GUN_SPOT_POS.to_world();
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 0.85, 0.1), Vec2::splat(TILE_SIZE * 0.4)),
        Transform::from_xyz(gun_world.x, gun_world.y, 2.5),
        GUN_SPOT_POS,
        GunPickupProp,
        InteriorEntity,
    ));

    crate::npc::spawn_npc(&mut commands, crate::npc::NpcId::Mom, MOM_POS, Direction::Down);
    crate::npc::spawn_npc(&mut commands, crate::npc::NpcId::Dad, DAD_POS, Direction::Down);
    crate::npc::spawn_npc(&mut commands, crate::npc::NpcId::OlderBrother, OLDER_BROTHER_POS, Direction::Down);
    crate::npc::spawn_npc(&mut commands, crate::npc::NpcId::YoungerBrother, YOUNGER_BROTHER_POS, Direction::Down);
}

pub fn despawn_interior_map_system(mut commands: Commands, q: Query<Entity, With<InteriorEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

/// Once every family member has been talked to, attach the interactable
/// component to the (already-spawned) gun prop so it can be found.
pub fn refresh_gun_spot_system(
    mut commands: Commands,
    quest: Res<crate::quest::QuestState>,
    q: Query<(Entity, Has<Interactable>), With<GunPickupProp>>,
) {
    if !quest.is_changed() || !quest.gun_unlocked {
        return;
    }
    for (entity, has_interactable) in &q {
        if !has_interactable {
            commands.entity(entity).insert(Interactable::GunHidingSpot);
        }
    }
}
