use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
#[cfg(feature = "bevygap")]
use bevygap_server_plugin::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::server::{Replicate, SyncTarget};
use lightyear::prelude::{server::*, *};
use lightyear::server::connection::ConnectionManager;
use lightyear::server::events::MessageEvent;
use shared::prelude::*;

#[derive(Default)]
pub struct BevygapSpaceshipsServerPlugin {
    pub cert_digest: String,
}

impl Plugin for BevygapSpaceshipsServerPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "bevygap")]
        {
            // only start listening once bevygap setup complete
            warn!("cert_digest: {}", self.cert_digest.clone());
            app.add_plugins(BevygapServerPlugin::self_signed_digest(
                self.cert_digest.clone(),
            ));
            app.observe(start_listening_once_bevygap_ready);
        }
        #[cfg(not(feature = "bevygap"))]
        {
            // without bevygap we just start listening immediately.
            app.add_systems(Startup, start_listening);
        }

        app.add_systems(Startup, init);
        app.add_systems(
            PreUpdate,
            // this system will replicate the inputs of a client to other clients
            // so that a client can predict other clients
            replicate_inputs.after(MainSet::EmitEvents),
        );
        // the physics/FixedUpdates systems that consume inputs should be run in this set
        app.add_systems(
            FixedUpdate,
            (player_movement, shared_player_firing)
                .chain()
                .in_set(FixedSet::Main),
        );
        app.add_systems(
            Update,
            (
                handle_connections,
                update_player_metrics.run_if(on_timer(Duration::from_secs(1))),
            ),
        );

        app.add_systems(
            FixedUpdate,
            handle_hit_event
                .run_if(on_event::<BulletHitEvent>())
                .after(process_collisions),
        );

        #[cfg(feature = "bevygap")]
        app.add_systems(
            Update,
            update_server_metadata.run_if(resource_added::<ArbitriumContext>),
        );
    }
}

// fn report_certificate_digest

#[cfg(feature = "bevygap")]
fn update_server_metadata(
    mut metadata: ResMut<ServerMetadata>,
    context: Res<ArbitriumContext>,
    mut commands: Commands,
) {
    metadata.fqdn = context.fqdn();
    metadata.location = context.location();
    metadata.build_info = format!(
        "Git: {} built at: {}",
        env!("VERGEN_GIT_DESCRIBE"),
        env!("VERGEN_BUILD_TIMESTAMP")
    );
    info!("Updating server metadata: {metadata:?}");
    commands.replicate_resource::<ServerMetadata, ResourceChannel>(NetworkTarget::All);
}

/// Since Player is replicated, this allows the clients to display remote players' latency stats.
fn update_player_metrics(
    connection_manager: Res<ConnectionManager>,
    mut q: Query<(Entity, &mut Player)>,
) {
    for (_e, mut player) in q.iter_mut() {
        if let Ok(connection) = connection_manager.connection(player.client_id) {
            player.rtt = connection.rtt();
            player.jitter = connection.jitter();
        }
    }
}

#[cfg(not(feature = "bevygap"))]
fn start_listening(mut commands: Commands) {
    commands.start_server();
}

#[cfg(feature = "bevygap")]
fn start_listening_once_bevygap_ready(_trigger: Trigger<BevygapReady>, mut commands: Commands) {
    info!("Starting to listen - bevygap reports ready");
    commands.start_server();
}

fn init(mut commands: Commands) {
    #[cfg(feature = "gui")]
    {
        commands.spawn(
            TextBundle::from_section(
                "Server",
                TextStyle {
                    font_size: 30.0,
                    color: Color::WHITE,
                    ..default()
                },
            )
            .with_style(Style {
                align_self: AlignSelf::End,
                ..default()
            }),
        );
    }
    // the balls are server-authoritative
    const NUM_BALLS: usize = 6;
    for i in 0..NUM_BALLS {
        let radius = 10.0 + i as f32 * 4.0;
        let angle: f32 = i as f32 * (TAU / NUM_BALLS as f32);
        let pos = Vec2::new(125.0 * angle.cos(), 125.0 * angle.sin());
        commands.spawn(BallBundle::new(radius, pos, css::GOLD.into()));
    }
}

pub(crate) fn replicate_inputs(
    mut connection: ResMut<ConnectionManager>,
    mut input_events: ResMut<Events<MessageEvent<InputMessage<PlayerActions>>>>,
) {
    for mut event in input_events.drain() {
        let client_id = *event.context();

        // Optional: do some validation on the inputs to check that there's no cheating
        // Inputs for a specific tick should be write *once*. Don't let players change old inputs.

        // rebroadcast the input to other clients
        connection
            .send_message_to_target::<InputChannel, _>(
                &mut event.message,
                NetworkTarget::AllExceptSingle(client_id),
            )
            .unwrap()
    }
}

