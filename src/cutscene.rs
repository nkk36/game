use bevy::prelude::*;

use crate::grid::{Direction, Facing, GridPos, MoveTween, TILE_SIZE};
use crate::npc::{spawn_npc, NpcId};
use crate::player::Player;
use crate::states::{AppState, InputLock};
use crate::tilemap::{GunPickupProp, InteriorEntity, BEN_AMBUSH_ENTRY_POS, GUN_SPOT_POS};

#[derive(PartialEq, Eq, Clone, Copy)]
enum CutscenePhase {
    BenEnters,
    BenGrabsGun,
    SquirtHit,
    Done,
}

#[derive(Resource)]
pub struct CutsceneState {
    phase: CutscenePhase,
    timer: Timer,
}

impl Default for CutsceneState {
    fn default() -> Self {
        Self { phase: CutscenePhase::BenEnters, timer: Timer::from_seconds(0.4, TimerMode::Once) }
    }
}

#[derive(Component)]
pub(crate) struct BenActor;

#[derive(Component)]
struct SplashFx;

pub fn start_cutscene_system(mut commands: Commands, mut cutscene: ResMut<CutsceneState>, mut input_lock: ResMut<InputLock>) {
    input_lock.0 = true;
    *cutscene = CutsceneState::default();
    let entity = spawn_npc(&mut commands, NpcId::Ben, BEN_AMBUSH_ENTRY_POS, Direction::Left);
    commands.entity(entity).insert(BenActor);
}

pub fn cutscene_sequencer_system(
    time: Res<Time>,
    mut commands: Commands,
    mut cutscene: ResMut<CutsceneState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ben_q: Query<(Entity, &mut GridPos, &mut Facing), (With<BenActor>, Without<MoveTween>)>,
    gun_q: Query<Entity, With<GunPickupProp>>,
    player_q: Query<&Transform, With<Player>>,
) {
    cutscene.timer.tick(time.delta());
    if !cutscene.timer.is_finished() {
        return;
    }

    match cutscene.phase {
        CutscenePhase::BenEnters => {
            if let Ok((entity, mut pos, mut facing)) = ben_q.single_mut() {
                let start = pos.to_world();
                facing.dir = Direction::Left;
                *pos = GUN_SPOT_POS;
                commands.entity(entity).insert(MoveTween::new(start, GUN_SPOT_POS.to_world()));
            }
            cutscene.phase = CutscenePhase::BenGrabsGun;
            cutscene.timer = Timer::from_seconds(0.5, TimerMode::Once);
        }
        CutscenePhase::BenGrabsGun => {
            for gun in &gun_q {
                commands.entity(gun).despawn();
            }
            cutscene.phase = CutscenePhase::SquirtHit;
            cutscene.timer = Timer::from_seconds(0.6, TimerMode::Once);
        }
        CutscenePhase::SquirtHit => {
            if let Ok(player_t) = player_q.single() {
                commands.spawn((
                    Sprite::from_color(Color::srgb(0.3, 0.7, 1.0), Vec2::splat(TILE_SIZE * 0.6)),
                    Transform::from_xyz(player_t.translation.x, player_t.translation.y, 4.0),
                    SplashFx,
                    InteriorEntity,
                ));
            }
            cutscene.phase = CutscenePhase::Done;
            cutscene.timer = Timer::from_seconds(0.8, TimerMode::Once);
        }
        CutscenePhase::Done => {
            next_state.set(AppState::EndScreen);
        }
    }
}
