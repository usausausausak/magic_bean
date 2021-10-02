use bevy::prelude::*;

use crate::component::{BallHandleBundle, PathHandle, SlideHandle, RotateHandle};

use crate::util::{self, bezier::Path};

const N_BALLS: usize = 18;
const SLIDE_N_BALLS: usize = 4;
const ROTATE_N_BALLS: usize = 2;
const BALL_RADIANS: f32 = 0.3;
const BALL_STEP: f32 = 1.0 / N_BALLS as f32;

const FLOOR_Y: f32 = 0.3;

const PATH_OUTSIDE_ORIGIN_X: f32 = -1.067;
const PATH_OUTSIDE_BACKWARD_X: f32 = -1.067;
const PATH_OUTSIDE_FORWARD_X: f32 = -1.12986;
const PATH_SIDE_ORIGIN_Y: f32 = 0.9;
const PATH_SIDE_BACKWARD_Y: f32 = 0.3;
const PATH_SIDE_FORWARD_Y: f32 = 1.5567;
const PATH_TOP_ORIGIN_X: f32 = 0.0;
const PATH_TOP_BACKWARD_X: f32 = -0.68989;
const PATH_TOP_ORIGIN_Y: f32 = 2.1249;
const PATH_TOP_BACKWARD_Y: f32 = 2.0171;

const PATH_INSIDE_OX: f32 = GROUP_LEFT_OX - PATH_OUTSIDE_ORIGIN_X;

const GROUP_LEFT_OX: f32 = -1.5;
const GROUP_OY: f32 = 0.0;
const GROUP_UP_Z: f32 = FLOOR_Y + BALL_RADIANS;

pub struct CubeDescriptor {
    pub left_path: BallPath,
    pub right_path: BallPath,
    pub slide_path: SlicePath,
    pub rotate_path: RotatePath,
}

impl Default for CubeDescriptor {
    fn default() -> Self {
        CubeDescriptor {
            left_path: BallPath::left(),
            right_path: BallPath::right(),
            slide_path: Default::default(),
            rotate_path: Default::default(),
        }
    }
}

impl CubeDescriptor {
    pub fn group_origin_iter(&self) -> impl Iterator<Item=(&str, Vec3)> {
        std::array::IntoIter::new([
            ("group.a", Vec3::new( GROUP_LEFT_OX, GROUP_OY,  GROUP_UP_Z)),
            ("group.b", Vec3::new(-GROUP_LEFT_OX, GROUP_OY,  GROUP_UP_Z)),
            ("group.c", Vec3::new( GROUP_LEFT_OX, GROUP_OY, -GROUP_UP_Z)),
            ("group.d", Vec3::new(-GROUP_LEFT_OX, GROUP_OY, -GROUP_UP_Z)),
        ])
    }

    pub fn group_capacity(&self) -> usize {
        N_BALLS
    }

    pub fn slide_capacity(&self) -> usize {
        SLIDE_N_BALLS * 2
    }

    pub fn rotate_capacity(&self) -> usize {
        ROTATE_N_BALLS * 2
    }

    pub fn balls_per_color(&self) -> usize {
        N_BALLS - 2
    }

    pub fn ball_radians(&self) -> f32 {
        BALL_RADIANS
    }

    pub fn ball_step(&self) -> f32 {
        BALL_STEP
    }

    pub fn ball_init_handle_iter(&self) -> impl Iterator<Item=BallHandleBundle> {
        const SKIP_N_BALLS: usize = SLIDE_N_BALLS;
        let step = self.ball_step();
        let groups = [
            (SlideHandle::left() , RotateHandle::up()  , 0..N_BALLS),
            (SlideHandle::right(), RotateHandle::up()  , SKIP_N_BALLS..N_BALLS),
            (SlideHandle::left() , RotateHandle::down(), 0..N_BALLS),
            (SlideHandle::right(), RotateHandle::down(), SKIP_N_BALLS..N_BALLS),
        ];
        std::array::IntoIter::new(groups)
            .map(move |(slide, rotate, range)| {
                range.map(move |i| {
                    let path = PathHandle::new(i as f32 * step);
                    BallHandleBundle { path, slide, rotate }
                })
            })
            .flatten()
    }

