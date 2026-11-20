//! The zoom: how close it is right now, easing toward where the key and the wheel want it.
//!
//! The ease runs on the logarithm of the magnification, so going from 1x to 2x takes as long as
//! going from 4x to 8x and the zoom feels even all the way in.

use crate::config::{Config, MAX_ZOOM, MIN_ZOOM};
use std::sync::Mutex;

/// One wheel notch multiplies (or divides) the zoom by this.
const NOTCH: f32 = 1.25;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zoom {
    /// The key is held.
    pub held: bool,
    /// What the wheel has made of the zoom amount this time (or kept from last time).
    pub level: Option<f32>,
    /// The magnification now: 1 is none.
    pub now: f32,
}

impl Default for Zoom {
    fn default() -> Self {
        Self {
            held: false,
            level: None,
            now: 1.0,
        }
    }
}

impl Zoom {
    /// Where the zoom is heading.
    pub fn goal(&self, c: &Config) -> f32 {
        if !self.held || !c.zoom {
            return 1.0;
        }
        self.level
            .unwrap_or(c.zoom_amount)
            .clamp(MIN_ZOOM, MAX_ZOOM)
    }

    /// The key went down or up.
    pub fn hold(&mut self, held: bool, c: &Config) {
        if held && !self.held && !c.zoom_keep_scroll {
            self.level = None;
        }
        self.held = held;
    }

    /// Moves the zoom on by `seconds`.
    pub fn step(&mut self, seconds: f32, c: &Config) {
        let goal = self.goal(c);
        if !c.zoom_smooth {
            self.now = goal;
            return;
        }
        // Speed 1 takes about a second to settle, speed 10 about a tenth of one.
        let rate = 1.5 + c.zoom_speed.clamp(1.0, 10.0) * 2.5;
        let keep = (-rate * seconds.clamp(0.0, 0.25)).exp();
        let (from, to) = (self.now.max(1.0).ln(), goal.ln());
        let next = to + (from - to) * keep;
        self.now = if (next - to).abs() < 0.002 { goal } else { next.exp() };
    }

    /// Wheel notches while zoomed: up goes closer. True when the wheel was used.
    pub fn scroll(&mut self, notches: f32, c: &Config) -> bool {
        if !self.held || !c.zoom || !c.zoom_scroll || notches == 0.0 {
            return false;
        }
        let level = self.level.unwrap_or(c.zoom_amount) * NOTCH.powf(notches);
        self.level = Some(level.clamp(MIN_ZOOM, MAX_ZOOM));
        true
    }

    /// The mouse's turning while zoomed: slower as the view gets closer.
    pub fn turn(&self, c: &Config) -> f32 {
        if c.zoom_slow_mouse {
            1.0 / self.now.max(1.0)
        } else {
            1.0
        }
    }

    pub fn zoomed(&self) -> bool {
        self.now > 1.01
    }
}

static ZOOM: Mutex<Option<Zoom>> = Mutex::new(None);

pub fn with<R>(f: impl FnOnce(&mut Zoom) -> R) -> R {
    let mut guard = ZOOM.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(Zoom::default))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eases_in_and_out() {
        let c = Config::default();
        let mut z = Zoom::default();
        z.hold(true, &c);
        z.step(0.05, &c);
        assert!(z.now > 1.0 && z.now < c.zoom_amount);
        for _ in 0..100 {
            z.step(0.05, &c);
        }
        assert_eq!(z.now, c.zoom_amount);
        z.hold(false, &c);
        for _ in 0..100 {
            z.step(0.05, &c);
        }
        assert_eq!(z.now, 1.0);
    }

    #[test]
    fn instant_without_smoothing() {
        let c = Config {
            zoom_smooth: false,
            ..Config::default()
        };
        let mut z = Zoom::default();
        z.hold(true, &c);
        z.step(0.0, &c);
        assert_eq!(z.now, c.zoom_amount);
    }

    #[test]
    fn wheel_moves_the_level_within_bounds_and_resets() {
        let c = Config::default();
        let mut z = Zoom::default();
        assert!(!z.scroll(1.0, &c));
        z.hold(true, &c);
        assert!(z.scroll(1.0, &c));
        assert_eq!(z.goal(&c), c.zoom_amount * NOTCH);
        z.scroll(100.0, &c);
        assert_eq!(z.goal(&c), MAX_ZOOM);
        z.scroll(-100.0, &c);
        assert_eq!(z.goal(&c), MIN_ZOOM);
        z.hold(false, &c);
        z.hold(true, &c);
        assert_eq!(z.goal(&c), c.zoom_amount);
    }

    #[test]
    fn slower_mouse_follows_the_zoom() {
        let c = Config::default();
        let z = Zoom {
            held: true,
            level: None,
            now: 4.0,
        };
        assert_eq!(z.turn(&c), 0.25);
    }
}
