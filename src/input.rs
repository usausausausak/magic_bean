use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy_mod_picking::{PickingCamera, Primitive3d};

use crate::component::*;

use crate::util::{otry, range01::WrappingF32};

pub(super) fn cube_rotate(
    In(is_grabbing): In<bool>,
    mouse: Res<Input<MouseButton>>,
    mut ev_motion: EventReader<MouseMotion>,
    mut query: Query<&mut CubeRotation>,
) {
    if is_grabbing {
        return;
    }

    if mouse.pressed(MouseButton::Left) {
        let mut movement = Vec2::ZERO;
        for ev in ev_motion.iter() {
            movement += ev.delta;
        }

        let mut rotation = query.single_mut().unwrap();
        rotation.yaw += movement.x / 100.0;
        rotation.pitch += movement.y / 200.0;
    }
}

pub(super) fn drag(
    cube: Res<CubeDescriptor>,
    grab_status: Res<GrabStatus>,
    mut ev_motion: EventReader<MouseMotion>,
    cube_query: Query<&CubeRotation>,
    picking_query: Query<&PickingCamera>,
) -> Option<Movement> {
    let grabbing = grab_status.grabbing?;
    let cube_rotation = cube_query.single().unwrap();

    let picking = picking_query.single().unwrap();
    let picking_ray = picking.ray()?;
    let plane = Primitive3d::Plane {
            normal: picking_ray.direction(),
            point: grab_status.origin,
        };
    let cursor_pos = picking.intersect_primitive(plane)?.position();

    let mut cursor_v = Vec2::ZERO;
    for ev in ev_motion.iter() {
        cursor_v += ev.delta;
    }
    let movement = cursor_v.length();

    // speed down some
    let movement = match grabbing.kind {
        MovementKind::Path => {
            let s = cube.path_sign(grab_status.origin, cursor_pos, cursor_v);
            let s = if cube_rotation.is_face_back() { -s } else { s };
            movement.copysign(s) / 600.0
        }
        MovementKind::Slide   => movement.copysign(cursor_v.x) / 100.0,
        MovementKind::Rotate  => {
            let s = if cube_rotation.is_face_back() { -cursor_v.x } else { cursor_v.x };
            movement.copysign(s) / 200.0
        }
    };

    Some(Movement { grabbing, movement })
}

pub(super) fn grab(
    mouse: Res<Input<MouseButton>>,
    mut grab_status: ResMut<GrabStatus>,
    mut events: EventWriter<SnapEvent>,
    picking_query: Query<&PickingCamera>,
    sensor_query: Query<(&BallSensor, &MovementKind)>,
) -> bool {
    if mouse.just_pressed(MouseButton::Left) {
        let (entity, intersection) = otry!(picking_query.single().unwrap().intersect_top(), false);
        let (sensor, kind) = otry!(sensor_query.get(entity).ok(), false);
        if sensor.is_full() {
            let grabbing = GrabbingSensor { kind: *kind, entity };
            grab_status.grabbing = Some(grabbing);
            grab_status.origin = intersection.position();
        }
    }

    if mouse.just_released(MouseButton::Left) {
        if let Some(grabbing) = grab_status.grabbing.take() {
            events.send(SnapEvent(grabbing));
        }
    }

    grab_status.grabbing.is_some()
}

pub(super) fn key(
    key: Res<Input<KeyCode>>,
    cube: Res<CubeDescriptor>,
    mut cube_query: Query<&mut CubeRotation>,
    block_query: Query<&SlideHandle, With<Block>>,
    sensor_query: Query<(Entity, &Name, &BallSensor)>,
) -> Option<Movement> {
    let mut movement = None;
    if key.just_pressed(KeyCode::W) {
        movement = Some(("group", MovementKind::Path, cube.ball_step()));
    }
    if key.just_pressed(KeyCode::S) {
        movement = Some(("group", MovementKind::Path, -cube.ball_step()));
    }

    if key.just_pressed(KeyCode::A) {
        movement = Some(("block.slide", MovementKind::Slide, -1.0));
    }
    if key.just_pressed(KeyCode::D) {
        movement = Some(("block.slide", MovementKind::Slide, 1.0));
    }

    if key.just_pressed(KeyCode::R) {
        movement = Some(("block.rotate", MovementKind::Rotate, 0.5));
    }
    if key.just_pressed(KeyCode::T) {
        movement = Some(("block.rotate", MovementKind::Rotate, -0.5));
    }

    if key.just_pressed(KeyCode::Q) {
        let mut rotation = cube_query.single_mut().unwrap();
        if rotation.is_face_back() {
            rotation.pitch = WrappingF32::new(-0.05);
        } else {
            rotation.pitch = WrappingF32::new(0.45);
        }
        movement = None;
    }

    let (name, kind, movement) = movement?;
    let is_face_back = cube_query.single_mut().unwrap().is_face_back();
    let (name, movement) = match kind {
        MovementKind::Path => {
            let is_slide_left = block_query.iter().next().unwrap().t.to_f32() < 0.5;
            let name = match (is_face_back, is_slide_left) {
                (false,  true) => "group.a",
                (false, false) => "group.b",
                ( true,  true) => "group.c",
                ( true, false) => "group.d",
            };
            let movement = if is_face_back { -movement } else { movement };
            (name, movement)
        }
        MovementKind::Slide => {
            (name, movement)
        }
        MovementKind::Rotate => {
            let movement = if is_face_back { -movement } else { movement };
            (name, movement)
        }
    };

    let (entity, _, sensor) = sensor_query.iter()
        .filter(|(_, n, _)| n.as_str() == name).next().unwrap();
    if sensor.is_full() {
        let grabbing = GrabbingSensor { kind, entity };
        Some(Movement { grabbing, movement })
    } else {
        None
    }
}

