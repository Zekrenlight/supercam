//! The settings: their keys, defaults and the copy every part reads.

use std::sync::Mutex;

pub const ID: &str = "supercam";

/// The zoom key's id (Options > Controls) and its default: hold to zoom.
pub const ZOOM_KEY: &str = "zoom";
pub const ZOOM_KEY_DEFAULT: &str = "key.keyboard.c";
/// The free camera key's id and its default: press to send the camera out, again to bring it
/// back. F7 is free in the game and in the other mods of the set.
pub const FREECAM_KEY: &str = "freecam";
pub const FREECAM_KEY_DEFAULT: &str = "key.keyboard.f7";

// Zoom
pub const KEY_ZOOM: &str = "zoom";
pub const KEY_ZOOM_AMOUNT: &str = "zoom_amount";
pub const KEY_ZOOM_SCROLL: &str = "zoom_scroll";
pub const KEY_ZOOM_KEEP_SCROLL: &str = "zoom_keep_scroll";
pub const KEY_ZOOM_SMOOTH: &str = "zoom_smooth";
pub const KEY_ZOOM_SPEED: &str = "zoom_speed";
pub const KEY_ZOOM_SLOW_MOUSE: &str = "zoom_slow_mouse";
pub const KEY_ZOOM_HIDE_HAND: &str = "zoom_hide_hand";
// Free camera
pub const KEY_FREECAM: &str = "freecam";
pub const KEY_FREECAM_SPEED: &str = "freecam_speed";
pub const KEY_FREECAM_GLIDE: &str = "freecam_glide";
pub const KEY_FREECAM_THROUGH: &str = "freecam_through_blocks";
pub const KEY_FREECAM_SHOW_PLAYER: &str = "freecam_show_player";
// Camera motion
pub const KEY_MOTION: &str = "motion";
pub const KEY_MOTION_THIRD: &str = "motion_third_person";
pub const KEY_MOTION_SMOOTH: &str = "motion_smoothness";
pub const KEY_STRAFE_ROLL: &str = "strafe_roll";
pub const KEY_STRAFE_ROLL_AMT: &str = "strafe_roll_strength";
pub const KEY_TURN_ROLL: &str = "turn_roll";
pub const KEY_TURN_ROLL_AMT: &str = "turn_roll_strength";
pub const KEY_MOVE_PITCH: &str = "move_pitch";
pub const KEY_MOVE_PITCH_AMT: &str = "move_pitch_strength";
pub const KEY_AIR_PITCH: &str = "air_pitch";
pub const KEY_AIR_PITCH_AMT: &str = "air_pitch_strength";
pub const KEY_SWAY: &str = "idle_sway";
pub const KEY_SWAY_AMT: &str = "idle_sway_strength";
pub const KEY_SWAY_DELAY: &str = "idle_sway_delay";
pub const KEY_BLAST: &str = "explosion_shake";
pub const KEY_BLAST_AMT: &str = "explosion_shake_strength";
pub const KEY_THUNDER: &str = "thunder_shake";
pub const KEY_WALKING: &str = "walking_strength";
pub const KEY_FLYING: &str = "flying_strength";
pub const KEY_RIDING: &str = "riding_strength";
// View
pub const KEY_FOV: &str = "fov";
// Breathing hands
pub const KEY_BREATHING: &str = "breathing_hands";
pub const KEY_BREATHING_AMT: &str = "breathing_strength";
pub const KEY_BREATHING_SPEED: &str = "breathing_speed";

/// Every setting, as the parts read them. Strengths are fractions (1.0 is the 100% default).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub zoom: bool,
    /// How many times closer the zoom brings things, before scrolling.
    pub zoom_amount: f32,
    pub zoom_scroll: bool,
    pub zoom_keep_scroll: bool,
    pub zoom_smooth: bool,
    /// 1 (slow) to 10 (quick).
    pub zoom_speed: f32,
    pub zoom_slow_mouse: bool,
    pub zoom_hide_hand: bool,

    pub freecam: bool,
    /// Blocks a second.
    pub freecam_speed: f32,
    pub freecam_glide: bool,
    pub freecam_through: bool,
    pub freecam_show_player: bool,

    pub motion: bool,
    pub motion_third: bool,
    /// 1 (snappy) to 10 (floaty).
    pub motion_smooth: f32,
    pub strafe_roll: bool,
    pub strafe_roll_amt: f32,
    pub turn_roll: bool,
    pub turn_roll_amt: f32,
    pub move_pitch: bool,
    pub move_pitch_amt: f32,
    pub air_pitch: bool,
    pub air_pitch_amt: f32,
    pub sway: bool,
    pub sway_amt: f32,
    /// Seconds of standing still before the sway starts.
    pub sway_delay: f32,
    pub blast: bool,
    pub blast_amt: f32,
    pub thunder: bool,
    pub walking: f32,
    pub flying: f32,
    pub riding: f32,

    /// The game's own field of view, degrees (the slider mirrors it).
    pub fov: f32,

    pub breathing: bool,
    pub breathing_amt: f32,
    /// 1 (slow breaths) to 10 (quick ones).
    pub breathing_speed: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            zoom: true,
            zoom_amount: 4.0,
            zoom_scroll: true,
            zoom_keep_scroll: false,
            zoom_smooth: true,
            zoom_speed: 5.0,
            zoom_slow_mouse: true,
            zoom_hide_hand: false,

            freecam: true,
            freecam_speed: 10.0,
            freecam_glide: true,
            freecam_through: true,
            freecam_show_player: true,

            motion: true,
            motion_third: false,
            motion_smooth: 5.0,
            strafe_roll: true,
            strafe_roll_amt: 1.0,
            turn_roll: true,
            turn_roll_amt: 1.0,
            move_pitch: true,
            move_pitch_amt: 1.0,
            air_pitch: true,
            air_pitch_amt: 1.0,
            sway: true,
            sway_amt: 1.0,
            sway_delay: 6.0,
            blast: true,
            blast_amt: 1.0,
            thunder: true,
            walking: 1.0,
            flying: 1.0,
            riding: 0.6,

            fov: 70.0,

            breathing: false,
            breathing_amt: 1.0,
            breathing_speed: 4.0,
        }
    }
}

pub const MIN_ZOOM: f32 = 1.5;
/// Sunrise bounds a mod's zoom at ten times.
pub const MAX_ZOOM: f32 = 10.0;
pub const MIN_FOV: f32 = 30.0;
pub const MAX_FOV: f32 = 110.0;
pub const MIN_FREECAM_SPEED: f32 = 1.0;
pub const MAX_FREECAM_SPEED: f32 = 100.0;

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

pub fn config() -> Config {
    CONFIG.lock().map(|c| c.unwrap_or_default()).unwrap_or_default()
}

pub fn config_loaded() -> bool {
    CONFIG.lock().map(|c| c.is_some()).unwrap_or(false)
}

pub fn set_config(config: Config) {
    if let Ok(mut c) = CONFIG.lock() {
        *c = Some(config);
    }
}
