use bevy::prelude::*;
use bevy::core::FloatOrd;
use bevy_mod_picking::PickableBundle;

use crate::AppState;
use crate::component::*;
use debug::DebugVisible;
use crate::input;

use crate::util::otry;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, SystemLabel)]
enum Stage {
    SyncTransform,
}

pub struct CubePlugin;

impl Plugin for CubePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .init_resource::<CubeDescriptor>()
            .init_resource::<GrabStatus>()
            .add_event::<SnapEvent>()
            .add_system_set(SystemSet::on_enter(AppState::Setup)
                .with_system(setup_ball.system())
                .with_system(setup_sensor.system())
                .with_system(setup_cube.system())
            )
            .add_system_set(SystemSet::on_update(AppState::Pause)
                .before(Stage::SyncTransform)
                .with_system(animation.system())
            )
            .add_system_set(SystemSet::new()
                .before(Stage::SyncTransform)
                .with_system(input::grab.system().chain(input::cube_rotate.system()))
            )
            .add_system_set(SystemSet::on_enter(AppState::InGame)
                .with_system(cube_transform.system())
                .with_system(ball_transform.system())
                .with_system(slide_block_transform.system())
                .with_system(rotate_block_transform.system())
            )
            .add_system_set(SystemSet::on_update(AppState::InGame)
                .before(Stage::SyncTransform)
                .with_system(input::drag.system().chain(input::apply_movement.system()))
                .with_system(input::key.system().chain(input::apply_movement.system()))
                .with_system(input::reset.system())
            )
            .add_system_set(SystemSet::new()
                .label(Stage::SyncTransform)
                .with_system(cube_transform.system())
                .with_system(ball_transform.system())
                .with_system(slide_block_transform.system())
                .with_system(rotate_block_transform.system())
            )
            .add_system_set(SystemSet::new()
                .after(Stage::SyncTransform)
                .with_system(trace_ball.system())
            )
            .add_system_set(SystemSet::on_update(AppState::InGame)
                .after(Stage::SyncTransform)
                .with_system(snap.system())
            );
    }
}

fn setup_cube(
    mut commands: Commands,
    cube: Res<CubeDescriptor>,
    query: QuerySet<(
        Query<(Entity, &Children), With<Cube>>,
        Query<(Entity, &Name, &Children, &Block)>,
        Query<&Children, With<Deco>>,
    )>,
    mesh_query: Query<&Handle<Mesh>>,
) {
    let get_first_mesh_child = |children: &Children| {
        children.iter().find(|c| mesh_query.get(**c).is_ok()).unwrap().clone()
    };

    let (entity, children) = query.q0().single().unwrap();
    commands.entity(entity)
        .insert(CubeRotation::new(0.5, -0.05));
    commands.entity(get_first_mesh_child(children))
        .insert(DebugVisible::No)
        .insert_bundle(PickableBundle::default()); // no grab kind

    for (entity, name, children, block) in query.q1().iter() {
        let (grab_kind, sensor) = match block {
            Block::Slide => {
                let shape = cube.slide_sensor_box();
                let capacity = cube.slide_capacity();
                let sensor = BallSensor::new(shape, capacity);
                commands.entity(entity).insert(SlideHandle::left());
                (MovementKind::Slide, sensor)
            }
            Block::Rotate => {
                let shape = cube.rotate_sensor_box();
                let capacity = cube.rotate_capacity();
                let sensor = BallSensor::new(shape, capacity);
                commands.entity(entity)
                    .insert_bundle((SlideHandle::left(), RotateHandle::up()));
                (MovementKind::Rotate, sensor)
            }
        };
        commands.entity(get_first_mesh_child(children))
            .insert(name.clone())
            .insert(sensor)
            .insert(grab_kind)
            .insert(DebugVisible::No)
            .insert_bundle(PickableBundle::default());
    }

    for children in query.q2().iter() {
        commands.entity(get_first_mesh_child(children))
            .insert(DebugVisible::No);
    }
}

