// use avian2d::*;
// use avian2d::{prelude::*, sync::SyncPlugin};
// use bevy::ecs::query::QueryData;
// use bevy::prelude::*;
// use client::Predicted;
// use client::prediction::Predicted;
// use connection::id::ClientId;

use lightyear::prelude::*;

// use shared::config::{Mode, SharedConfig};
mod protocol_plugin;
mod shared_plugin;

#[cfg(feature = "gui")]
mod entity_label;
#[cfg(feature = "gui")]
mod renderer;

// use shared::plugin::NetworkIdentity;
// use shared::replication::components::Controlled;

pub mod prelude {
    pub use bevy::utils::Duration;

    pub const SERVER_PORT: u16 = 6420;
    pub const PHYSICS_SCALE: f32 = 100.0;
    pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(20);
    pub const PROTOCOL_ID: u64 = 80085;
    pub const WALL_SIZE: f32 = 350.0;
    pub const FIXED_TIMESTEP_HZ: f64 = 64.0;
    pub const MAX_VELOCITY: f32 = 200.0;
    pub use std::f32::consts::TAU;
    pub const CERTIFICATE_DIGEST: &str = "db:3c:d4:0d:b5:40:e2:7e:9e:8f:96:4e:95:7c:d8:52:8b:3c:9b:33:f6:af:56:b6:e9:9f:23:fa:43:82:98:ed";

    // For non-bevygap (ie, non-connect token) builds, we use a dummy zeroed key on client and server
    pub const DUMMY_PRIVATE_KEY: [u8; PRIVATE_KEY_BYTES] = [0; PRIVATE_KEY_BYTES];

    pub use super::protocol_plugin::*;
    pub use super::shared_plugin::*;

    #[cfg(feature = "gui")]
    pub use super::entity_label::*;
    #[cfg(feature = "gui")]
    pub use super::renderer::*;

    pub use avian2d::prelude::*;
    pub use leafwing_input_manager::prelude::ActionState;
    pub use lightyear::connection::netcode::PRIVATE_KEY_BYTES;
    pub use lightyear::prelude::{client::Predicted, *};
    pub use lightyear::shared::replication::components::Controlled;
}

use prelude::*;

/// The [`SharedConfig`] must be shared between the `ClientConfig` and `ServerConfig`
pub fn shared_config() -> SharedConfig {
    SharedConfig {
        server_replication_send_interval: SERVER_REPLICATION_INTERVAL,
        tick: lightyear::shared::tick_manager::TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        },
        mode: Mode::Separate,
    }
}
