#!/usr/bin/env python3
"""Transcribe the hardcoded maps in src/tilemap.rs into an LDtk project.

Produces ldtk/neighborhood.ldtk with two levels (Outdoor, HouseInterior),
each with a `Terrain` IntGrid layer (one value per `TileKind`) and an
`Entities` layer (NPCs, doors, signs, spawn points, gun spot, ambush entry).

The game grid is Y-up (y=0 is the bottom row); LDtk is row-major top-down.
Rows are flipped on export so the level looks the same in the editor as it
does in-game (north at the top).

Re-run this whenever the Rust maps change:  python3 tools/generate_ldtk.py
"""

import json
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT_PATH = ROOT / "ldtk" / "neighborhood.ldtk"

LDTK_VERSION = "1.5.3"
TILE_SIZE = 32  # grid.rs: TILE_SIZE

# ---------------------------------------------------------------------
# TileKind (tilemap.rs) -- order defines the IntGrid value (index + 1).
# Colors copied from TileKind::color().
# ---------------------------------------------------------------------

TILE_KINDS = [
    ("Grass", (0.30, 0.55, 0.25)),
    ("Path", (0.75, 0.72, 0.60)),
    ("HouseWall", (0.55, 0.35, 0.25)),
    ("DoorClosed", (0.35, 0.22, 0.15)),
    ("DoorBen", (0.75, 0.6, 0.15)),
    ("SignPost", (0.6, 0.5, 0.35)),
    ("Floor", (0.80, 0.68, 0.50)),
    ("Wall", (0.45, 0.42, 0.40)),
    ("ExitDoor", (0.75, 0.6, 0.15)),
    ("FurnitureCouch", (0.6, 0.25, 0.25)),
    ("FurnitureTv", (0.1, 0.1, 0.1)),
    ("FurnitureTable", (0.5, 0.35, 0.2)),
    ("FurnitureCounter", (0.7, 0.7, 0.75)),
    ("FurnitureBed", (0.4, 0.55, 0.8)),
    ("FurnitureBedGun", (0.4, 0.55, 0.8)),
]
TILE_VALUE = {name: i + 1 for i, (name, _) in enumerate(TILE_KINDS)}


def hex_color(rgb):
    return "#" + "".join(f"{round(c * 255):02X}" for c in rgb)


class TileGrid:
    """Mirror of tilemap.rs TileGrid (row-major, y-up)."""

    def __init__(self, width, height, fill):
        self.width = width
        self.height = height
        self.tiles = [fill] * (width * height)

    def set(self, x, y, kind):
        if 0 <= x < self.width and 0 <= y < self.height:
            self.tiles[y * self.width + x] = kind

    def get(self, x, y):
        return self.tiles[y * self.width + x]

    def fill_rect(self, x0, y0, x1, y1, kind):
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                self.set(x, y, kind)


# ---------------------------------------------------------------------
# Outdoor map (tilemap.rs: build_outdoor_grid)
# ---------------------------------------------------------------------

OUTDOOR_WIDTH = 17
OUTDOOR_HEIGHT = 7
OUTDOOR_PATH_Y = 2
OUTDOOR_DOOR_Y = 4
OUTDOOR_ROOF_Y = 5
NEIGHBOR1_X = 2
NEIGHBOR2_X = 7
BEN_HOUSE_X = 12

BEN_DOOR_POS = (BEN_HOUSE_X + 1, OUTDOOR_DOOR_Y)
OUTDOOR_SPAWN = (1, OUTDOOR_PATH_Y)
OUTDOOR_RETURN_POS = (BEN_HOUSE_X + 1, OUTDOOR_DOOR_Y - 1)
OUTDOOR_SIGNS = [
    ((NEIGHBOR1_X + 1, 1), 'A weathered sign: "The Johnsons"'),
    ((NEIGHBOR2_X + 1, 1), 'A weathered sign: "The Millers"'),
]


def place_house(grid, base_x, door_kind):
    grid.fill_rect(base_x, OUTDOOR_ROOF_Y, base_x + 2, OUTDOOR_ROOF_Y, "HouseWall")
    grid.set(base_x, OUTDOOR_DOOR_Y, "HouseWall")
    grid.set(base_x + 1, OUTDOOR_DOOR_Y, door_kind)
    grid.set(base_x + 2, OUTDOOR_DOOR_Y, "HouseWall")


