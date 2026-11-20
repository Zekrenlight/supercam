//! Camera motion: the view leans into strafes and turns, dips as you speed up, rises and sinks
//! with jumps and falls, sways when you stand still for a while, and shakes when something
//! blows up nearby or lightning strikes. Read once a tick from the player, eased every frame,
//! and handed to the game as a camera shake.
//!
//! Signs follow the game's camera: a positive pitch looks down, a positive roll leans right.

use crate::config::Config;
use std::sync::Mutex;

/// What the player is doing, for the per-situation strengths.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Situation {
    #[default]
    Walking,
    /// Creative flight or an elytra.
    Flying,
    /// On a mount, in a boat or a minecart.
    Riding,
}

/// One tick's reading of the player.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reading {
    /// Movement this tick in blocks: x, y, z.
    pub moved: [f64; 3],
    /// Facing, degrees.
    pub yaw: f32,
    /// How far the view turned this tick, degrees (positive turns right).
    pub turned: f32,
    pub situation: Situation,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Angles {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Motion {
    /// Where the lean and dip want to be (from the last tick).
    target: Angles,
    /// Where they are, eased toward the target every frame.
    eased: Angles,
    /// The situation strength, eased so getting on a horse doesn't jolt.
    strength: f32,
    /// Seconds without moving or turning.
    still: f32,
    /// How much of the sway shows, 0 to 1.
    sway_in: f32,
    /// A running clock for the sway and the shake, seconds.
    clock: f32,
    /// Shake energy, 0 to 1; it dies away by itself.
    trauma: f32,
    /// The last shake handed out.
    pub out: Angles,
    pub out_offset: [f32; 3],
}

const MAX_LEAN: f32 = 8.0;

fn situation_strength(c: &Config, s: Situation) -> f32 {
    match s {
        Situation::Walking => c.walking,
        Situation::Flying => c.flying,
        Situation::Riding => c.riding,
    }
}

fn wrap(degrees: f32) -> f32 {
    (degrees + 180.0).rem_euclid(360.0) - 180.0
}

impl Motion {
    /// A tick's reading: new targets for the lean and dip, and the stillness clock.
    pub fn read(&mut self, r: Reading, c: &Config) {
        let yaw = r.yaw.to_radians();
        let (fx, fz) = (-yaw.sin(), yaw.cos());
        let (rx, rz) = (-yaw.cos(), -yaw.sin());
        let (mx, my, mz) = (r.moved[0] as f32, r.moved[1] as f32, r.moved[2] as f32);
        let ahead = mx * fx + mz * fz;
        let aside = mx * rx + mz * rz;
        let turned = wrap(r.turned);

        let mut t = Angles::default();
        if c.strafe_roll {
            // A sprinting side-step (about 0.28 blocks a tick) leans about three degrees.
            t.roll += aside * 11.0 * c.strafe_roll_amt;
        }
        if c.turn_roll {
            // Banks into turns, a little for a flick and more for a long sweep.
            t.roll += (turned * 0.18).clamp(-5.0, 5.0) * c.turn_roll_amt;
        }
        if c.move_pitch {
            t.pitch += ahead * 7.0 * c.move_pitch_amt;
        }
        if c.air_pitch {
            // Up on a jump, down into a fall.
            t.pitch += (-my * 6.0).clamp(-4.0, 6.0) * c.air_pitch_amt;
        }
        t.pitch = t.pitch.clamp(-MAX_LEAN, MAX_LEAN);
        t.roll = t.roll.clamp(-MAX_LEAN, MAX_LEAN);
        self.target = t;

        let moving = ahead.abs() + aside.abs() + my.abs() > 0.003 || turned.abs() > 0.05;
        if moving {
            self.still = 0.0;
        }
        // The per-situation strength.
        let goal = situation_strength(c, r.situation);
        self.strength += (goal - self.strength) * 0.15;
    }

    /// Something went off `distance` blocks away; `power` 1 is a creeper or TNT.
    pub fn blast(&mut self, distance: f64, power: f32) {
        let reach = 28.0;
        if !(0.0..reach).contains(&distance) {
            return;
        }
        let near = 1.0 - (distance / reach) as f32;
        self.trauma = (self.trauma + power * near * near.sqrt()).min(1.0);
    }

    /// Thunder far away gives a light rumble.
    pub fn rumble(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(0.4).max(self.trauma);
    }

    /// One frame, `seconds` long: eases everything and works out this frame's shake.
    pub fn frame(&mut self, seconds: f32, c: &Config) {
        let dt = seconds.clamp(0.0, 0.25);
        self.clock = (self.clock + dt) % 3600.0;
        self.still += dt;

        // Smoothness 1 settles in about a tenth of a second, 10 in about a second.
        let rate = 26.0 - c.motion_smooth.clamp(1.0, 10.0) * 2.3;
        let k = 1.0 - (-rate * dt).exp();
        self.eased.pitch += (self.target.pitch - self.eased.pitch) * k;
        self.eased.roll += (self.target.roll - self.eased.roll) * k;

        let mut out = Angles {
            pitch: self.eased.pitch * self.strength,
            yaw: 0.0,
            roll: self.eased.roll * self.strength,
        };

        // Sway: fades in over two seconds after the delay, out in under half a second.
        let swaying = c.sway && self.still > c.sway_delay;
        self.sway_in = if swaying {
            (self.sway_in + dt / 2.0).min(1.0)
        } else {
            (self.sway_in - dt * 2.5).max(0.0)
        };
        if self.sway_in > 0.0 {
            let s = self.sway_in * self.sway_in * (3.0 - 2.0 * self.sway_in) * c.sway_amt;
            let t = self.clock;
            out.pitch += s * 0.55 * ((t * 0.71).sin() + 0.4 * (t * 0.29 + 2.1).sin());
            out.yaw += s * 0.45 * ((t * 0.53 + 1.0).sin() + 0.5 * (t * 0.19).sin());
            out.roll += s * 0.35 * (t * 0.43 + 0.6).sin();
        }

        // Shake: strong at first, fading over about a second; the square makes small ones soft.
        let mut offset = [0.0; 3];
        if self.trauma > 0.0 {
            let amp = self.trauma * self.trauma;
            let t = self.clock * 23.0;
            let n = |a: f32, b: f32| (t * a + b).sin() * 0.6 + (t * a * 1.7 + b * 3.0).sin() * 0.4;
            let degrees = 4.5 * amp * c.blast_amt;
            out.pitch += degrees * n(1.0, 0.3);
            out.yaw += degrees * n(0.83, 1.9);
            out.roll += degrees * 0.7 * n(1.21, 4.2);
            let blocks = 0.06 * amp * c.blast_amt;
            offset = [blocks * n(0.91, 2.5), blocks * n(1.13, 0.7), 0.0];
            self.trauma = (self.trauma - dt * 0.9).max(0.0);
        }

        self.out = out;
        self.out_offset = offset;
    }

    /// Straight back to level, for the free camera and leaving a world.
    pub fn settle(&mut self) {
        *self = Motion {
            strength: self.strength,
            ..Motion::default()
        };
    }
}

static MOTION: Mutex<Option<Motion>> = Mutex::new(None);

pub fn with<R>(f: impl FnOnce(&mut Motion) -> R) -> R {
    let mut guard = MOTION.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(Motion::default))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(m: &mut Motion, r: Reading, c: &Config, frames: usize) {
        for _ in 0..frames {
            m.read(r, c);
            m.frame(0.05, c);
        }
    }

    #[test]
    fn leans_into_a_strafe_and_a_turn() {
        let c = Config::default();
        let mut m = Motion::default();
        // Facing south (+z), moving west (-x) is moving right.
        run(&mut m, Reading { moved: [-0.2, 0.0, 0.0], ..Reading::default() }, &c, 60);
        assert!(m.out.roll > 1.0, "{:?}", m.out);
        let mut m = Motion::default();
        run(&mut m, Reading { turned: -20.0, ..Reading::default() }, &c, 60);
        assert!(m.out.roll < -1.0, "{:?}", m.out);
    }

    #[test]
    fn dips_ahead_and_follows_jumps_and_falls() {
        let c = Config::default();
        let mut m = Motion::default();
        run(&mut m, Reading { moved: [0.0, 0.0, 0.25], ..Reading::default() }, &c, 60);
        assert!(m.out.pitch > 1.0);
        let mut m = Motion::default();
        run(&mut m, Reading { moved: [0.0, -0.8, 0.0], ..Reading::default() }, &c, 60);
        assert!(m.out.pitch > 3.0);
        let mut m = Motion::default();
        run(&mut m, Reading { moved: [0.0, 0.4, 0.0], ..Reading::default() }, &c, 60);
        assert!(m.out.pitch < -1.0);
    }

    #[test]
    fn switches_off_and_scales_by_situation() {
        let off = Config { strafe_roll: false, ..Config::default() };
        let mut m = Motion::default();
        run(&mut m, Reading { moved: [-0.2, 0.0, 0.0], ..Reading::default() }, &off, 60);
        assert!(m.out.roll.abs() < 1e-3);
        let c = Config { riding: 0.0, ..Config::default() };
        let mut m = Motion::default();
        let ride = Reading { moved: [-0.2, 0.0, 0.0], situation: Situation::Riding, ..Reading::default() };
        run(&mut m, ride, &c, 120);
        assert!(m.out.roll.abs() < 0.05);
    }

    #[test]
    fn sways_only_after_standing_still() {
        let c = Config { sway_delay: 1.0, ..Config::default() };
        let mut m = Motion::default();
        run(&mut m, Reading::default(), &c, 10);
        assert_eq!(m.sway_in, 0.0);
        run(&mut m, Reading::default(), &c, 80);
        assert!(m.sway_in > 0.9);
        run(&mut m, Reading { moved: [0.0, 0.0, 0.2], ..Reading::default() }, &c, 10);
        assert_eq!(m.sway_in, 0.0);
    }

    #[test]
    fn blasts_shake_then_settle() {
        let c = Config::default();
        let mut m = Motion::default();
        m.blast(100.0, 1.0);
        assert_eq!(m.trauma, 0.0);
        m.blast(3.0, 1.0);
        assert!(m.trauma > 0.5);
        m.frame(0.016, &c);
        assert!(m.out.pitch != 0.0 || m.out.yaw != 0.0);
        for _ in 0..100 {
            m.frame(0.05, &c);
        }
        assert_eq!(m.trauma, 0.0);
    }
}
