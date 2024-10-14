use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use lightyear::prelude::server::ServerTransport;
use lightyear::server::config::ServerConfig;
use shared::prelude::*;

mod server_plugin;
use server_plugin::*;

// This needs to be passed to the matchmaker service as a cli flag too, since it's needed to
// construct the ConnectTokens.
// TODO this should be read from ENV or flag or file.. or maybe we deterministically generate it
//      based on the cert hash, which we already send to the matchmaker?
pub const PRIVATE_KEY: [u8; PRIVATE_KEY_BYTES] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];

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

    info!("bevygap-spaceships server main() ..");
    info!("⭐️ Build time: {}", env!("VERGEN_BUILD_TIMESTAMP"));
    info!("⭐️ Git desc: {}", env!("VERGEN_GIT_DESCRIBE"));
    info!("⭐️ Git sha: {}", env!("VERGEN_GIT_SHA"));
    info!("⭐️ Git commit @ {}", env!("VERGEN_GIT_COMMIT_TIMESTAMP"));

    // configure the network configuration
    let (net_config, cert_digest) = build_server_netcode_config();

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
    app.add_plugins(BLEMServerPlugin { cert_digest });

    app.run();
}

pub fn build_server_netcode_config() -> (server::NetConfig, String) {
    let conditioner = None;

    /*
    Generates a self-signed certificate and private key for new identity.

    The certificate conforms to the W3C WebTransport specifications as follows:

    The certificate MUST be an X.509v3 certificate as defined in RFC5280.
    The key used in the Subject Public Key field MUST be one of the allowed public key algorithms. This function uses the ECDSA P-256 algorithm.
    The current time MUST be within the validity period of the certificate as defined in Section 4.1.2.5 of RFC5280.
    The total length of the validity period MUST NOT exceed two weeks.
     */
    let mut sans = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];
    // Are we running on edgegap?
    if let Ok(public_ip) = std::env::var("ARBITRIUM_PUBLIC_IP") {
        info!("🔐 SAN += ARBITRIUM_PUBLIC_IP: {}", public_ip);
        sans.push(public_ip);
        sans.push("*.pr.edgegap.net".to_string());
    }
    // generic env to add domains and ips to SAN list:
    // SELF_SIGNED_SANS="example.org,example.com,127.1.1.1"
    if let Ok(san) = std::env::var("SELF_SIGNED_SANS") {
        info!("🔐 SAN += SELF_SIGNED_SANS: {}", san);
        sans.extend(san.split(',').map(|s| s.to_string()));
    }
    info!("🔐 Creating self-signed certificate with SANs: {:?}", sans);
    let certificate = server::Identity::self_signed(sans).unwrap();
    // newer version of wtransport has this API:
    // let identity = server::Identity::self_signed_builder()
    //     .subject_alt_names(&["localhost", "127.0.0.1", "::1"])
    //     .from_now_utc()
    //     .validity_days(14)
    //     .build()
    //     .unwrap();

    // We could load certs from files if needed. This would make more sense for a deployment
    // where you own the domain name and have a certificate management solution in place,
    // for example with LetsEncrypt.
    //
    // this is async because we need to load the certificate from io
    // we need async_compat because wtransport expects a tokio reactor
    // let certificate = IoTaskPool::get()
    //     .scope(|s| {
    //         s.spawn(Compat::new(async {
    //             server::Identity::load_pemfiles("./certificates/cert.pem", "./certificates/key.pem")
    //                 .await
    //                 .unwrap()
    //         }));
    //     })
    //     .pop()
    //     .unwrap();

    let digest = certificate.certificate_chain().as_slice()[0].hash();
    let digest_str = format!("{}", digest);
    info!("🔐 Certificate digest: {}", digest);

    // in edgegap or other cloud environments, or even just docker containers, you don't generally
    // know what your public IP is, so we just listen on everything and let the network
    // layer (docker, EC2 NAT, whatever) hook you up.
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

    #[cfg(not(feature = "bevygap"))]
    let key = DUMMY_PRIVATE_KEY;
    #[cfg(feature = "bevygap")]
    let key = PRIVATE_KEY;

    // this is to aid debugging, silly to dump it to the logs most of the time.
    // info!("🔐 Using private key: {:?}", key);

    let netcode_config = server::NetcodeConfig::default()
        .with_protocol_id(PROTOCOL_ID)
        .with_key(key);

    (
        server::NetConfig::Netcode {
            config: netcode_config,
            io: io_config,
        },
        digest_str,
    )
}
