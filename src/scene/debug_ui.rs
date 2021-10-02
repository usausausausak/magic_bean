#![allow(unused)]
use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, Diagnostics};
use bevy_mod_picking::*;

use crate::AppState;
use crate::cube::BallSensor;

struct FpsText;
struct SensorDetailText;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_system(setup_fpi_ui.system())
            .add_system(fps_ui.system());

        #[cfg(not(target_family = "wasm"))]
        app.add_plugin(DebugCursorPickingPlugin)
            .add_startup_system(setup_detail_ui.system())
            .add_system_set(
                SystemSet::on_update(AppState::Pause)
                    .with_system(sensor_detail_ui.system())
             );
    }
}

fn setup_fpi_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let style = TextStyle {
        font: asset_server.load("FiraMono-Medium.ttf"),
        font_size: 40.0,
        color: Color::rgb(0.0, 1.0, 1.0),
    };

    commands.spawn_bundle(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                ..Default::default()
            },
            margin: Rect::all(Val::Px(2.0)),
            ..Default::default()
        },
        text: Text {
            sections: vec![
                TextSection {
                    value: "Average FPS: ".to_string(),
                    style: style.clone(),
                },
                TextSection {
                    value: "".to_string(),
                    style: style.clone(),
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    }).insert(FpsText);
}

fn setup_detail_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let style = TextStyle {
        font: asset_server.load("FiraMono-Medium.ttf"),
        font_size: 40.0,
        color: Color::rgb(0.0, 1.0, 1.0),
    };

    commands.spawn_bundle(NodeBundle {
        style: Style {
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        material: materials.add(Color::NONE.into()),
        ..Default::default()
    })
    .with_children(|parent| {
        for _ in 0..6 {
            parent.spawn_bundle(TextBundle {
                style: Style {
                    margin: Rect::all(Val::Px(2.0)),
                    ..Default::default()
                },
                text: Text::with_section("", style.clone(), Default::default()),
                ..Default::default()
            }).insert(SensorDetailText);
        }
    });
}

fn fps_ui(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    let mut text = query.single_mut().unwrap();
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            // Update the value of the second section
            text.sections[1].value = format!("{:.2}", average);
        }
    }
}

fn sensor_detail_ui(
    sensor_query: Query<(&Name, &BallSensor)>,
    mut text_query: Query<&mut Text, With<SensorDetailText>>,
) {
    for ((name, sensor), mut text) in sensor_query.iter().zip(text_query.iter_mut()) {
        text.sections[0].value = format!("{}: {}", name.as_str(), sensor);
    }
}
