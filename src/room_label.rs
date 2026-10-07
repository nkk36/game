use bevy::prelude::*;

use crate::grid::GridPos;
use crate::ldtk::{CurrentLevel, GameMaps};
use crate::player::Player;
use crate::states::AppState;

#[derive(Component)]
pub struct RoomLabelText;

/// Top-right counterpart to the quest hint, naming the room the player is in.
pub fn spawn_room_label_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            Visibility::Hidden,
            RoomLabelText,
        ))
        .with_children(|parent| {
            parent.spawn((Text::new(""), TextFont { font_size: 18.0.into(), ..default() }, TextColor(Color::WHITE)));
        });
}

/// Shows the name of the `Room` the player stands in. Doorways between
/// rooms usually aren't part of either, so the label keeps the last room's
/// name until the player is fully inside the next one.
pub fn room_label_system(
    app_state: Res<State<AppState>>,
    maps: Res<GameMaps>,
    current_level: Res<CurrentLevel>,
    player_q: Query<&GridPos, With<Player>>,
    mut label_q: Query<(&mut Visibility, &Children), With<RoomLabelText>>,
    mut text_q: Query<&mut Text>,
) {
    let Ok((mut visibility, children)) = label_q.single_mut() else {
        return;
    };
    let exploring = matches!(app_state.get(), AppState::Outdoor | AppState::HouseInterior);
    let room = player_q.single().ok().and_then(|p| maps.levels[current_level.0].room_at(*p));

    visibility.set_if_neq(if exploring { Visibility::Inherited } else { Visibility::Hidden });
    let (Some(room), true) = (room, exploring) else {
        return;
    };
    if let Some(mut text) = children.first().and_then(|c| text_q.get_mut(*c).ok()) {
        if text.0 != room.name {
            text.0 = room.name.to_string();
        }
    }
}
