//! Loads the game's maps from the LDtk project at `ldtk/neighborhood.ldtk`.
//!
//! The project is embedded at compile time (so the web build needs no file
//! access) and parsed once at startup into a [`GameMaps`] resource. Only the
//! parts of the LDtk format the game uses are deserialized: each level's
//! `Terrain` IntGrid layer and `Entities` layer.
//!
//! The level named `Outdoor` is the neighborhood; every other level is an
//! indoor level, entered through a `Door` whose `destination` field links to
//! a `SpawnPoint` in that level.
//!
//! LDtk grids are row-major top-down, while the game grid is Y-up, so rows
//! are flipped on load (`game_y = height - 1 - ldtk_row`).

use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;
use serde_json::Value;

use crate::grid::{Direction, GridPos, TILE_SIZE};
use crate::npc::NpcId;
use crate::tilemap::{TileGrid, TileKind};

const PROJECT_JSON: &str = include_str!("../ldtk/neighborhood.ldtk");

const TERRAIN_LAYER: &str = "Terrain";
const ENTITIES_LAYER: &str = "Entities";
const OUTDOOR_LEVEL: &str = "Outdoor";

/// Index into [`GameMaps::levels`].
pub type LevelId = usize;

/// Where a door leads: a level plus the tile and facing the player arrives
/// with (the linked `SpawnPoint`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorLink {
    pub level: LevelId,
    pub pos: GridPos,
    pub facing: Direction,
}

/// Everything placed on one level, converted to game types.
pub struct LevelMap {
    pub identifier: String,
    pub grid: TileGrid,
    pub doors: Vec<(GridPos, DoorLink)>,
    pub signs: Vec<(GridPos, &'static str)>,
    pub npcs: Vec<(GridPos, NpcId, Direction)>,
}

#[derive(Resource)]
pub struct GameMaps {
    pub levels: Vec<LevelMap>,
    pub outdoor: LevelId,
    /// Where the player starts (and restarts) in the outdoor level.
    pub player_start: (GridPos, Direction),
    /// Ben's bed - the gun hiding spot - and the level it's in.
    pub gun_spot: (LevelId, GridPos),
    /// Where Ben is placed at the start of the ambush cutscene. Always in
    /// the same level as the gun spot.
    pub ben_ambush_entry: GridPos,
}

/// The level whose map is currently spawned (or about to be, mid-fade).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentLevel(pub LevelId);

impl GameMaps {
    /// Parses the embedded LDtk project. Panics with a descriptive message
    /// if it is malformed or missing something the game relies on, since
    /// the game can't run without its maps.
    pub fn load() -> Self {
        Self::from_json(PROJECT_JSON).unwrap_or_else(|e| panic!("ldtk/neighborhood.ldtk: {e}"))
    }

    pub fn is_outdoor(&self, level: LevelId) -> bool {
        level == self.outdoor
    }

