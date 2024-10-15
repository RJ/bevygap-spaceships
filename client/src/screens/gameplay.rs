use crate::screens::*;
use bevy::prelude::*;

// this mostly exists to clean up state scoped stuff from prior state.

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), enter_gameplay_state);
}

fn enter_gameplay_state() {
    info!("Entering gameplay state");
}
