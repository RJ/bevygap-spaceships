use crate::screens::*;
use bevy::{color::palettes::css, prelude::*};
use client::NetClient;
use lightyear::connection::client::ConnectionState;
use shared::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.observe(update_connect_status_text_observer);
    app.add_systems(OnEnter(Screen::Connect), spawn_connect_screen);
    // systems that only run in Connect state.
    app.add_systems(
        Update,
        (
            continue_to_gameplay_screen.run_if(connected_to_server),
            button_system,
        )
            .run_if(in_state(Screen::Connect)),
    );
    #[cfg(feature = "bevygap")]
    app.add_systems(
        Update,
        on_bevygap_state_change.run_if(in_state(Screen::Connect)),
    );
}

fn continue_to_gameplay_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Gameplay);
}

fn connected_to_server(connection: Res<client::ClientConnection>) -> bool {
    matches!(connection.state(), ConnectionState::Connected)
}

// We need a "Connect Now" button, and a status text to update during connection.

// Marker tag for loading screen components.
#[derive(Component)]
struct ConnectUIText;
#[derive(Component)]
struct ConnectUIButton;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn spawn_connect_screen(mut commands: Commands, _asset_server: ResMut<AssetServer>) {
    info!("spawn_connect_screen");
    let text_style = TextStyle {
        font_size: 30.0,
        ..default()
    };

    commands
        .spawn((
            StateScoped(Screen::Connect),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ConnectUIButton,
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            margin: UiRect {
                                bottom: Val::Px(20.0),
                                ..default()
                            },
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        border_radius: BorderRadius::MAX,
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Connect",
                        TextStyle {
                            font_size: 22.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });

            parent.spawn((
                ConnectUIText,
                TextBundle::from_sections([TextSection::new("Standing By", text_style.clone())]),
            ));
        });
}

#[derive(Event, Clone)]
struct ConnectStatusText(String);

/// Emitted when user clicks the connect button.
#[derive(Event)]
pub(crate) struct ConnectToServerRequest;

fn update_connect_status_text_observer(
    trigger: Trigger<ConnectStatusText>,
    mut q: Query<&mut Text, With<ConnectUIText>>,
) {
    if let Ok(mut text) = q.get_single_mut() {
        text.sections[0].value.clone_from(&trigger.event().0);
    }
}

#[cfg(feature = "bevygap")]
fn on_bevygap_state_change(
    state: Res<State<bevygap_client_plugin::BevygapClientState>>,
    mut commands: Commands,
) {
    use bevygap_client_plugin::BevygapClientState;

    let msg = match state.get() {
        BevygapClientState::Dormant => "Chrome only atm!",
        BevygapClientState::Request => "Making request...",
        BevygapClientState::AwaitingResponse => "Waiting for matchmaker...",
        BevygapClientState::ReadyToConnect => "Ready!",
        BevygapClientState::Finished => "Finished connection setup.",
        BevygapClientState::Error => "Error connecting.",
    };
    commands.trigger(ConnectStatusText(msg.to_string()));
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<ConnectUIButton>),
    >,
    mut commands: Commands,
) {
    for (interaction, mut color, mut border_color, _children) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = css::RED.into();
                info!("PRESSED");
                commands.trigger(ConnectStatusText("Connecting to server...".to_string()));
                commands.trigger(ConnectToServerRequest);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}
