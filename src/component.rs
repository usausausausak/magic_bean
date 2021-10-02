use bevy::prelude::*;
use bevy::core::FloatOrd;

use crate::util;
use crate::util::range01::{WrappingF32, SaturatingF32};

mod descriptor;
pub use descriptor::CubeDescriptor;

#[allow(unused)]
pub mod debug {
    pub enum Debug { On, Off }
    pub enum DebugVisible { Yes, No }
}

pub struct MainCamera;

pub struct Cube;
pub enum Block { Slide, Rotate }
pub struct Deco;

#[derive(Default)]
pub struct CubeRotation {
    pub yaw: SaturatingF32,
    pub pitch: WrappingF32,
}

impl CubeRotation {
    pub fn new(yaw: f32, pitch: f32) -> Self {
        CubeRotation {
            yaw: SaturatingF32::new(yaw),
            pitch: WrappingF32::new(pitch),
        }
    }

    pub fn to_quat(&self) -> Quat {
        const ANGLE30: f32 = std::f32::consts::FRAC_PI_6;
        let yaw = util::bezier::f32::linear(-ANGLE30, ANGLE30, self.yaw.to_f32());

        const ANGLE360: f32 = std::f32::consts::PI * 2.0;
        let pitch = util::bezier::f32::linear(0.0, ANGLE360, self.pitch.to_f32());

        Quat::from_rotation_ypr(yaw, pitch, 0.0)
    }

    pub fn is_face_back(&self) -> bool {
        let pitch = self.pitch.to_f32();
        pitch >= 0.25 && pitch <= 0.75
    }
}

#[derive(Clone, Copy)]
pub enum BallColor { A, B, C, D }

impl BallColor {
    pub fn to_char(&self) -> char {
        use BallColor::*;
        match self {
            A => 'A',
            B => 'B',
            C => 'C',
            D => 'D',
        }
    }
}

#[derive(Bundle)]
pub struct BallHandleBundle {
    pub path: PathHandle,
    pub slide: SlideHandle,
    pub rotate: RotateHandle,
}

#[derive(Clone, Copy)]
pub struct PathHandle {
    pub t: WrappingF32,
}

impl PathHandle {
    pub fn new(t: f32) -> Self {
        PathHandle {
            t: WrappingF32::new(t),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SlideHandle {
    pub t: SaturatingF32,
}

impl SlideHandle {
    pub fn new(t: f32) -> Self {
        SlideHandle {
            t: SaturatingF32::new(t),
        }
    }

    pub fn left() -> Self {
        SlideHandle::new(0.0)
    }

    pub fn right() -> Self {
        SlideHandle::new(1.0)
    }
}

#[derive(Clone, Copy)]
pub struct RotateHandle {
    pub t: WrappingF32,
}

impl RotateHandle {
    pub fn new(t: f32) -> Self {
        RotateHandle {
            t: WrappingF32::new(t),
        }
    }

    pub fn up() -> Self {
        RotateHandle::new(0.0)
    }

    pub fn down() -> Self {
        RotateHandle::new(0.5)
    }
}

pub struct BallSensor {
    shape: shape::Box,
    capacity: usize,
    pub detail: Vec<(Entity, BallColor, FloatOrd)>,
}

impl std::fmt::Display for BallSensor {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.detail.iter().try_for_each(|(_, c, _)| write!(fmt, "{}", c.to_char()))
    }
}

impl BallSensor {
    pub fn new(shape: shape::Box, capacity: usize) -> Self {
        BallSensor {
            shape,
            capacity,
            detail: Vec::with_capacity(capacity),
        }
    }

    pub fn intersect_point(&self, point: Vec3) -> bool {
        let x = point.x >= self.shape.min_x && point.x <= self.shape.max_x;
        let y = point.y >= self.shape.min_y && point.y <= self.shape.max_y;
        let z = point.z >= self.shape.min_z && point.z <= self.shape.max_z;
        x && y && z
    }

    pub fn entities(&self) -> impl Iterator<Item=Entity> + '_ {
        self.detail.iter().map(|(e, _, _)| *e)
    }

    pub fn is_full(&self) -> bool {
        self.detail.len() == self.capacity
    }
}


#[derive(Clone, Copy)]
pub enum MovementKind {
    Path,
    Slide,
    Rotate,
}

pub struct Movement {
    pub grabbing: GrabbingSensor,
    pub movement: f32,
}

pub struct Animation {
    pub kind: MovementKind,
    pub movement: f32,
    pub base: f32,
}

#[derive(Clone, Copy)]
pub struct GrabbingSensor {
    pub kind: MovementKind,
    pub entity: Entity,
}

#[derive(Default)]
pub struct GrabStatus {
    pub grabbing: Option<GrabbingSensor>,
    pub origin: Vec3,
}

pub struct SnapEvent(pub GrabbingSensor);