    pub fn get_ball_transform(&self, handle: &BallHandleBundle) -> Transform {
        let (path, outside_v) = if handle.slide.t.to_f32() < 0.5 {
            self.left_path()
        } else {
            self.right_path()
        };
        let path_v = path.evaluate(handle.path);
        let slide_v = self.slide_path.evaluate(handle.slide);
        let rotate_q = self.rotate_path.evaluate(handle.rotate);
        let up_v = rotate_q * self.ball_up_translation();

        let v = path_v + slide_v + outside_v + up_v;
        Transform::from_translation(v)
    }

    fn left_path(&self) -> (&BallPath, Vec3) {
        (&self.left_path, Vec3::new(PATH_OUTSIDE_ORIGIN_X, 0.0, 0.0))
    }

    fn right_path(&self) -> (&BallPath, Vec3) {
        (&self.right_path, Vec3::new(-PATH_OUTSIDE_ORIGIN_X, 0.0, 0.0))
    }

    fn ball_up_translation(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, GROUP_UP_Z)
    }

    pub fn path_sign(&self, o: Vec3, cp: Vec3, v: Vec2) -> f32 {
        let sy = if cp.y > 0.0 { v.x } else { -v.x };
        let (sx, sy) = if o.x > 0.0 {
            // right path
            if cp.x < -GROUP_LEFT_OX {
                (-v.y, sy)
            } else {
                (v.y, sy)
            }
        } else {
            // left path
            if cp.x < GROUP_LEFT_OX {
                (v.y, -sy)
            } else {
                (-v.y, -sy)
            }
        };

        if v.y.abs() > v.x.abs() { sx } else { sy }
    }

    pub fn path_snap_first(&self, t: f32) -> f32 {
        const HSTEP: f32 = BALL_STEP / 2.0;
        if t > HSTEP {
            BALL_STEP
        } else {
            0.0
        }
    }

    pub fn ball_sensor_box(&self) -> shape::Box {
        const HX: f32 = -PATH_OUTSIDE_ORIGIN_X + BALL_RADIANS;
        const HY: f32 = PATH_TOP_ORIGIN_Y + BALL_RADIANS;
        const HZ: f32 = BALL_RADIANS / 2.0 + 0.1;
        shape::Box::new(HX * 2.0, HY * 2.0, HZ * 2.0)
    }

    pub fn slide_sensor_box(&self) -> shape::Box {
        const HX: f32 = -PATH_INSIDE_OX + BALL_RADIANS;
        const HY: f32 = BALL_RADIANS * SLIDE_N_BALLS as f32;
        const HZ: f32 = GROUP_UP_Z + BALL_RADIANS + 0.05;
        shape::Box::new(HX * 2.0, HY * 2.0, HZ * 2.0)
    }

    pub fn rotate_sensor_box(&self) -> shape::Box {
        const HX: f32 = -GROUP_LEFT_OX;
        const HY: f32 = BALL_RADIANS * ROTATE_N_BALLS as f32;
        const HZ: f32 = GROUP_UP_Z + BALL_RADIANS + 0.1;
        shape::Box::new(HX * 2.0, HY * 2.0, HZ * 2.0)
    }
}

pub struct SlicePath {
    p1: f32,
    p2: f32,
}

impl Default for SlicePath {
    fn default() -> Self {
        SlicePath {
            p1: PATH_INSIDE_OX,
            p2: -PATH_INSIDE_OX,
        }
    }
}

impl SlicePath {
    pub fn evaluate(&self, h: SlideHandle) -> Vec3 {
        let x = util::bezier::f32::linear(self.p1, self.p2, h.t.to_f32());
        Vec3::new(x, 0.0, 0.0)
    }
}

pub struct RotatePath {
    p1: f32,
    p2: f32,
}

impl Default for RotatePath {
    fn default() -> Self {
        RotatePath {
            p1: 0.0,
            p2: std::f32::consts::PI * 2.0,
        }
    }
}