pub(super) fn apply_movement(
    In(movement): In<Option<Movement>>,
    mut block_query: QuerySet<(
        Query<Entity, (With<SlideHandle>, With<Block>)>,
        Query<Entity, (With<RotateHandle>, With<Block>)>,
    )>,
    sensor_query: Query<&BallSensor>,
    path_query: Query<&mut PathHandle>,
    slide_query: Query<&mut SlideHandle>,
    rotate_query: Query<&mut RotateHandle>,
) {
    let Movement { grabbing, movement } = otry!(movement);
    let sensor = sensor_query.get(grabbing.entity).unwrap();
    if !sensor.is_full() {
        return;
    }
    match grabbing.kind {
        MovementKind::Path => {
            path(movement, &sensor, path_query);
        }
        MovementKind::Slide => {
            let mut block_query = block_query.q0_mut();
            slide(movement, &sensor, &mut block_query, slide_query);
        }
        MovementKind::Rotate => {
            let mut block_query = block_query.q1_mut();
            rotate(movement, &sensor, &mut block_query, rotate_query);
        }
    }
}

fn path(
    movement: f32,
    sensor: &BallSensor,
    mut query: Query<&mut PathHandle>,
) {
    for entity in sensor.entities() {
        let mut handle = query.get_mut(entity).unwrap();
        handle.t += movement;
    }
}

fn slide(
    movement: f32,
    sensor: &BallSensor,
    block_query: &mut Query<Entity, (With<SlideHandle>, With<Block>)>,
    mut query: Query<&mut SlideHandle>,
) {
    for entity in block_query.iter_mut().chain(sensor.entities()) {
        let mut handle = query.get_mut(entity).unwrap();
        handle.t += movement;
    }
}

fn rotate(
    movement: f32,
    sensor: &BallSensor,
    block_query: &mut Query<Entity, (With<RotateHandle>, With<Block>)>,
    mut query: Query<&mut RotateHandle>,
) {
    for entity in block_query.iter_mut().chain(sensor.entities()) {
        let mut handle = query.get_mut(entity).unwrap();
        handle.t += movement;
    }
}

pub(super) fn reset(
    key: Res<Input<KeyCode>>,
    cube: Res<CubeDescriptor>,
    ball_query: Query<Entity, With<BallColor>>,
    mut query: QuerySet<(
        Query<(&mut PathHandle, &mut SlideHandle, &mut RotateHandle)>,
        Query<&mut SlideHandle, With<Block>>,
        Query<&mut RotateHandle, With<Block>>,
    )>,
) {
    let is_shuffle = key.just_pressed(KeyCode::P);
    let is_reset   = key.just_pressed(KeyCode::L);
    if is_shuffle || is_reset {
        let mut balls = ball_query.iter().collect::<Vec<_>>();
        if is_shuffle {
            fastrand::shuffle(&mut balls);
        } else {
            balls.sort();
        }
        for (entity, init) in balls.into_iter().zip(cube.ball_init_handle_iter()) {
            let mut handle = query.q0_mut().get_mut(entity).unwrap();
            *handle.0 = init.path;
            *handle.1 = init.slide;
            *handle.2 = init.rotate;
        }

        for mut handle in query.q1_mut().iter_mut() {
            *handle = SlideHandle::left();
        }

        for mut handle in query.q2_mut().iter_mut() {
            *handle = RotateHandle::up();
        }
    }
}
