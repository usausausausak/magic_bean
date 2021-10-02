#![allow(unused)]
use bevy::prelude::*;

macro_rules! otry {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None    => return,
        }
    };
    ($e:expr, $ret:expr) => {
        match $e {
            Some(v) => v,
            None    => return $ret,
        }
    };
}
pub(crate) use otry;

#[allow(unused)]
pub mod bezier {
    use bevy::prelude::Vec3;

    pub mod f32 {
        pub fn linear(p1: f32, p2: f32, t: f32) -> f32 {
            debug_assert!(0.0 <= t && t <= 1.0);
            (1.0 - t) * p1 + t * p2
        }

        pub fn quadratic(p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
            debug_assert!(0.0 <= t && t <= 1.0);
            let c = 1.0 - t;
            c * c * p1 + 2.0 * c * t * p2 + t * t * p3
        }

        pub fn cubic(p1: f32, p2: f32, p3: f32, p4: f32, t: f32) -> f32 {
            debug_assert!(0.0 <= t && t <= 1.0);
            let c = 1.0 - t;
            let c2 = c * c;
            let t2 = t * t;
            c2 * c * p1 + 3.0 * c2 * t * p2 + 3.0 * c * t2 * p3 + t2 * t * p4
        }
    }

    pub fn cubic(from: &ControlPoint, to: &ControlPoint, t: f32) -> Vec3 {
        Vec3::new(
            f32::cubic(from.origin.x, from.forward.x, to.backward.x, to.origin.x, t),
            f32::cubic(from.origin.y, from.forward.y, to.backward.y, to.origin.y, t),
            f32::cubic(from.origin.z, from.forward.z, to.backward.z, to.origin.z, t),
        )
    }

    pub struct ControlPoint {
        pub backward: Vec3,
        pub origin: Vec3,
        pub forward: Vec3,
    }

    pub struct Path<const N: usize> {
        pub cp: [ControlPoint; N],
    }

    impl <const N: usize> Path<N> {
        pub const N_POINTS: usize = N;

        pub fn evaluate(&self, t: f32) -> Vec3 {
            debug_assert!(t >= 0.0 && t <= 1.0);

            let m = t * Self::N_POINTS as f32;
            let i = m.trunc() as usize % Self::N_POINTS;
            let it = m.fract();

            if i >= Self::N_POINTS - 1 {
                cubic(&self.cp[i], &self.cp[0], it)
            } else {
                cubic(&self.cp[i], &self.cp[i + 1], it)
            }
        }

        pub fn reverse(&mut self) {
            self.cp.reverse();
            for cp in self.cp.iter_mut() {
                std::mem::swap(&mut cp.backward, &mut cp.forward);
            }
        }
    }
}

pub mod range01 {
    macro_rules! handle_type {
        ($t:ident, $m:ident) => {
            #[derive(Clone, Copy, Default)]
            pub struct $t {
                t: f32,
            }

            impl $t {
                pub fn new(t: f32) -> Self {
                    Self { t: $m(t) }
                }

                pub fn to_f32(&self) -> f32 {
                    self.t
                }
            }

            impl std::ops::AddAssign<f32> for $t {
                fn add_assign(&mut self, rhs: f32) {
                    self.t = $m(self.t + rhs);
                }
            }
        }
    }

    handle_type!(WrappingF32, wrapping_new);
    handle_type!(SaturatingF32, saturating_new);

    pub fn saturating_new(t: f32) -> f32 {
        t.clamp(0.0, 1.0)
    }

    pub fn wrapping_new(t: f32) -> f32 {
        let t = t % 1.0;
        if t < 0.0 {
            saturating_new(1.0 + t)
        } else {
            t.clamp(0.0, 1.0)
        }
    }
}