impl RotatePath {
    pub fn evaluate(&self, h: RotateHandle) -> Quat {
        let rotate = util::bezier::f32::linear(self.p1, self.p2, h.t.to_f32());
        Quat::from_rotation_y(rotate)
    }
}

pub struct BallPath(Path<6>);

impl BallPath {
    fn left() -> Self {
        let mut p = BallPath::right();
        p.0.reverse();
        let [cp6, cp5, cp4, cp3, cp2, cp1] = p.0.cp;
        BallPath( Path {
            cp: [
                cp5,
                cp4,
                cp3,
                cp2,
                cp1,
                cp6,
            ]
        })
    }

    fn right() -> Self {
        let cp1 = util::bezier::ControlPoint {
            backward: Vec3::new(  PATH_OUTSIDE_FORWARD_X,  -PATH_SIDE_FORWARD_Y, 0.0),
            origin:   Vec3::new(   PATH_OUTSIDE_ORIGIN_X,   -PATH_SIDE_ORIGIN_Y, 0.0),
            forward:  Vec3::new( PATH_OUTSIDE_BACKWARD_X, -PATH_SIDE_BACKWARD_Y, 0.0),
        };
        let cp2 = util::bezier::ControlPoint {
            backward: Vec3::new( PATH_OUTSIDE_BACKWARD_X,  PATH_SIDE_BACKWARD_Y, 0.0),
            origin:   Vec3::new(   PATH_OUTSIDE_ORIGIN_X,    PATH_SIDE_ORIGIN_Y, 0.0),
            forward:  Vec3::new(  PATH_OUTSIDE_FORWARD_X,   PATH_SIDE_FORWARD_Y, 0.0),
        };
        let cp3 = util::bezier::ControlPoint {
            backward: Vec3::new(     PATH_TOP_BACKWARD_X,   PATH_TOP_BACKWARD_Y, 0.0),
            origin:   Vec3::new(              PATH_TOP_ORIGIN_X,            PATH_TOP_ORIGIN_Y, 0.0),
            forward:  Vec3::new(    -PATH_TOP_BACKWARD_X,   PATH_TOP_BACKWARD_Y, 0.0),
        };
        let cp4 = util::bezier::ControlPoint {
            backward: Vec3::new( -PATH_OUTSIDE_FORWARD_X,   PATH_SIDE_FORWARD_Y, 0.0),
            origin:   Vec3::new(  -PATH_OUTSIDE_ORIGIN_X,    PATH_SIDE_ORIGIN_Y, 0.0),
            forward:  Vec3::new(-PATH_OUTSIDE_BACKWARD_X,  PATH_SIDE_BACKWARD_Y, 0.0),
        };
        let cp5 = util::bezier::ControlPoint {
            backward: Vec3::new(-PATH_OUTSIDE_BACKWARD_X, -PATH_SIDE_BACKWARD_Y, 0.0),
            origin:   Vec3::new(  -PATH_OUTSIDE_ORIGIN_X,   -PATH_SIDE_ORIGIN_Y, 0.0),
            forward:  Vec3::new( -PATH_OUTSIDE_FORWARD_X,  -PATH_SIDE_FORWARD_Y, 0.0),
        };
        let cp6 = util::bezier::ControlPoint {
            backward: Vec3::new(    -PATH_TOP_BACKWARD_X,  -PATH_TOP_BACKWARD_Y, 0.0),
            origin:   Vec3::new(              PATH_TOP_ORIGIN_X,           -PATH_TOP_ORIGIN_Y, 0.0),
            forward:  Vec3::new(     PATH_TOP_BACKWARD_X,  -PATH_TOP_BACKWARD_Y, 0.0),
        };

        BallPath(Path {
            cp: [
                cp1,
                cp2,
                cp3,
                cp4,
                cp5,
                cp6,
            ]
        })
    }

    pub fn evaluate(&self, h: PathHandle) -> Vec3 {
        self.0.evaluate(h.t.to_f32())
    }
}