    fn from_json(json: &str) -> Result<Self, String> {
        let project: Project = serde_json::from_str(json).map_err(|e| e.to_string())?;

        let terrain_def = project
            .defs
            .layers
            .iter()
            .find(|l| l.identifier == TERRAIN_LAYER)
            .ok_or_else(|| format!("no `{TERRAIN_LAYER}` layer definition"))?;
        let mut tile_kinds = HashMap::new();
        for v in &terrain_def.int_grid_values {
            let kind = TileKind::from_identifier(&v.identifier)
                .ok_or_else(|| format!("unknown Terrain value `{}`", v.identifier))?;
            tile_kinds.insert(v.value, kind);
        }

        // First pass: build each level and collect spawn points by iid, so
        // door links (which may point into levels not parsed yet) can be
        // resolved afterwards.
        let mut levels = Vec::new();
        let mut spawn_points: HashMap<String, DoorLink> = HashMap::new();
        let mut unresolved_doors: Vec<(LevelId, GridPos, Option<String>)> = Vec::new();
        let mut player_start = Vec::new();
        let mut gun_spots = Vec::new();
        let mut ambush_entries = Vec::new();

        for level in &project.levels {
            let id = levels.len();
            let (grid, entities) = parse_level(level, &tile_kinds)?;
            let mut map = LevelMap {
                identifier: level.identifier.clone(),
                grid,
                doors: Vec::new(),
                signs: Vec::new(),
                npcs: Vec::new(),
            };
            for e in entities {
                let ctx = |msg: String| format!("level `{}`, {} at {:?}: {msg}", level.identifier, e.kind, e.pos);
                match e.kind.as_str() {
                    "Door" => unresolved_doors.push((id, e.pos, e.entity_ref("destination"))),
                    // Signs are loaded once for the lifetime of the app, so
                    // leaking the text keeps `Interactable` cheap to copy.
                    "Sign" => map.signs.push((e.pos, Box::leak(e.field("text").map_err(ctx)?.to_owned().into_boxed_str()))),
                    "Npc" => map.npcs.push((
                        e.pos,
                        parse_npc_id(e.field("npc_id").map_err(ctx)?).map_err(ctx)?,
                        parse_direction(e.field("facing").map_err(ctx)?).map_err(ctx)?,
                    )),
                    "SpawnPoint" => {
                        let facing = parse_direction(e.field("facing").map_err(ctx)?).map_err(ctx)?;
                        spawn_points.insert(e.iid.clone(), DoorLink { level: id, pos: e.pos, facing });
                    }
                    "PlayerStart" => {
                        let facing = parse_direction(e.field("facing").map_err(ctx)?).map_err(ctx)?;
                        player_start.push((id, e.pos, facing));
                    }
                    "GunSpot" => gun_spots.push((id, e.pos)),
                    "AmbushEntry" => ambush_entries.push((id, e.pos)),
                    other => warn!("ldtk: ignoring unknown entity `{other}` in level `{}`", level.identifier),
                }
            }
            levels.push(map);
        }

        let outdoor = levels
            .iter()
            .position(|l| l.identifier == OUTDOOR_LEVEL)
            .ok_or_else(|| format!("missing level `{OUTDOOR_LEVEL}`"))?;

        // Second pass: resolve door links. Scene changes are driven by the
        // Outdoor/HouseInterior app states, so a door must connect the
        // outdoor level with an indoor one.
        for (from, pos, target) in unresolved_doors {
            let from_name = &levels[from].identifier;
            let target = target.ok_or_else(|| format!("level `{from_name}`: Door at {pos:?} has no destination"))?;
            let link = *spawn_points.get(&target).ok_or_else(|| {
                format!("level `{from_name}`: Door at {pos:?} links to something that isn't a SpawnPoint")
            })?;
            if (from == outdoor) == (link.level == outdoor) {
                return Err(format!(
                    "level `{from_name}`: Door at {pos:?} leads to `{}`; doors must connect `{OUTDOOR_LEVEL}` with an indoor level",
                    levels[link.level].identifier
                ));
            }
            levels[from].doors.push((pos, link));
        }

        let [(start_level, start_pos, start_facing)] = player_start[..] else {
            return Err(format!("expected exactly one PlayerStart, found {}", player_start.len()));
        };
        if start_level != outdoor {
            return Err(format!("PlayerStart must be in level `{OUTDOOR_LEVEL}`"));
        }
        let [gun_spot] = gun_spots[..] else {
            return Err(format!("expected exactly one GunSpot, found {}", gun_spots.len()));
        };
        let [(ambush_level, ben_ambush_entry)] = ambush_entries[..] else {
            return Err(format!("expected exactly one AmbushEntry, found {}", ambush_entries.len()));
        };
        if ambush_level != gun_spot.0 {
            return Err("AmbushEntry must be in the same level as the GunSpot".into());
        }

        Ok(Self { levels, outdoor, player_start: (start_pos, start_facing), gun_spot, ben_ambush_entry })
    }
}

struct EntityData {
    kind: String,
    iid: String,
    pos: GridPos,
    fields: HashMap<String, Value>,
}

impl EntityData {
    fn field(&self, name: &str) -> Result<&str, String> {
        self.fields
            .get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("no `{name}` value"))
    }

    /// The target iid of an entity-reference field, if set.
    fn entity_ref(&self, name: &str) -> Option<String> {
        self.fields.get(name)?.get("entityIid")?.as_str().map(str::to_owned)
    }
}

fn parse_level(level: &Level, tile_kinds: &HashMap<i64, TileKind>) -> Result<(TileGrid, Vec<EntityData>), String> {
    let layer = |id: &str| {
        level
            .layer_instances
            .iter()
            .find(|l| l.identifier == id)
            .ok_or_else(|| format!("level `{}` has no `{id}` layer", level.identifier))
    };

    let terrain = layer(TERRAIN_LAYER)?;
    let (w, h) = (terrain.c_wid, terrain.c_hei);
    if terrain.int_grid_csv.len() != (w * h) as usize {
        return Err(format!("level `{}` Terrain has wrong cell count", level.identifier));
    }
    let mut grid = TileGrid::filled(w, h, TileKind::Grass);
    for (i, value) in terrain.int_grid_csv.iter().enumerate() {
        let (col, row) = (i as i32 % w, i as i32 / w);
        let kind = *tile_kinds.get(value).ok_or_else(|| {
            format!("level `{}` has an unpainted or unknown Terrain cell at ({col}, {row})", level.identifier)
        })?;
        grid.set(col, h - 1 - row, kind);
    }

    let entity_layer = layer(ENTITIES_LAYER)?;
    let grid_size = entity_layer.grid_size as f32;
    debug_assert_eq!(grid_size, TILE_SIZE);
    let entities = entity_layer
        .entity_instances
        .iter()
        .map(|e| EntityData {
            kind: e.identifier.clone(),
            iid: e.iid.clone(),
            pos: GridPos::new(e.grid[0], h - 1 - e.grid[1]),
            fields: e.field_instances.iter().map(|f| (f.identifier.clone(), f.value.clone())).collect(),
        })
        .collect();

    Ok((grid, entities))
}