fn setup_ball(
    mut commands: Commands,
    cube: Res<CubeDescriptor>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cube_query: Query<Entity, With<Cube>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Icosphere { radius: cube.ball_radians(), subdivisions: 12 }));
    let get_material = {
        let mut add_material = |base_color| {
            materials.add(StandardMaterial {
                base_color,
                roughness: 0.3,
                reflectance: 0.2,
                ..Default::default()
            })
        };
        let a = add_material(Color::rgb(0.6, 0.2, 0.1));
        let b = add_material(Color::rgb(0.2, 0.6, 0.1));
        let c = add_material(Color::rgb(0.1, 0.2, 0.6));
        let d = add_material(Color::rgb(0.6, 0.2, 0.6));
        use BallColor::*;
        move |ball| {
            match ball {
                A => a.clone(),
                B => b.clone(),
                C => c.clone(),
                D => d.clone(),
            }
        }
    };

    let color_iter = {
        use std::iter::repeat;

        let mut colors = repeat(BallColor::A).take(cube.balls_per_color())
            .chain(repeat(BallColor::B).take(cube.balls_per_color()))
            .chain(repeat(BallColor::C).take(cube.balls_per_color()))
            .chain(repeat(BallColor::D).take(cube.balls_per_color()))
            .collect::<Vec<_>>();
        fastrand::shuffle(&mut colors);
        colors.into_iter()
    };

    let cube_entity = cube_query.single().unwrap();
    commands.entity(cube_entity).with_children(|parent| {
        for (handle, color) in cube.ball_init_handle_iter().zip(color_iter) {
            let material = get_material(color);

            let pbr = PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                ..Default::default()
            };

            parent.spawn_bundle(pbr)
                .insert(color)
                .insert_bundle(handle);
        }
    });
}

fn setup_sensor(
    mut commands: Commands,
    cube: Res<CubeDescriptor>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<Entity, With<Cube>>,
) {
    #[derive(Bundle)]
    struct SensorBundle {
        sensor: BallSensor,
        #[bundle]
        pbr: PbrBundle,
        debug: DebugVisible,
    }
    fn sensor_bundle(
        capacity: usize,
        point: Vec3,
        shape: shape::Box,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
    ) -> SensorBundle {
        let pbr = PbrBundle {
            mesh,
            material,
            transform: Transform::from_translation(point.into()),
            //visible: Visible { is_visible: false, is_transparent: true },
            visible: Visible { is_visible: true, is_transparent: true },
            ..Default::default()
        };

        SensorBundle {
            sensor: BallSensor::new(shape, capacity),
            pbr,
            debug: DebugVisible::Yes,
        }
    }

    let cube_entity = query.single().unwrap();
    commands.entity(cube_entity).with_children(|parent| {
        let shape = cube.ball_sensor_box();
        let mesh = meshes.add(Mesh::from(shape));
        //let material = materials.add(Color::rgba(0.0, 0.7, 0.5, 0.4).into());
        let material = materials.add(Color::rgba(0.0, 0.0, 0.0, 0.0).into());

        for (name, origin) in cube.group_origin_iter() {
            let sensor = sensor_bundle(cube.group_capacity(), origin, shape, mesh.clone(), material.clone());

            parent.spawn_bundle(sensor)
                .insert(Name::new(name.to_string()))
                .insert(MovementKind::Path)
                .insert_bundle(PickableBundle::default());
        }
    });
}

fn cube_transform(
    mut query: Query<(&mut Transform, &CubeRotation), Changed<CubeRotation>>,
) {
    let (mut transform, rotation) = otry!(query.single_mut().ok());
    transform.rotation = rotation.to_quat();
}

type BallChanged = Or<(Changed<PathHandle>, Changed<SlideHandle>, Changed<RotateHandle>)>;
fn ball_transform(
    cube: Res<CubeDescriptor>,
    mut query: Query<(&mut Transform, &PathHandle, &SlideHandle, &RotateHandle), BallChanged>,
) {
    for (mut transform, path_h, slide_h, rotate_h) in query.iter_mut() {
        let bundle = BallHandleBundle {
            path: *path_h,
            slide: *slide_h,
            rotate: *rotate_h,
        };
        *transform = cube.get_ball_transform(&bundle);
    }
}

fn slide_block_transform(
    cube: Res<CubeDescriptor>,
    mut query: Query<(&mut Transform, &SlideHandle), (Changed<SlideHandle>, With<Block>)>,
) {
    for (mut transform, handle) in query.iter_mut() {
        let v = cube.slide_path.evaluate(*handle);
        transform.translation.x = v.x;
    }
}

fn rotate_block_transform(
    cube: Res<CubeDescriptor>,
    mut query: Query<(&mut Transform, &RotateHandle), (Changed<RotateHandle>, With<Block>)>,
) {
    for (mut transform, handle) in query.iter_mut() {
        let q = cube.rotate_path.evaluate(*handle);
        transform.rotation = q;
    }
}

