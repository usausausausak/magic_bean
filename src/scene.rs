use bevy::prelude::*;
use bevy_mod_picking::*;

use crate::AppState;
use crate::component::*;
use crate::component::debug::*;

pub struct ScenePlugin;

impl ScenePlugin {
    #[cfg(not(target_family = "wasm"))]
    fn add_default_plugin(&self, app: &mut AppBuilder) {
        app.insert_resource(Msaa { samples: 4 })
            .add_plugins(DefaultPlugins);
    }

    #[cfg(target_family = "wasm")]
    fn add_default_plugin(&self, app: &mut AppBuilder) {
        app.insert_resource(Msaa { samples: 4 })
            .add_plugins(DefaultPlugins)
            // when building for Web, use WebGL2 rendering
            .add_plugin(bevy_webgl2::WebGL2Plugin);
    }
}

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut AppBuilder) {
        self.add_default_plugin(app);

        app
            .add_plugin(PickingPlugin)
            .insert_resource(Debug::Off)
            .add_state(AppState::Load)
            .add_startup_system(setup_ui.system())
            .add_system(debug.system())
            .add_system_set(SystemSet::on_enter(AppState::Load)
                .with_system(setup_scene.system())
            )
            .add_system_set(SystemSet::on_update(AppState::Load)
                .with_system(load_complation.system())
            )
            .add_system_set(SystemSet::on_exit(AppState::Load)
                .with_system(tag_entity.system())
            )
            .add_system_set(SystemSet::on_update(AppState::Setup)
                .with_system(setup_complation.system())
            );
    }
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    if cfg!(feature = "public") {
        commands.spawn_scene(asset_server.load("untitled6.glb#Scene0"));
    } else {
        commands.spawn_scene(asset_server.load("untitled6.gltf#Scene0"));
    }

    // light
    let lights = [
        (Vec3::new(-9.0, 9.0, -8.0), 800.0),
        (Vec3::new(6.0, 6.0, 10.0), 3000.0),
        (Vec3::new(-9.0, 6.0, 10.0), 200.0),
    ];
    for (point, intensity) in std::array::IntoIter::new(lights) {
        commands.spawn_bundle(LightBundle {
            light: Light {
                intensity,
                ..Default::default()
            },
            transform: Transform::from_translation(point),
            ..Default::default()
        });
    }

    // camera
    let point = Vec3::new(0.0, 0.0, 10.0);
    let slash_point = Vec3::new(0.0, 0.0, 20.0);
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(point).looking_at(slash_point, Vec3::Y),
            ..Default::default()
        })
        .insert(MainCamera);
}

fn load_complation(
    mut state: ResMut<State<AppState>>,
    mut events: EventReader<AssetEvent<Scene>>
) {
    for ev in events.iter() {
        match ev {
            AssetEvent::Created { .. } => {
                //eprintln!("load");
                state.set(AppState::Setup).unwrap();
            },
            _ => (),
        }
    }
}

fn tag_entity(mut commands: Commands, query: Query<(Entity, &Name)>) {
    for (entity, name) in query.iter() {
        let name = name.as_str();
        //eprintln!("{} {}", entity.id(), name);
        if name == "cube" {
            commands.entity(entity).insert(Cube);
        } else if name == "block.slide" {
            commands.entity(entity).insert(Block::Slide);
        } else if name == "block.rotate" {
            commands.entity(entity).insert(Block::Rotate);
        } else {
            commands.entity(entity).insert(Deco);
        }
    }
}

fn setup_complation(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    mut query: Query<(Entity, &mut Transform), With<MainCamera>>,
) {
    {
        let (camera, mut transform) = query.single_mut().unwrap();
        let point = Vec3::new(0.0, 0.0, 10.0);
        *transform = Transform::from_translation(point).looking_at(Vec3::ZERO, Vec3::Y);
        commands.entity(camera)
            .insert_bundle(PickingCameraBundle::default());
    }

    state.set(AppState::InGame).unwrap();
}

fn setup_ui(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}

fn debug(
    key: Res<Input<KeyCode>>,
    mut debug: ResMut<Debug>,
    mut visible_query: Query<(&mut Visible, &DebugVisible)>,
) {
    if key.just_pressed(KeyCode::X) {
        let (new, enter_debug) = match *debug {
            Debug::On  => (Debug::Off, false),
            Debug::Off => (Debug::On,  true),
        };

        for (mut visible, debug_visible) in visible_query.iter_mut() {
            visible.is_visible = match debug_visible {
                DebugVisible::Yes => enter_debug,
                DebugVisible::No => !enter_debug,
            };
        }

        *debug = new;
    }
}
