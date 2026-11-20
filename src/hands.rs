//! Breathing hands: the first-person hand and what it holds rise and settle slowly while you
//! stand idle, and fade back to still as soon as you move, swing or use something.

use crate::config::Config;
use std::f32::consts::TAU;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Breath {
    /// Where in the breath we are, 0 to 1.
    phase: f32,
    /// How much of the breathing shows, 0 to 1.
    shown: f32,
    /// Seconds since the hands were last busy (moving, swinging, using).
    calm: f32,
}

/// Seconds of calm before the breathing starts to show.
const SETTLE: f32 = 0.6;

impl Breath {
    /// One frame. `busy` is true while moving, swinging or using an item.
    pub fn frame(&mut self, seconds: f32, busy: bool, c: &Config) {
        let dt = seconds.clamp(0.0, 0.25);
        // Speed 1 is a breath every eight seconds or so, speed 10 one a second.
        let hz = 0.1 + c.breathing_speed.clamp(1.0, 10.0) * 0.09;
        self.phase = (self.phase + dt * hz).fract();
        self.calm = if busy { 0.0 } else { self.calm + dt };
        let want = c.breathing && self.calm > SETTLE;
        self.shown = if want {
            (self.shown + dt / 1.5).min(1.0)
        } else {
            (self.shown - dt * 4.0).max(0.0)
        };
    }

    /// The hand's offset this frame: up and down in blocks, and a slight nod in degrees.
    pub fn offset(&self, c: &Config) -> ([f32; 3], f32) {
        if self.shown <= 0.0 {
            return ([0.0; 3], 0.0);
        }
        let ease = self.shown * self.shown * (3.0 - 2.0 * self.shown);
        let amp = ease * c.breathing_amt;
        // In slowly, out a little quicker: a breath, not a metronome.
        let wave = (self.phase * TAU).sin() + 0.25 * (self.phase * 2.0 * TAU).sin();
        let lift = 0.012 * amp * wave;
        ([0.0, lift, lift * -0.3], 0.6 * amp * wave)
    }
}

static BREATH: Mutex<Option<Breath>> = Mutex::new(None);

pub fn with<R>(f: impl FnOnce(&mut Breath) -> R) -> R {
    let mut guard = BREATH.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(Breath::default))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_by_default_and_still() {
        let c = Config::default();
        let mut b = Breath::default();
        for _ in 0..200 {
            b.frame(0.05, false, &c);
        }
        assert_eq!(b.offset(&c), ([0.0; 3], 0.0));
    }

    #[test]
    fn breathes_when_idle_and_fades_when_busy() {
        let c = Config {
            breathing: true,
            ..Config::default()
        };
        let mut b = Breath::default();
        let mut most = 0.0f32;
        for _ in 0..400 {
            b.frame(0.05, false, &c);
            most = most.max(b.offset(&c).0[1].abs());
        }
        assert!(most > 0.008, "{most}");
        for _ in 0..10 {
            b.frame(0.05, true, &c);
        }
        assert_eq!(b.offset(&c), ([0.0; 3], 0.0));
    }
}