fn trace_ball(
    mut sensor_query: Query<(&Transform, &mut BallSensor)>,
    ball_query: Query<(Entity, &Transform, &BallColor, &PathHandle)>,
) {
    for (transform, mut sensor) in sensor_query.iter_mut() {
        let sensor_origin = transform.translation;
        sensor.detail.clear();
        for (entity, transform, color, handle) in ball_query.iter() {
            let translation = transform.translation - sensor_origin;
            if sensor.intersect_point(translation) {
                sensor.detail.push((entity, *color, FloatOrd(handle.t.to_f32())));
            }
        }
        sensor.detail.sort_by_key(|(_, _, t)| *t);
    }
}

fn snap(
    mut commands: Commands,
    cube: Res<CubeDescriptor>,
    mut state: ResMut<State<AppState>>,
    mut events: EventReader<SnapEvent>,
    block_query: QuerySet<(
        Query<Entity, (With<SlideHandle>, With<Block>)>,
        Query<Entity, (With<RotateHandle>, With<Block>)>,
    )>,
    sensor_query: Query<&BallSensor>,
    path_query: Query<&PathHandle>,
    slide_query: Query<&SlideHandle>,
    rotate_query: Query<&RotateHandle>,
) {
    fn new_animation_bundle(kind: MovementKind, to: f32, from: f32) -> (Animation, Timer) {
        let offset = to - from;
        (
            Animation { kind, movement: offset, base: from },
            Timer::from_seconds(0.2, false),
        )
    }

    fn snap3(t: f32) -> f32 {
        const HHALF: f32 = 0.5 / 2.0;
        if t < HHALF {
            0.0
        } else if t > 0.5 + HHALF {
            1.0
        } else {
            0.5
        }
    }

    let SnapEvent(grabbing) = *otry!(events.iter().next());
    let kind = grabbing.kind;
    let sensor = sensor_query.get(grabbing.entity).unwrap();
    if !sensor.is_full() {
        return
    }
    match kind {
        MovementKind::Path => {
            let mut iter = sensor.entities();

            let first = iter.next().unwrap();
            let handle = path_query.get(first).unwrap().t.to_f32();

            let mut to = cube.path_snap_first(handle);
            let animation = new_animation_bundle(kind, to, handle);
            commands.entity(first).insert_bundle(animation);

            for entity in iter {
                let handle = path_query.get(entity).unwrap().t.to_f32();
                to += cube.ball_step();
                let animation = new_animation_bundle(kind, to, handle);
                commands.entity(entity).insert_bundle(animation);
            }
        }
        MovementKind::Slide => {
            for entity in block_query.q0().iter().chain(sensor.entities()) {
                let handle = slide_query.get(entity).unwrap().t.to_f32();
                let to = snap3(handle);
                let animation = new_animation_bundle(kind, to, handle);
                commands.entity(entity).insert_bundle(animation);
            }
        }
        MovementKind::Rotate => {
            for entity in block_query.q1().iter().chain(sensor.entities()) {
                let handle = rotate_query.get(entity).unwrap().t.to_f32();
                let to = snap3(handle);
                let animation = new_animation_bundle(kind, to, handle);
                commands.entity(entity).insert_bundle(animation);
            }
        }
    }

    state.set(AppState::Pause).unwrap();
}

fn animation(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        Option<&mut PathHandle>, Option<&mut SlideHandle>, Option<&mut RotateHandle>,
        &Animation, &mut Timer,
    )>,
) {
    let mut i = 0;
    for (entity, path_h, slide_h, rotate_h, animation, mut timer) in query.iter_mut() {
        i += 1;
        let t = timer.tick(time.delta()).percent();
        match animation.kind {
            MovementKind::Path => {
                let mut handle = path_h.unwrap();
                *handle = PathHandle::new(animation.base + animation.movement * t);
            }
            MovementKind::Slide => {
                let mut handle = slide_h.unwrap();
                *handle = SlideHandle::new(animation.base + animation.movement * t);
            }
            MovementKind::Rotate => {
                let mut handle = rotate_h.unwrap();
                *handle = RotateHandle::new(animation.base + animation.movement * t);
            }
        }

        if timer.just_finished() {
            commands.entity(entity).remove_bundle::<(Animation, Timer)>();
        }
    }

    if i == 0 {
        state.set(AppState::InGame).unwrap();
    }
}