/// Whenever a new client connects, spawn their spaceship
pub(crate) fn handle_connections(
    mut connections: EventReader<ConnectEvent>,
    mut commands: Commands,
    all_players: Query<Entity, With<Player>>,
) {
    // track the number of connected players in order to pick colors and starting positions
    let mut player_n = all_players.iter().count();
    for connection in connections.read() {
        let client_id = connection.client_id;
        info!("New connected client, client_id: {client_id:?}. Spawning player entity..");
        // replicate newly connected clients to all players
        let replicate = Replicate {
            sync: SyncTarget {
                prediction: NetworkTarget::All,
                ..default()
            },
            controlled_by: ControlledBy {
                target: NetworkTarget::Single(client_id),
                ..default()
            },
            // make sure that all entities that are predicted are part of the same replication group
            group: REPLICATION_GROUP,
            ..default()
        };
        // pick color and x,y pos for player

        let available_colors = [
            css::LIMEGREEN,
            css::PINK,
            css::YELLOW,
            css::AQUA,
            css::CRIMSON,
            css::GOLD,
            css::ORANGE_RED,
            css::SILVER,
            css::SALMON,
            css::YELLOW_GREEN,
            css::WHITE,
            css::RED,
        ];
        let col = available_colors[player_n % available_colors.len()];
        let angle: f32 = player_n as f32 * 5.0;
        let x = 200.0 * angle.cos();
        let y = 200.0 * angle.sin();

        // spawn the player with ActionState - the client will add their own InputMap
        let player_ent = commands
            .spawn((
                Player::new(client_id, pick_player_name(client_id.to_bits())),
                Score(0),
                Name::new("Player"),
                ActionState::<PlayerActions>::default(),
                Position(Vec2::new(x, y)),
                replicate,
                PhysicsBundle::player_ship(),
                Weapon::new((FIXED_TIMESTEP_HZ / 5.0) as u16),
                ColorComponent(col.into()),
            ))
            .id();

        info!("Created entity {player_ent:?} for client {client_id:?}");
        player_n += 1;
    }
}

fn pick_player_name(client_id: u64) -> String {
    let index = (client_id % NAMES.len() as u64) as usize;
    NAMES[index].to_string()
}

const NAMES: [&str; 35] = [
    "Ellen Ripley",
    "Sarah Connor",
    "Neo",
    "Trinity",
    "Morpheus",
    "John Connor",
    "T-1000",
    "Rick Deckard",
    "Princess Leia",
    "Han Solo",
    "Spock",
    "James T. Kirk",
    "Hikaru Sulu",
    "Nyota Uhura",
    "Jean-Luc Picard",
    "Data",
    "Beverly Crusher",
    "Seven of Nine",
    "Doctor Who",
    "Rose Tyler",
    "Marty McFly",
    "Doc Brown",
    "Dana Scully",
    "Fox Mulder",
    "Riddick",
    "Barbarella",
    "HAL 9000",
    "Megatron",
    "Furiosa",
    "Lois Lane",
    "Clark Kent",
    "Tony Stark",
    "Natasha Romanoff",
    "Bruce Banner",
    "Mr. T",
];

/// Server will manipulate scores when a bullet collides with a player.
/// the `Score` component is a simple replication. scores fully server-authoritative.
pub(crate) fn handle_hit_event(
    connection_manager: Res<server::ConnectionManager>,
    mut events: EventReader<BulletHitEvent>,
    client_q: Query<&ControlledEntities, Without<Player>>,
    mut player_q: Query<(&Player, &mut Score)>,
) {
    let client_id_to_player_entity = |client_id: ClientId| -> Option<Entity> {
        if let Ok(e) = connection_manager.client_entity(client_id) {
            if let Ok(controlled_entities) = client_q.get(e) {
                return controlled_entities.entities().pop();
            }
        }
        None
    };

    for ev in events.read() {
        // did they hit a player?
        if let Some(victim_entity) = ev.victim_client_id.and_then(client_id_to_player_entity) {
            if let Ok((_player, mut score)) = player_q.get_mut(victim_entity) {
                score.0 -= 1;
            }
            if let Some(shooter_entity) = client_id_to_player_entity(ev.bullet_owner) {
                if let Ok((_player, mut score)) = player_q.get_mut(shooter_entity) {
                    score.0 += 1;
                }
            }
        }
    }
}

/// Read inputs and move players
///
/// If we didn't receive the input for a given player, we do nothing (which is the default behaviour from lightyear),
/// which means that we will be using the last known input for that player
/// (i.e. we consider that the player kept pressing the same keys).
/// see: https://github.com/cBournhonesque/lightyear/issues/492
pub(crate) fn player_movement(
    mut q: Query<(&ActionState<PlayerActions>, ApplyInputsQuery), With<Player>>,
    tick_manager: Res<TickManager>,
) {
    let tick = tick_manager.tick();
    for (action_state, mut aiq) in q.iter_mut() {
        apply_action_state_to_player_movement(action_state, 0, &mut aiq, tick);
    }
}
