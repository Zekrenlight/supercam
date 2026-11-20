//! The free camera: a point with a heading that flies where you look while the player's body
//! stays where it was. No game types here, so the tests run without a client.

use std::sync::Mutex;

/// How far the camera keeps from walls when it doesn't pass through them, in blocks.
const HALF: f64 = 0.2;
/// Holding the sprint key multiplies the speed by this.
const FAST: f64 = 3.0;
/// With gliding on, each tick closes this much of the gap to the wanted velocity.
const GLIDE: f64 = 0.3;
/// Mouse counts to degrees, as the game turns the player.
const DEGREES_PER_COUNT: f64 = 0.15;

/// What is held this tick (the player's own movement keys, as bound in Controls).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Input {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub fast: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cam {
    pub at: [f64; 3],
    pub before: [f64; 3],
    pub velocity: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
    /// The view to go back to: 0 first person, 1 behind, 2 in front.
    pub view: u8,
}

impl Cam {
    pub fn new(at: [f64; 3], yaw: f32, pitch: f32, view: u8) -> Self {
        Self {
            at,
            before: at,
            velocity: [0.0; 3],
            yaw,
            pitch,
            view,
        }
    }

    pub fn turn(&mut self, xo: f64, yo: f64) {
        self.yaw = (f64::from(self.yaw) + xo * DEGREES_PER_COUNT) as f32;
        self.pitch = ((f64::from(self.pitch) + yo * DEGREES_PER_COUNT) as f32).clamp(-90.0, 90.0);
    }

    /// The velocity the keys ask for, in blocks a tick, from `speed` in blocks a second.
    pub fn wanted(&self, input: Input, speed: f64) -> [f64; 3] {
        let (yaw, pitch) = (
            f64::from(self.yaw).to_radians(),
            f64::from(self.pitch).to_radians(),
        );
        let look = [
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        ];
        let right = [-yaw.cos(), 0.0, -yaw.sin()];
        let axis = |a: bool, b: bool| f64::from(i8::from(a) - i8::from(b));
        let (f, r, u) = (
            axis(input.forward, input.back),
            axis(input.right, input.left),
            axis(input.up, input.down),
        );
        let mut v = [0.0; 3];
        for i in 0..3 {
            v[i] = look[i] * f + right[i] * r;
        }
        v[1] += u;
        let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if length < 1e-9 {
            return [0.0; 3];
        }
        let per_tick = speed / 20.0 * if input.fast { FAST } else { 1.0 };
        v.map(|c| c / length * per_tick)
    }

    /// One client tick. `blocked` says whether a box (lowest and highest corner) runs into a
    /// block; it is not asked when the camera passes through blocks.
    pub fn tick(
        &mut self,
        input: Input,
        speed: f64,
        glide: bool,
        through: bool,
        blocked: &dyn Fn([f64; 3], [f64; 3]) -> bool,
    ) {
        self.before = self.at;
        let want = self.wanted(input, speed);
        for i in 0..3 {
            self.velocity[i] = if glide {
                self.velocity[i] + (want[i] - self.velocity[i]) * GLIDE
            } else {
                want[i]
            };
            if self.velocity[i].abs() < 1e-5 {
                self.velocity[i] = 0.0;
            }
        }
        for i in 0..3 {
            let mut next = self.at;
            next[i] += self.velocity[i];
            if through || !blocked(next.map(|c| c - HALF), next.map(|c| c + HALF)) {
                self.at = next;
            } else {
                self.velocity[i] = 0.0;
            }
        }
    }

    /// Where the camera is drawn, `partial` of the way into the tick.
    pub fn at(&self, partial: f32) -> [f64; 3] {
        let t = f64::from(partial.clamp(0.0, 1.0));
        std::array::from_fn(|i| self.before[i] + (self.at[i] - self.before[i]) * t)
    }
}

/// Whether the box `[min, max]` overlaps any of `boxes` (`[x0, y0, z0, x1, y1, z1]`, in the
/// block's own corner `origin`).
pub fn overlaps(min: [f64; 3], max: [f64; 3], origin: [i32; 3], boxes: &[[f64; 6]]) -> bool {
    boxes.iter().any(|b| {
        (0..3).all(|i| {
            let o = f64::from(origin[i]);
            min[i] < o + b[i + 3] && max[i] > o + b[i]
        })
    })
}

static CAM: Mutex<Option<Cam>> = Mutex::new(None);

pub fn with<R>(f: impl FnOnce(&mut Option<Cam>) -> R) -> R {
    let mut guard = CAM.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut guard)
}

pub fn active() -> bool {
    with(|c| c.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: [f64; 3], b: [f64; 3]) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < 1e-6)
    }

    #[test]
    fn flies_where_it_looks() {
        let c = Cam::new([0.0; 3], 0.0, 0.0, 0);
        let fwd = Input {
            forward: true,
            ..Input::default()
        };
        assert!(near(c.wanted(fwd, 20.0), [0.0, 0.0, 1.0]));
        let down = Cam::new([0.0; 3], 0.0, 90.0, 0);
        assert!(near(down.wanted(fwd, 20.0), [0.0, -1.0, 0.0]));
        let right = Input {
            right: true,
            ..Input::default()
        };
        assert!(near(c.wanted(right, 20.0), [-1.0, 0.0, 0.0]));
        let fast_up = Input {
            up: true,
            fast: true,
            ..Input::default()
        };
        assert!(near(c.wanted(fast_up, 20.0), [0.0, FAST, 0.0]));
    }

    #[test]
    fn walls_stop_it_unless_it_passes_through() {
        let wall = |min: [f64; 3], max: [f64; 3]| overlaps(min, max, [1, -5, -5], &[[0.0, 0.0, 0.0, 1.0, 10.0, 10.0]]);
        let fwd_x = Input {
            left: true,
            ..Input::default()
        };
        let mut c = Cam::new([0.5, 0.5, 0.5], 0.0, 0.0, 0);
        for _ in 0..10 {
            c.tick(fwd_x, 20.0, false, false, &wall);
        }
        assert!(c.at[0] <= 0.8 + 1e-9);
        let mut c = Cam::new([0.5, 0.5, 0.5], 0.0, 0.0, 0);
        for _ in 0..10 {
            c.tick(fwd_x, 20.0, false, true, &wall);
        }
        assert!(c.at[0] > 5.0);
    }

    #[test]
    fn glides_to_speed_and_interpolates() {
        let none = |_: [f64; 3], _: [f64; 3]| false;
        let up = Input {
            up: true,
            ..Input::default()
        };
        let mut c = Cam::new([0.0; 3], 0.0, 0.0, 0);
        c.tick(up, 20.0, true, true, &none);
        assert!(c.at[1] > 0.0 && c.at[1] < 1.0);
        assert!(near(c.at(0.5), [0.0, c.at[1] / 2.0, 0.0]));
        c.turn(100.0, 1000.0);
        assert_eq!((c.yaw, c.pitch), (15.0, 90.0));
    }
}
