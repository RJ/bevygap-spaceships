use async_compat::Compat;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use lightyear::prelude::server::ServerTransport;
use lightyear::server::config::ServerConfig;
use shared::prelude::*;

mod server_plugin;
use server_plugin::*;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "gui")]
    app.add_plugins(
        DefaultPlugins
            .build()
            // logger added with custom config below
            .disable::<LogPlugin>(),
    );

    #[cfg(not(feature = "gui"))]
    app.add_plugins((
        MinimalPlugins,
        HierarchyPlugin,
        bevy::state::app::StatesPlugin,
    ));

    app.add_plugins(LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error".to_string(),
        ..default() // filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
                    // update_subscriber: Some(add_log_layer),
    });

    // configure the network configuration
    let net_config = build_server_netcode_config();
    // we can listen on multiple interfaces, or steam+wt+udp etc
    let net_configs = vec![net_config];

    app.add_plugins(server::ServerPlugins {
        config: ServerConfig {
            shared: shared::shared_config(),
            net: net_configs,
            replication: ReplicationConfig {
                send_interval: SERVER_REPLICATION_INTERVAL,
                ..default()
            },
            ..default()
        },
    });

    app.add_plugins(BLEMSharedPlugin);
    app.add_plugins(BLEMServerPlugin);

    app.run();
}

pub fn build_server_netcode_config() -> server::NetConfig {
    let conditioner = None;

    // this is async because we need to load the certificate from io
    // we need async_compat because wtransport expects a tokio reactor
    let certificate = IoTaskPool::get()
        .scope(|s| {
            s.spawn(Compat::new(async {
                server::Identity::load_pemfiles("./certificates/cert.pem", "./certificates/key.pem")
                    .await
                    .unwrap()
            }));
        })
        .pop()
        .unwrap();

    let digest = certificate.certificate_chain().as_slice()[0].hash();
    println!("Generated self-signed certificate with digest: {}", digest);

    let listen_addr = format!("0.0.0.0:{SERVER_PORT}").parse().unwrap();

    info!("Listening on {listen_addr:?}");

    let transport_config = ServerTransport::WebTransportServer {
        server_addr: listen_addr,
        certificate,
    };

    let io_config = server::IoConfig {
        transport: transport_config,
        conditioner,
        compression: CompressionConfig::None,
    };

    let key = DUMMY_PRIVATE_KEY;
    #[cfg(all(feature = "server", feature = "bevygap"))]
    let key = PRIVATE_KEY;

    let netcode_config = server::NetcodeConfig::default()
        .with_protocol_id(PROTOCOL_ID)
        .with_key(key);

    server::NetConfig::Netcode {
        config: netcode_config,
        io: io_config,
    }
}