def build_outdoor_grid():
    grid = TileGrid(OUTDOOR_WIDTH, OUTDOOR_HEIGHT, "Grass")
    grid.fill_rect(0, OUTDOOR_PATH_Y, OUTDOOR_WIDTH - 1, OUTDOOR_PATH_Y, "Path")
    place_house(grid, NEIGHBOR1_X, "DoorClosed")
    place_house(grid, NEIGHBOR2_X, "DoorClosed")
    place_house(grid, BEN_HOUSE_X, "DoorBen")
    grid.set(NEIGHBOR1_X + 1, 1, "SignPost")
    grid.set(NEIGHBOR2_X + 1, 1, "SignPost")
    return grid


# ---------------------------------------------------------------------
# House interior map (tilemap.rs: build_interior_grid)
# ---------------------------------------------------------------------

INTERIOR_WIDTH = 21
INTERIOR_HEIGHT = 15
EXIT_DOOR_POS = (10, 0)
INTERIOR_SPAWN = (10, 1)
MOM_POS = (5, 4)
DAD_POS = (15, 4)
OLDER_BROTHER_POS = (3, 11)
YOUNGER_BROTHER_POS = (10, 11)
GUN_SPOT_POS = (17, 11)
BEN_AMBUSH_ENTRY_POS = (19, 10)


def build_interior_grid():
    w, h = INTERIOR_WIDTH, INTERIOR_HEIGHT
    grid = TileGrid(w, h, "Floor")

    grid.fill_rect(0, 0, w - 1, 0, "Wall")
    grid.fill_rect(0, h - 1, w - 1, h - 1, "Wall")
    grid.fill_rect(0, 0, 0, h - 1, "Wall")
    grid.fill_rect(w - 1, 0, w - 1, h - 1, "Wall")
    grid.set(*EXIT_DOOR_POS, "ExitDoor")

    grid.fill_rect(1, 7, w - 2, 7, "Wall")
    for gap_x in (5, 10, 15):
        grid.set(gap_x, 7, "Floor")

    grid.fill_rect(10, 2, 10, 6, "Wall")
    grid.set(10, 3, "Floor")

    grid.fill_rect(7, 8, 7, 13, "Wall")
    grid.set(7, 10, "Floor")
    grid.fill_rect(14, 8, 14, 13, "Wall")
    grid.set(14, 10, "Floor")

    grid.set(3, 3, "FurnitureCouch")
    grid.set(7, 5, "FurnitureTv")
    grid.set(17, 3, "FurnitureCounter")
    grid.set(13, 5, "FurnitureTable")
    grid.set(2, 12, "FurnitureBed")
    grid.set(12, 12, "FurnitureBed")
    grid.set(*GUN_SPOT_POS, "FurnitureBedGun")
    return grid


# ---------------------------------------------------------------------
# LDtk project construction
# ---------------------------------------------------------------------

_next_uid = 1


def new_uid():
    global _next_uid
    uid = _next_uid
    _next_uid += 1
    return uid


def new_iid():
    return str(uuid.uuid4())


def enum_def(identifier, values):
    return {
        "uid": new_uid(),
        "identifier": identifier,
        "values": [{"id": v, "tileRect": None, "color": 0} for v in values],
        "iconTilesetUid": None,
        "externalRelPath": None,
        "externalFileChecksum": None,
        "tags": [],
    }


