use crate::FIXED_TIMESTEP_HZ;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::utils::Duration;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

use lightyear::client::components::ComponentSyncMode;
// use lightyear::client::interpolation::LinearInterpolator;
// use lightyear::prelude::client::{self, LeafwingInputConfig};
use lightyear::prelude::server::{Replicate, SyncTarget};
use lightyear::prelude::*;
use lightyear::utils::avian2d::*;
// use tracing_subscriber::util::SubscriberInitExt;

// use crate::color_from_id;

pub const BULLET_SIZE: f32 = 1.5;
pub const SHIP_WIDTH: f32 = 19.0;
pub const SHIP_LENGTH: f32 = 32.0;

// For prediction, we want everything entity that is predicted to be part of the same replication group
// This will make sure that they will be replicated in the same message and that all the entities in the group
// will always be consistent (= on the same tick)
pub const REPLICATION_GROUP: ReplicationGroup = ReplicationGroup::new_id(1);

// Bullet
#[derive(Bundle)]
pub struct BulletBundle {
    position: Position,
    velocity: LinearVelocity,
    color: ColorComponent,
    marker: BulletMarker,
    lifetime: Lifetime,
}

impl BulletBundle {
    pub fn new(
        owner: ClientId,
        position: Vec2,
        velocity: Vec2,
        color: Color,
        current_tick: Tick,
    ) -> Self {
        Self {
            position: Position(position),
            velocity: LinearVelocity(velocity),
            color: ColorComponent(color),
            lifetime: Lifetime {
                origin_tick: current_tick,
                lifetime: FIXED_TIMESTEP_HZ as i16 * 2,
            },
            marker: BulletMarker::new(owner),
        }
    }
}

// Ball
#[derive(Bundle)]
pub struct BallBundle {
    position: Position,
    color: ColorComponent,
    replicate: Replicate,
    marker: BallMarker,
    physics: PhysicsBundle,
    name: Name,
}

impl BallBundle {
    pub fn new(radius: f32, position: Vec2, color: Color) -> Self {
        let ball = BallMarker::new(radius);
        let sync_target = SyncTarget {
            prediction: NetworkTarget::All,
            ..default()
        };
        let replicate = Replicate {
            sync: sync_target,
            group: REPLICATION_GROUP,
            ..default()
        };
        Self {
            position: Position(position),
            color: ColorComponent(color),
            replicate,
            physics: ball.physics_bundle(),
            marker: ball,
            name: Name::new("Ball"),
        }
    }
}

#[derive(Bundle)]
pub struct PhysicsBundle {
    pub collider: Collider,
    pub collider_density: ColliderDensity,
    pub rigid_body: RigidBody,
    pub external_force: ExternalForce,
}

impl PhysicsBundle {
    pub fn bullet() -> Self {
        Self {
            collider: Collider::circle(BULLET_SIZE),
            collider_density: ColliderDensity(5.0),
            rigid_body: RigidBody::Dynamic,
            external_force: ExternalForce::default(),
        }
    }

    pub fn player_ship() -> Self {
        // triangle ship, pointing up the screen
        let points = vec![
            Vec2::new(0.0, SHIP_LENGTH / 2.),
            Vec2::new(SHIP_WIDTH / 2., -SHIP_LENGTH / 2.),
            Vec2::new(-SHIP_WIDTH / 2., -SHIP_LENGTH / 2.),
        ];
        let collider = Collider::convex_hull(points).unwrap();
        // Note: due to a bug in older (?) versions of bevy_xpbd, using a triangle collider here
        // sometimes caused strange behaviour. Unsure if this is fixed now.
        // Also, counter-clockwise ordering of points was required for convex hull creation (?)
        Self {
            collider,
            collider_density: ColliderDensity(1.0),
            rigid_body: RigidBody::Dynamic,
            external_force: ExternalForce::ZERO.with_persistence(false),
        }
    }
}

// Components
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct Player {
    pub client_id: ClientId,
    pub nickname: String,
    pub rtt: Duration,
    pub jitter: Duration,
}

impl Player {
    pub fn new(client_id: ClientId, nickname: String) -> Self {
        Self {
            client_id,
            nickname,
            rtt: Duration::ZERO,
            jitter: Duration::ZERO,
        }
    }
}

/// A shared system generates these events on server and client.
/// On the server, we use them to manipulate player scores;
/// On the clients, we just use them for visual effects.
#[derive(Event, Debug)]
pub struct BulletHitEvent {
    pub bullet_owner: ClientId,
    pub bullet_color: Color,
    /// if it struck a player, this is their clientid:
    pub victim_client_id: Option<ClientId>,
    pub position: Vec2,
}

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ColorComponent(pub Color);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BallMarker {
    pub radius: f32,
}

impl BallMarker {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }

    pub fn physics_bundle(&self) -> PhysicsBundle {
        PhysicsBundle {
            collider: Collider::circle(self.radius),
            collider_density: ColliderDensity(1.5),
            rigid_body: RigidBody::Dynamic,
            external_force: ExternalForce::ZERO.with_persistence(false),
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BulletMarker {
    pub owner: ClientId,
}

impl BulletMarker {
    pub fn new(owner: ClientId) -> Self {
        Self { owner }
    }
}

// Limiting firing rate: once you fire on `last_fire_tick` you have to wait `cooldown` ticks before firing again.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Weapon {
    pub last_fire_tick: Tick,
    pub cooldown: u16,
    pub bullet_speed: f32,
}

impl Weapon {
    pub fn new(cooldown: u16) -> Self {
        Self {
            last_fire_tick: Tick(0),
            bullet_speed: 500.0,
            cooldown,
        }
    }
}

// increases if you hit another player with a bullet, decreases if you get hit.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Score(pub i32);

// despawns `lifetime` ticks after `origin_tick`
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Lifetime {
    pub origin_tick: Tick,
    /// number of ticks to live for
    pub lifetime: i16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum PlayerActions {
    Up,
    Down,
    Left,
    Right,
    Fire,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Resource)]
pub struct ServerMetadata {
    pub location: String,
    pub fqdn: String,
}

/// Just used to replicate resources, like ServerMetadata
#[derive(Channel)]
pub struct ResourceChannel;

// Protocol
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServerMetadata>();

        app.add_plugins(LeafwingInputPlugin::<PlayerActions>::default());

        // Player is synced as Simple, because we periodically update rtt ping stats
        app.register_component::<Player>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple);

        app.register_component::<ColorComponent>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<Name>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<BallMarker>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<BulletMarker>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<Lifetime>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);

        app.register_component::<Score>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple);

        // Fully replicated, but not visual, so no need for lerp/corrections:

        app.register_component::<LinearVelocity>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<AngularVelocity>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<Weapon>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full);

        // Position and Rotation have a `correction_fn` set, which is used to smear rollback errors
        // over a few frames, just for the rendering part in postudpate.
        //
        // They also set `interpolation_fn` which is used by the VisualInterpolationPlugin to smooth
        // out rendering between fixedupdate ticks.
        app.register_component::<Position>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation_fn(position::lerp)
            .add_correction_fn(position::lerp);

        app.register_component::<Rotation>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation_fn(rotation::lerp)
            .add_correction_fn(rotation::lerp);

        app.add_channel::<ResourceChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        });
    }
}
