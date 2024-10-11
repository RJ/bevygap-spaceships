use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use client::{Authentication, ClientConfig, PredictionConfig};

use shared::prelude::*;

mod client_plugin;
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

    app.add_plugins(BLEMSharedPlugin);
    app.add_plugins(BLEMClientPlugin);
    app.run();
}

fn get_client_net_config() -> client::NetConfig {
    let client_addr = "0.0.0.0:0".parse().unwrap();
    // this gets overwritten if using connect tokens
    let server_addr = format!("127.0.0.1:{SERVER_PORT}").parse().unwrap();

    // pick a client id, but it will be overwridden by connect token if supplied
    let client_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let conditioner = None;

    let auth = Authentication::Manual {
        server_addr,
        client_id,
        private_key: DUMMY_PRIVATE_KEY,
        protocol_id: PROTOCOL_ID,
    };

    let netcode_config = client::NetcodeConfig::default();

    let transport_config = client::ClientTransport::WebTransportClient {
        client_addr,
        server_addr,
        #[cfg(target_family = "wasm")]
        certificate_digest: CERTIFICATE_DIGEST.to_string().replace(":", ""),
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