def field_def(identifier, kind, enum=None):
    """kind: 'String' or 'Enum' (pass the enum def)."""
    if kind == "Enum":
        type_name = f"LocalEnum.{enum['identifier']}"
        ldtk_type = f"F_Enum({enum['uid']})"
    else:
        type_name = "String"
        ldtk_type = "F_String"
    return {
        "identifier": identifier,
        "doc": None,
        "__type": type_name,
        "uid": new_uid(),
        "type": ldtk_type,
        "isArray": False,
        "canBeNull": False,
        "arrayMinLength": None,
        "arrayMaxLength": None,
        "editorDisplayMode": "ValueOnly",
        "editorDisplayScale": 1,
        "editorDisplayPos": "Above",
        "editorLinkStyle": "StraightArrow",
        "editorDisplayColor": None,
        "editorAlwaysShow": False,
        "editorShowInWorld": True,
        "editorCutLongValues": True,
        "editorTextSuffix": None,
        "editorTextPrefix": None,
        "useForSmartColor": False,
        "exportToToc": False,
        "searchable": False,
        "min": None,
        "max": None,
        "regex": None,
        "acceptFileTypes": None,
        "defaultOverride": None,
        "textLanguageMode": None,
        "symmetricalRef": False,
        "autoChainRef": True,
        "allowOutOfLevelRef": True,
        "allowedRefs": "OnlySame",
        "allowedRefsEntityUid": None,
        "allowedRefTags": [],
        "tilesetUid": None,
    }


def entity_def(identifier, color, fields):
    return {
        "identifier": identifier,
        "uid": new_uid(),
        "tags": [],
        "exportToToc": False,
        "allowOutOfBounds": False,
        "doc": None,
        "width": TILE_SIZE,
        "height": TILE_SIZE,
        "resizableX": False,
        "resizableY": False,
        "minWidth": None,
        "maxWidth": None,
        "minHeight": None,
        "maxHeight": None,
        "keepAspectRatio": False,
        "tileOpacity": 1,
        "fillOpacity": 0.3,
        "lineOpacity": 1,
        "hollow": False,
        "color": color,
        "renderMode": "Rectangle",
        "showName": True,
        "tilesetId": None,
        "tileRenderMode": "FitInside",
        "tileRect": None,
        "uiTileRect": None,
        "nineSliceBorders": [],
        "maxCount": 0,
        "limitScope": "PerLevel",
        "limitBehavior": "MoveLastOne",
        "pivotX": 0,
        "pivotY": 0,
        "fieldDefs": fields,
    }


def layer_def(identifier, layer_type, int_grid_values=()):
    return {
        "__type": layer_type,
        "identifier": identifier,
        "type": layer_type,
        "uid": new_uid(),
        "doc": None,
        "uiColor": None,
        "gridSize": TILE_SIZE,
        "guideGridWid": 0,
        "guideGridHei": 0,
        "displayOpacity": 1,
        "inactiveOpacity": 1,
        "hideInList": False,
        "hideFieldsWhenInactive": False,
        "canSelectWhenInactive": True,
        "renderInWorldView": True,
        "pxOffsetX": 0,
        "pxOffsetY": 0,
        "parallaxFactorX": 0,
        "parallaxFactorY": 0,
        "parallaxScaling": True,
        "requiredTags": [],
        "excludedTags": [],
        "autoTilesKilledByOtherLayerUid": None,
        "uiFilterTags": [],
        "useAsyncRender": False,
        "intGridValues": list(int_grid_values),
        "intGridValuesGroups": [],
        "autoRuleGroups": [],
        "autoSourceLayerDefUid": None,
        "tilesetDefUid": None,
        "tilePivotX": 0,
        "tilePivotY": 0,
        "biomeFieldUid": None,
    }


def ldtk_cell(pos, height):
    """Game (x, y-up) -> LDtk (col, row top-down)."""
    x, y = pos
    return x, height - 1 - y


def field_instance(fdef, value):
    return {
        "__identifier": fdef["identifier"],
        "__type": fdef["__type"],
        "__value": value,
        "__tile": None,
        "defUid": fdef["uid"],
        "realEditorValues": [{"id": "V_String", "params": [value]}],
    }


def entity_instance(edef, pos, level, values=None):
    col, row = ldtk_cell(pos, level["_height"])
    px = [col * TILE_SIZE, row * TILE_SIZE]
    fields_by_name = {f["identifier"]: f for f in edef["fieldDefs"]}
    return {
        "__identifier": edef["identifier"],
        "__grid": [col, row],
        "__pivot": [0, 0],
        "__tags": [],
        "__tile": None,
        "__smartColor": edef["color"],
        "__worldX": level["worldX"] + px[0],
        "__worldY": level["worldY"] + px[1],
        "iid": new_iid(),
        "width": TILE_SIZE,
        "height": TILE_SIZE,
        "defUid": edef["uid"],
        "px": px,
        "fieldInstances": [
            field_instance(fields_by_name[k], v) for k, v in (values or {}).items()
        ],
    }


