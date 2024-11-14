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

    /// Must match on server and client. Bump it whenever you make breaking changes to the protocol.
    pub const PROTOCOL_ID: u64 = 80085;
    pub const SERVER_PORT: u16 = 6420;
    pub const PHYSICS_SCALE: f32 = 100.0;
    pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(20);
    pub const WALL_SIZE: f32 = 350.0;
    pub const FIXED_TIMESTEP_HZ: f64 = 64.0;
    pub const MAX_VELOCITY: f32 = 200.0;
    pub use std::f32::consts::TAU;

    // For non-bevygap (ie, non-connect token) builds, we use a dummy zeroed key on client and server
    pub const DUMMY_PRIVATE_KEY: [u8; PRIVATE_KEY_BYTES] = [0; PRIVATE_KEY_BYTES];

    pub use super::protocol_plugin::*;
    pub use super::shared_plugin::*;

    #[cfg(feature = "gui")]
    pub use super::entity_label::*;
    #[cfg(feature = "gui")]
    pub use super::renderer::*;

    pub use super::read_lightyear_private_key_from_env;
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

/// Reads and parses the LIGHTYEAR_PRIVATE_KEY environment variable into a private key.
pub fn read_lightyear_private_key_from_env() -> Option<[u8; PRIVATE_KEY_BYTES]> {
    let Ok(key_str) = std::env::var("LIGHTYEAR_PRIVATE_KEY") else {
        return None;
    };
    let private_key: Vec<u8> = key_str
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == ',')
        .collect::<String>()
        .split(',')
        .map(|s| {
            s.parse::<u8>()
                .expect("Failed to parse number in private key")
        })
        .collect();

    if private_key.len() != PRIVATE_KEY_BYTES {
        panic!(
            "Private key must contain exactly {} numbers",
            PRIVATE_KEY_BYTES
        );
    }

    let mut bytes = [0u8; PRIVATE_KEY_BYTES];
    bytes.copy_from_slice(&private_key);
    Some(bytes)
}
