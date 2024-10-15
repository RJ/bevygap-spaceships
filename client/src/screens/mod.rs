mod connect;
mod gameplay;

use bevy::prelude::*;
pub(crate) use connect::ConnectToServerRequest;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.enable_state_scoped_entities::<Screen>();

    app.add_plugins((connect::plugin, gameplay::plugin));
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum Screen {
    #[default]
    Connect,
    Gameplay,
}