def int_grid_csv(grid):
    csv = []
    for y in range(grid.height - 1, -1, -1):  # top row first
        for x in range(grid.width):
            csv.append(TILE_VALUE[grid.get(x, y)])
    return csv


def layer_instance(ldef, level, int_grid=None, entities=None):
    return {
        "__identifier": ldef["identifier"],
        "__type": ldef["type"],
        "__cWid": level["_width"],
        "__cHei": level["_height"],
        "__gridSize": TILE_SIZE,
        "__opacity": 1,
        "__pxTotalOffsetX": 0,
        "__pxTotalOffsetY": 0,
        "__tilesetDefUid": None,
        "__tilesetRelPath": None,
        "iid": new_iid(),
        "levelId": level["uid"],
        "layerDefUid": ldef["uid"],
        "pxOffsetX": 0,
        "pxOffsetY": 0,
        "visible": True,
        "optionalRules": [],
        "intGridCsv": int_grid or [],
        "autoLayerTiles": [],
        "seed": 1000000 + ldef["uid"],
        "overrideTilesetUid": None,
        "gridTiles": [],
        "entityInstances": entities or [],
    }


def new_level(identifier, width, height, world_x, world_y):
    return {
        "identifier": identifier,
        "iid": new_iid(),
        "uid": new_uid(),
        "worldX": world_x,
        "worldY": world_y,
        "worldDepth": 0,
        "pxWid": width * TILE_SIZE,
        "pxHei": height * TILE_SIZE,
        "__bgColor": "#696A79",
        "bgColor": None,
        "useAutoIdentifier": False,
        "bgRelPath": None,
        "bgPos": None,
        "bgPivotX": 0.5,
        "bgPivotY": 0.5,
        "__smartColor": "#ADADB5",
        "__bgPos": None,
        "externalRelPath": None,
        "fieldInstances": [],
        "layerInstances": [],
        "__neighbours": [],
        # Private helpers, stripped before export.
        "_width": width,
        "_height": height,
    }


