use bevy::prelude::*;

use crate::grid::{GridPos, TILE_SIZE};
use crate::interaction::Interactable;
use crate::ldtk::{CurrentLevel, GameMaps, LevelId, LevelMap};

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
    FurnitureStove,
    FurnitureSink,
    FurnitureFridge,
    FurnitureIsland,
    FurnitureStool,
    FurnitureChair,
    FurniturePantryShelf,
    FurnitureFireplace,
    FurnitureBookcase,
    FurnitureBookcaseGun,
    FurnitureRecliner,
    StairsUp,
    StairsDown,
    FurnitureChinaCabinet,
    FurniturePlant,
    FurnitureLiquorShelf,
    FurnitureBarCounter,
    Rug,
    FurnitureToilet,
    SlidingDoor,
    Paver,
    Hedge,
    Bush,
    FlowerBed,
    Tree,
    Fountain,
    Grill,
    PizzaOven,
    PoolTable,
    CueRack,
    ArcadeCabinet,
    TheaterSeat,
    ProjectorScreen,
    WineRack,
    GymEquipment,
    Dresser,
    Nightstand,
    Bathtub,
    Shower,
    ClosetRack,
    WasherDryer,
    Railing,
    /// A drop to the level below (foyer, backyard). Drawn as a dimmed copy
    /// of the level's `Below` tile here, if it has one.
    OpenToBelow,
    Road,
    Driveway,
    RoofSlate,
    RoofTerracotta,
    RoofShingle,
    GarageDoor,
    Pool,
    Car,
    IronFence,
    Lamppost,
    TennisCourt,
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
                | FurnitureStove
                | FurnitureSink
                | FurnitureFridge
                | FurnitureIsland
                | FurnitureStool
                | FurnitureChair
                | FurniturePantryShelf
                | FurnitureFireplace
                | FurnitureBookcase
                | FurnitureBookcaseGun
                | FurnitureRecliner
                | StairsUp
                | StairsDown
                | FurnitureChinaCabinet
                | FurniturePlant
                | FurnitureLiquorShelf
                | FurnitureBarCounter
                | FurnitureToilet
                | SlidingDoor
                | Hedge
                | Bush
                | FlowerBed
                | Tree
                | Fountain
                | Grill
                | PizzaOven
                | PoolTable
                | CueRack
                | ArcadeCabinet
                | TheaterSeat
                | ProjectorScreen
                | WineRack
                | GymEquipment
                | Dresser
                | Nightstand
                | Bathtub
                | Shower
                | ClosetRack
                | WasherDryer
                | Railing
                | OpenToBelow
                | RoofSlate
                | RoofTerracotta
                | RoofShingle
                | GarageDoor
                | Pool
                | Car
                | IronFence
                | Lamppost
        )
    }

    /// Maps an LDtk `Terrain` IntGrid value identifier to its tile kind.
    pub fn from_identifier(s: &str) -> Option<Self> {
        use TileKind::*;
        Some(match s {
            "Grass" => Grass,
            "Path" => Path,
            "HouseWall" => HouseWall,
            "DoorClosed" => DoorClosed,
            "DoorBen" => DoorBen,
            "SignPost" => SignPost,
            "Floor" => Floor,
            "Wall" => Wall,
            "ExitDoor" => ExitDoor,
            "FurnitureCouch" => FurnitureCouch,
            "FurnitureTv" => FurnitureTv,
            "FurnitureTable" => FurnitureTable,
            "FurnitureCounter" => FurnitureCounter,
            "FurnitureBed" => FurnitureBed,
            "FurnitureBedGun" => FurnitureBedGun,
            "FurnitureStove" => FurnitureStove,
            "FurnitureSink" => FurnitureSink,
            "FurnitureFridge" => FurnitureFridge,
            "FurnitureIsland" => FurnitureIsland,
            "FurnitureStool" => FurnitureStool,
            "FurnitureChair" => FurnitureChair,
            "FurniturePantryShelf" => FurniturePantryShelf,
            "FurnitureFireplace" => FurnitureFireplace,
            "FurnitureBookcase" => FurnitureBookcase,
            "FurnitureBookcaseGun" => FurnitureBookcaseGun,
            "FurnitureRecliner" => FurnitureRecliner,
            "StairsUp" => StairsUp,
            "StairsDown" => StairsDown,
            "FurnitureChinaCabinet" => FurnitureChinaCabinet,
            "FurniturePlant" => FurniturePlant,
            "FurnitureLiquorShelf" => FurnitureLiquorShelf,
            "FurnitureBarCounter" => FurnitureBarCounter,
            "Rug" => Rug,
            "FurnitureToilet" => FurnitureToilet,
            "SlidingDoor" => SlidingDoor,
            "Paver" => Paver,
            "Hedge" => Hedge,
            "Bush" => Bush,
            "FlowerBed" => FlowerBed,
            "Tree" => Tree,
            "Fountain" => Fountain,
            "Grill" => Grill,
            "PizzaOven" => PizzaOven,
            "PoolTable" => PoolTable,
            "CueRack" => CueRack,
            "ArcadeCabinet" => ArcadeCabinet,
            "TheaterSeat" => TheaterSeat,
            "ProjectorScreen" => ProjectorScreen,
            "WineRack" => WineRack,
            "GymEquipment" => GymEquipment,
            "Dresser" => Dresser,
            "Nightstand" => Nightstand,
            "Bathtub" => Bathtub,
            "Shower" => Shower,
            "ClosetRack" => ClosetRack,
            "WasherDryer" => WasherDryer,
            "Railing" => Railing,
            "OpenToBelow" => OpenToBelow,
            "Road" => Road,
            "Driveway" => Driveway,
            "RoofSlate" => RoofSlate,
            "RoofTerracotta" => RoofTerracotta,
            "RoofShingle" => RoofShingle,
            "GarageDoor" => GarageDoor,
            "Pool" => Pool,
            "Car" => Car,
            "IronFence" => IronFence,
            "Lamppost" => Lamppost,
            "TennisCourt" => TennisCourt,
            _ => return None,
        })
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
            FurnitureStove => Color::srgb(0.23, 0.23, 0.25),
            FurnitureSink => Color::srgb(0.55, 0.71, 0.82),
            FurnitureFridge => Color::srgb(0.9, 0.9, 0.92),
            FurnitureIsland => Color::srgb(0.86, 0.84, 0.8),
            FurnitureStool => Color::srgb(0.35, 0.24, 0.16),
            FurnitureChair => Color::srgb(0.45, 0.31, 0.18),
            FurniturePantryShelf => Color::srgb(0.61, 0.49, 0.33),
            FurnitureFireplace => Color::srgb(0.63, 0.25, 0.18),
            FurnitureBookcase => Color::srgb(0.29, 0.18, 0.1),
            FurnitureBookcaseGun => Color::srgb(0.29, 0.18, 0.1),
            FurnitureRecliner => Color::srgb(0.18, 0.29, 0.22),
            StairsUp => Color::srgb(0.76, 0.69, 0.57),
            StairsDown => Color::srgb(0.41, 0.35, 0.29),
            FurnitureChinaCabinet => Color::srgb(0.43, 0.27, 0.16),
            FurniturePlant => Color::srgb(0.24, 0.55, 0.24),
            FurnitureLiquorShelf => Color::srgb(0.55, 0.24, 0.31),
            FurnitureBarCounter => Color::srgb(0.31, 0.2, 0.12),
            Rug => Color::srgb(0.55, 0.18, 0.18),
            FurnitureToilet => Color::srgb(0.94, 0.94, 0.96),
            SlidingDoor => Color::srgb(0.63, 0.82, 0.9),
            Paver => Color::srgb(0.71, 0.65, 0.57),
            Hedge => Color::srgb(0.12, 0.35, 0.14),
            Bush => Color::srgb(0.18, 0.45, 0.2),
            FlowerBed => Color::srgb(0.82, 0.35, 0.55),
            Tree => Color::srgb(0.08, 0.35, 0.12),
            Fountain => Color::srgb(0.35, 0.59, 0.78),
            Grill => Color::srgb(0.16, 0.16, 0.18),
            PizzaOven => Color::srgb(0.67, 0.35, 0.2),
            PoolTable => Color::srgb(0.13, 0.45, 0.22),
            CueRack => Color::srgb(0.4, 0.26, 0.14),
            ArcadeCabinet => Color::srgb(0.45, 0.2, 0.6),
            TheaterSeat => Color::srgb(0.5, 0.1, 0.14),
            ProjectorScreen => Color::srgb(0.95, 0.95, 0.92),
            WineRack => Color::srgb(0.4, 0.12, 0.2),
            GymEquipment => Color::srgb(0.35, 0.37, 0.4),
            Dresser => Color::srgb(0.47, 0.32, 0.2),
            Nightstand => Color::srgb(0.55, 0.4, 0.26),
            Bathtub => Color::srgb(0.92, 0.94, 0.96),
            Shower => Color::srgb(0.7, 0.85, 0.92),
            ClosetRack => Color::srgb(0.6, 0.45, 0.55),
            WasherDryer => Color::srgb(0.85, 0.87, 0.9),
            Railing => Color::srgb(0.85, 0.82, 0.75),
            OpenToBelow => Color::srgb(0.08, 0.08, 0.1),
            Road => Color::srgb(0.22, 0.22, 0.25),
            Driveway => Color::srgb(0.78, 0.77, 0.74),
            RoofSlate => Color::srgb(0.32, 0.36, 0.42),
            RoofTerracotta => Color::srgb(0.72, 0.38, 0.24),
            RoofShingle => Color::srgb(0.42, 0.33, 0.30),
            GarageDoor => Color::srgb(0.88, 0.86, 0.80),
            Pool => Color::srgb(0.30, 0.75, 0.90),
            Car => Color::srgb(0.12, 0.14, 0.20),
            IronFence => Color::srgb(0.15, 0.15, 0.16),
            Lamppost => Color::srgb(0.95, 0.88, 0.55),
            TennisCourt => Color::srgb(0.25, 0.45, 0.65),
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

    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        self.get(x, y).is_blocking()
    }
}

