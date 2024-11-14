use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use client::{Authentication, ClientConfig, PredictionConfig};

use shared::prelude::*;
mod client_plugin;
pub(crate) mod screens;
use client_plugin::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<LogPlugin>()
            .set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            }),
    );
    app.add_plugins(LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
        ..default() // update_subscriber: Some(add_log_layer),
    });

    let prediction = PredictionConfig {
        correction_ticks_factor: 1.5,
        minimum_input_delay_ticks: 3,
        maximum_input_delay_before_prediction: 6,
        ..Default::default()
    };

    info!("{prediction:?}");

    let client_config = ClientConfig {
        shared: shared::shared_config(),
        net: get_client_net_config(),
        prediction,
        ..default()
    };
    // lightyear client plugins
    app.add_plugins(client::ClientPlugins {
        config: client_config,
    });
    app.add_plugins(BevygapSpaceshipsSharedPlugin);
    app.add_plugins(BevygapSpaceshipsClientPlugin);

    app.run();
}

fn get_client_net_config() -> client::NetConfig {
    let client_addr = "0.0.0.0:0".parse().unwrap();
    // this gets overwritten if using connect tokens
    let server_addr = format!("127.0.0.1:{SERVER_PORT}").parse().unwrap();

    // pick a client id, but it will be overwridden by connect token if supplied
    #[cfg(target_arch = "wasm32")]
    let client_id: u64 = (web_sys::js_sys::Math::random() * u64::MAX as f64) as u64;
    #[cfg(not(target_arch = "wasm32"))]
    let client_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    info!("ClientID initially set to {client_id}");

    let conditioner = None;

    let auth = Authentication::Manual {
        server_addr,
        client_id,
        private_key: DUMMY_PRIVATE_KEY,
        protocol_id: PROTOCOL_ID,
    };

    let netcode_config = client::NetcodeConfig::default();

    #[cfg(target_family = "wasm")]
    let certificate_digest =
        std::env::var("LIGHTYEAR_CERTIFICATE_DIGEST").unwrap_or("".to_string());

    let transport_config = client::ClientTransport::WebTransportClient {
        client_addr,
        server_addr,
        #[cfg(target_family = "wasm")]
        certificate_digest,
    };

    let io_config = client::IoConfig {
        transport: transport_config,
        conditioner,
        compression: CompressionConfig::None,
    };

    client::NetConfig::Netcode {
        auth,
        config: netcode_config,
        io: io_config,
    }
}