fn parse_direction(s: &str) -> Result<Direction, String> {
    Ok(match s {
        "Up" => Direction::Up,
        "Down" => Direction::Down,
        "Left" => Direction::Left,
        "Right" => Direction::Right,
        _ => return Err(format!("unknown Direction `{s}`")),
    })
}

fn parse_npc_id(s: &str) -> Result<NpcId, String> {
    Ok(match s {
        "Mom" => NpcId::Mom,
        "Dad" => NpcId::Dad,
        "OlderBrother" => NpcId::OlderBrother,
        "YoungerBrother" => NpcId::YoungerBrother,
        "Ben" => NpcId::Ben,
        _ => return Err(format!("unknown NpcId `{s}`")),
    })
}

// ---------------------------------------------------------------------
// The subset of the LDtk JSON schema the game reads. Unknown fields are
// ignored by serde, so this keeps working as the editor adds metadata.
// ---------------------------------------------------------------------

#[derive(Deserialize)]
struct Project {
    defs: Defs,
    levels: Vec<Level>,
}

#[derive(Deserialize)]
struct Defs {
    layers: Vec<LayerDef>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayerDef {
    identifier: String,
    #[serde(default)]
    int_grid_values: Vec<IntGridValueDef>,
}

#[derive(Deserialize)]
struct IntGridValueDef {
    value: i64,
    identifier: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Level {
    identifier: String,
    layer_instances: Vec<LayerInstance>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayerInstance {
    #[serde(rename = "__identifier")]
    identifier: String,
    #[serde(rename = "__cWid")]
    c_wid: i32,
    #[serde(rename = "__cHei")]
    c_hei: i32,
    #[serde(rename = "__gridSize")]
    grid_size: i32,
    #[serde(default)]
    int_grid_csv: Vec<i64>,
    #[serde(default)]
    entity_instances: Vec<EntityInstance>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntityInstance {
    #[serde(rename = "__identifier")]
    identifier: String,
    iid: String,
    #[serde(rename = "__grid")]
    grid: [i32; 2],
    field_instances: Vec<FieldInstance>,
}

#[derive(Deserialize)]
struct FieldInstance {
    #[serde(rename = "__identifier")]
    identifier: String,
    #[serde(rename = "__value")]
    value: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_door_tile(kind: TileKind) -> bool {
        matches!(kind, TileKind::DoorBen | TileKind::ExitDoor)
    }

    #[test]
    fn embedded_project_loads() {
        GameMaps::from_json(PROJECT_JSON).unwrap();
    }

    /// Guards against a flipped/mirrored load and against map edits that
    /// put interactables or spawns somewhere unreachable.
    #[test]
    fn entities_sit_on_sensible_tiles() {
        let maps = GameMaps::from_json(PROJECT_JSON).unwrap();
        let walkable = |level: LevelId, p: GridPos| !maps.levels[level].grid.is_blocked(p.x, p.y);

        assert!(walkable(maps.outdoor, maps.player_start.0));
        let (gun_level, gun_pos) = maps.gun_spot;
        assert_eq!(maps.levels[gun_level].grid.get(gun_pos.x, gun_pos.y), TileKind::FurnitureBedGun);
        assert!(walkable(gun_level, maps.ben_ambush_entry));

        for (id, level) in maps.levels.iter().enumerate() {
            for (p, link) in &level.doors {
                assert!(is_door_tile(level.grid.get(p.x, p.y)), "{}: door at {p:?} not on a door tile", level.identifier);
                assert!(walkable(link.level, link.pos), "{}: door at {p:?} leads onto a blocked tile", level.identifier);
            }
            for (p, _) in &level.signs {
                assert_eq!(level.grid.get(p.x, p.y), TileKind::SignPost);
            }
            for (p, _, _) in &level.npcs {
                assert!(walkable(id, *p));
            }
        }
    }

    /// Every indoor level must be enterable from outside and have a way back out.
    #[test]
    fn indoor_levels_are_connected() {
        let maps = GameMaps::from_json(PROJECT_JSON).unwrap();
        for (id, level) in maps.levels.iter().enumerate().filter(|(id, _)| !maps.is_outdoor(*id)) {
            assert!(maps.levels[maps.outdoor].doors.iter().any(|(_, l)| l.level == id), "no door into `{}`", level.identifier);
            assert!(!level.doors.is_empty(), "no door out of `{}`", level.identifier);
        }
    }
}