// ---------------------------------------------------------------------
// Spawning / despawning
// ---------------------------------------------------------------------

#[derive(Component, Clone)]
pub struct OutdoorEntity;

#[derive(Component, Clone)]
pub struct InteriorEntity;

/// The water-gun prop entity sitting at [`GameMaps::gun_spot`]. Gains an
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

fn spawn_tile_sprites(commands: &mut Commands, map: &LevelMap, marker: impl Component + Clone, z: f32) {
    let grid = &map.grid;
    for y in 0..grid.height {
        for x in 0..grid.width {
            let kind = grid.get(x, y);
            let color = match (kind, map.below.get(&(x, y))) {
                (TileKind::OpenToBelow, Some(below)) => seen_from_above(below.color()),
                _ => kind.color(),
            };
            let world = GridPos::new(x, y).to_world();
            commands.spawn((
                Sprite::from_color(color, Vec2::splat(TILE_SIZE)),
                Transform::from_xyz(world.x, world.y, z),
                marker.clone(),
            ));
        }
    }
}

/// Darkens and cools a tile's color so a lower floor seen over a railing
/// reads as further away than the floor the player stands on.
fn seen_from_above(color: Color) -> Color {
    let c = color.to_srgba();
    Color::srgb(c.red * 0.45 + 0.02, c.green * 0.45 + 0.03, c.blue * 0.45 + 0.06)
}