def build_project():
    # Enums
    npc_enum = enum_def("NpcId", ["Mom", "Dad", "OlderBrother", "YoungerBrother", "Ben"])
    dir_enum = enum_def("Direction", ["Up", "Down", "Left", "Right"])
    door_enum = enum_def("DoorTarget", ["EnterHouse", "ExitHouse"])
    spawn_enum = enum_def("SpawnId", ["OutdoorSpawn", "OutdoorReturn", "InteriorSpawn"])

    # Entities
    npc = entity_def("Npc", "#E0A030", [
        field_def("npc_id", "Enum", npc_enum),
        field_def("facing", "Enum", dir_enum),
    ])
    door = entity_def("Door", "#F0D040", [field_def("target", "Enum", door_enum)])
    sign = entity_def("Sign", "#FFFFFF", [field_def("text", "String")])
    spawn = entity_def("SpawnPoint", "#40C0F0", [
        field_def("id", "Enum", spawn_enum),
        field_def("facing", "Enum", dir_enum),
    ])
    gun_spot = entity_def("GunSpot", "#FFD91A", [])
    ambush = entity_def("AmbushEntry", "#E04040", [])

    # Layers (defs order == draw order, top first)
    entities_layer = layer_def("Entities", "Entities")
    terrain_layer = layer_def("Terrain", "IntGrid", [
        {
            "value": TILE_VALUE[name],
            "identifier": name,
            "color": hex_color(rgb),
            "tile": None,
            "groupUid": 0,
        }
        for name, rgb in TILE_KINDS
    ])

    # --- Outdoor level
    outdoor = new_level("Outdoor", OUTDOOR_WIDTH, OUTDOOR_HEIGHT, 0, 0)
    outdoor_entities = [
        entity_instance(door, BEN_DOOR_POS, outdoor, {"target": "EnterHouse"}),
        *[entity_instance(sign, pos, outdoor, {"text": text}) for pos, text in OUTDOOR_SIGNS],
        entity_instance(spawn, OUTDOOR_SPAWN, outdoor, {"id": "OutdoorSpawn", "facing": "Up"}),
        entity_instance(spawn, OUTDOOR_RETURN_POS, outdoor, {"id": "OutdoorReturn", "facing": "Down"}),
    ]
    outdoor["layerInstances"] = [
        layer_instance(entities_layer, outdoor, entities=outdoor_entities),
        layer_instance(terrain_layer, outdoor, int_grid=int_grid_csv(build_outdoor_grid())),
    ]

    # --- Interior level, placed to the right of the outdoor level
    interior_x = (OUTDOOR_WIDTH + 2) * TILE_SIZE
    interior = new_level("HouseInterior", INTERIOR_WIDTH, INTERIOR_HEIGHT, interior_x, 0)
    interior_entities = [
        entity_instance(door, EXIT_DOOR_POS, interior, {"target": "ExitHouse"}),
        entity_instance(spawn, INTERIOR_SPAWN, interior, {"id": "InteriorSpawn", "facing": "Up"}),
        entity_instance(npc, MOM_POS, interior, {"npc_id": "Mom", "facing": "Down"}),
        entity_instance(npc, DAD_POS, interior, {"npc_id": "Dad", "facing": "Down"}),
        entity_instance(npc, OLDER_BROTHER_POS, interior, {"npc_id": "OlderBrother", "facing": "Down"}),
        entity_instance(npc, YOUNGER_BROTHER_POS, interior, {"npc_id": "YoungerBrother", "facing": "Down"}),
        entity_instance(gun_spot, GUN_SPOT_POS, interior),
        entity_instance(ambush, BEN_AMBUSH_ENTRY_POS, interior),
    ]
    interior["layerInstances"] = [
        layer_instance(entities_layer, interior, entities=interior_entities),
        layer_instance(terrain_layer, interior, int_grid=int_grid_csv(build_interior_grid())),
    ]

    levels = [outdoor, interior]
    for level in levels:
        del level["_width"], level["_height"]

    return {
        "__header__": {
            "fileType": "LDtk Project JSON",
            "app": "LDtk",
            "doc": "https://ldtk.io/json",
            "schema": "https://ldtk.io/files/JSON_SCHEMA.json",
            "appAuthor": "Sebastien 'deepnight' Benard",
            "appVersion": LDTK_VERSION,
            "url": "https://ldtk.io",
        },
        "iid": new_iid(),
        "jsonVersion": LDTK_VERSION,
        "appBuildId": 473703,
        "nextUid": _next_uid,
        "identifierStyle": "Free",
        "toc": [],
        "worldLayout": "Free",
        "worldGridWidth": 256,
        "worldGridHeight": 256,
        "defaultLevelWidth": 512,
        "defaultLevelHeight": 512,
        "defaultPivotX": 0,
        "defaultPivotY": 0,
        "defaultGridSize": TILE_SIZE,
        "defaultEntityWidth": TILE_SIZE,
        "defaultEntityHeight": TILE_SIZE,
        "bgColor": "#40465B",
        "defaultLevelBgColor": "#696A79",
        "minifyJson": False,
        "externalLevels": False,
        "exportTiled": False,
        "simplifiedExport": False,
        "imageExportMode": "None",
        "exportLevelBg": True,
        "pngFilePattern": None,
        "backupOnSave": False,
        "backupLimit": 10,
        "backupRelPath": None,
        "levelNamePattern": "Level_%idx",
        "tutorialDesc": None,
        "customCommands": [],
        "flags": [],
        "defs": {
            "layers": [entities_layer, terrain_layer],
            "entities": [npc, door, sign, spawn, gun_spot, ambush],
            "tilesets": [],
            "enums": [npc_enum, dir_enum, door_enum, spawn_enum],
            "externalEnums": [],
            "levelFields": [],
        },
        "levels": levels,
        "worlds": [],
        "dummyWorldIid": new_iid(),
    }


def main():
    project = build_project()
    OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUT_PATH.write_text(json.dumps(project, indent="\t") + "\n")
    print(f"Wrote {OUT_PATH.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