/// Spawns a level's tiles, collision and entities, each tagged with `marker`
/// so the matching despawn system can clear them.
fn spawn_level(commands: &mut Commands, maps: &GameMaps, level: LevelId, marker: impl Component + Clone) {
    let map = &maps.levels[level];
    commands.insert_resource(ActiveCollision::from_grid(&map.grid));
    spawn_tile_sprites(commands, map, marker.clone(), 0.0);

    for &(pos, link) in &map.doors {
        commands.spawn((pos, Interactable::Door(link), marker.clone()));
    }
    for &(pos, text) in &map.signs {
        commands.spawn((pos, Interactable::Sign(text), marker.clone()));
    }

    let (gun_level, gun_pos) = maps.gun_spot;
    if gun_level == level {
        let gun_world = gun_pos.to_world();
        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 0.85, 0.1), Vec2::splat(TILE_SIZE * 0.4)),
            Transform::from_xyz(gun_world.x, gun_world.y, 2.5),
            gun_pos,
            GunPickupProp,
            marker.clone(),
        ));
    }

    for &(pos, id, facing) in &map.npcs {
        let npc = crate::npc::spawn_npc(commands, id, pos, facing);
        commands.entity(npc).remove::<InteriorEntity>().insert(marker.clone());
    }
}

pub fn spawn_outdoor_map_system(mut commands: Commands, maps: Res<GameMaps>) {
    spawn_level(&mut commands, &maps, maps.outdoor, OutdoorEntity);
}

pub fn despawn_outdoor_map_system(mut commands: Commands, q: Query<Entity, With<OutdoorEntity>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

/// Spawns whichever indoor level [`CurrentLevel`] points at.
pub fn spawn_interior_map_system(mut commands: Commands, maps: Res<GameMaps>, current: Res<CurrentLevel>) {
    spawn_level(&mut commands, &maps, current.0, InteriorEntity);
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
