//! SuperCam 1.0.0 for Sunrise, by Zekrenlight (MIT).
//!
//! - Zoom: hold a key (C at first) and the view eases in; the mouse wheel goes closer or further
//!   while it is held, the mouse can turn slower, and the hand can step aside.
//! - Free camera: a key (F7 at first) sends the camera out to fly on its own while the player
//!   stays exactly where they were, and brings it back.
//! - Camera motion: leaning into strafes and turns, a dip when you speed up, a lift on jumps and
//!   a sink into falls, a slow sway after standing still, and a shake from explosions and
//!   lightning nearby, each with its own switch and strength, and a strength each for walking,
//!   flying and riding.
//! - FOV: a slider for the game's own field of view (the same value as Options > FOV).
//! - Breathing hands (off at first): the first-person hand rises and settles slowly while idle.
//!
//! Everything goes through the typed mod API but the mouse turning the free camera
//! (`src/mixins/turn.rs`). The parts with no game types are `zoom`, `freecam` and `motion`;
//! settings live on the mod's page, which Mod Menu opens from Options > Mods....

pub mod config;
pub mod freecam;
pub mod hands;
pub mod motion;
pub mod zoom;

pub use config::*;

use mod_api::{
    CameraPoseEvent, CameraShakeEvent, CameraType, ClientHost, ClientLevelUnloadEvent,
    ClientTickPostEvent, Flow, Hooks, InteractionKeyMappingTriggeredEvent, KeyRegistration,
    LoopContext, MouseScrollingEvent, MovementInputEvent, PlaySoundEvent, Registrar,
    RegistrationContext, RenderFramePreEvent, RenderHandEvent, RenderPlayerPreEvent,
    SettingChangedEvent, SettingSpec, ZoomEvent,
};
use motion::{Reading, Situation};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
struct SuperCam;

fn label(key: &str) -> String {
    format!("supercam.setting.{key}")
}

fn toggle(registrar: &mut dyn Registrar, key: &str, default: bool) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    let _ = registrar.register_setting(SettingSpec::toggle(key, l, default).tooltip(tip));
}

fn range(registrar: &mut dyn Registrar, key: &str, min: f64, max: f64, step: f64, default: f64) {
    let l = label(key);
    let tip = format!("{l}.tooltip");
    let _ = registrar
        .register_setting(SettingSpec::range(key, l, min, max, step, default).tooltip(tip));
}

/// A strength in percent on the page, a fraction in the config.
fn percent(registrar: &mut dyn Registrar, key: &str, max: f64, default: f32) {
    range(registrar, key, 0.0, max, 10.0, f64::from((default * 100.0).round()));
}

/// Reads every setting into the copy the hooks use.
fn sync(host: &dyn ClientHost) -> Config {
    let d = Config::default();
    let b = |key: &str, default: bool| host.setting_bool(key).unwrap_or(default);
    let n = |key: &str, default: f32, min: f32, max: f32| {
        host.setting_f64(key)
            .filter(|v| v.is_finite())
            .map_or(default, |v| v as f32)
            .clamp(min, max)
    };
    let p = |key: &str, default: f32| n(key, default * 100.0, 0.0, 200.0) / 100.0;
    let c = Config {
        zoom: b(KEY_ZOOM, d.zoom),
        zoom_amount: n(KEY_ZOOM_AMOUNT, d.zoom_amount, MIN_ZOOM, MAX_ZOOM),
        zoom_scroll: b(KEY_ZOOM_SCROLL, d.zoom_scroll),
        zoom_keep_scroll: b(KEY_ZOOM_KEEP_SCROLL, d.zoom_keep_scroll),
        zoom_smooth: b(KEY_ZOOM_SMOOTH, d.zoom_smooth),
        zoom_speed: n(KEY_ZOOM_SPEED, d.zoom_speed, 1.0, 10.0),
        zoom_slow_mouse: b(KEY_ZOOM_SLOW_MOUSE, d.zoom_slow_mouse),
        zoom_hide_hand: b(KEY_ZOOM_HIDE_HAND, d.zoom_hide_hand),

        freecam: b(KEY_FREECAM, d.freecam),
        freecam_speed: n(KEY_FREECAM_SPEED, d.freecam_speed, MIN_FREECAM_SPEED, MAX_FREECAM_SPEED),
        freecam_glide: b(KEY_FREECAM_GLIDE, d.freecam_glide),
        freecam_through: b(KEY_FREECAM_THROUGH, d.freecam_through),
        freecam_show_player: b(KEY_FREECAM_SHOW_PLAYER, d.freecam_show_player),

        motion: b(KEY_MOTION, d.motion),
        motion_third: b(KEY_MOTION_THIRD, d.motion_third),
        motion_smooth: n(KEY_MOTION_SMOOTH, d.motion_smooth, 1.0, 10.0),
        strafe_roll: b(KEY_STRAFE_ROLL, d.strafe_roll),
        strafe_roll_amt: p(KEY_STRAFE_ROLL_AMT, d.strafe_roll_amt),
        turn_roll: b(KEY_TURN_ROLL, d.turn_roll),
        turn_roll_amt: p(KEY_TURN_ROLL_AMT, d.turn_roll_amt),
        move_pitch: b(KEY_MOVE_PITCH, d.move_pitch),
        move_pitch_amt: p(KEY_MOVE_PITCH_AMT, d.move_pitch_amt),
        air_pitch: b(KEY_AIR_PITCH, d.air_pitch),
        air_pitch_amt: p(KEY_AIR_PITCH_AMT, d.air_pitch_amt),
        sway: b(KEY_SWAY, d.sway),
        sway_amt: p(KEY_SWAY_AMT, d.sway_amt),
        sway_delay: n(KEY_SWAY_DELAY, d.sway_delay, 1.0, 30.0),
        blast: b(KEY_BLAST, d.blast),
        blast_amt: p(KEY_BLAST_AMT, d.blast_amt),
        thunder: b(KEY_THUNDER, d.thunder),
        walking: p(KEY_WALKING, d.walking),
        flying: p(KEY_FLYING, d.flying),
        riding: p(KEY_RIDING, d.riding),

        fov: n(KEY_FOV, d.fov, MIN_FOV, MAX_FOV),

        breathing: b(KEY_BREATHING, d.breathing),
        breathing_amt: p(KEY_BREATHING_AMT, d.breathing_amt),
        breathing_speed: n(KEY_BREATHING_SPEED, d.breathing_speed, 1.0, 10.0),
    };
    set_config(c);
    c
}

fn cfg(host: &dyn ClientHost) -> Config {
    if config_loaded() {
        config()
    } else {
        sync(host)
    }
}

// ---- What the hooks share besides the three parts' own state ----

#[derive(Clone, Copy, Debug, Default)]
struct Shared {
    /// The player's position last tick, and their facing then.
    last: Option<([f64; 3], f32)>,
    /// Where the player's eyes were last tick, for how near a blast was.
    eyes: Option<[f64; 3]>,
    /// The player moved or turned last tick.
    moving: bool,
}

static SHARED: Mutex<Shared> = Mutex::new(Shared {
    last: None,
    eyes: None,
    moving: false,
});

// ---- FOV ----

/// A new FOV from SuperCam's page, waiting for the game half to set it.
static PENDING_FOV: Mutex<Option<f32>> = Mutex::new(None);

/// The game half's question after a setting changed: is there a new FOV to set?
pub fn take_pending_fov() -> Option<f32> {
    PENDING_FOV.lock().unwrap_or_else(|e| e.into_inner()).take()
}

fn fov_pending() -> bool {
    PENDING_FOV.lock().map(|p| p.is_some()).unwrap_or(false)
}

/// The game's FOV changed elsewhere (Options > FOV): the slider follows it.
fn mirror_fov(host: &dyn ClientHost, c: &Config) {
    let game = host.graphics_options().fov.round();
    if fov_pending() || !game.is_finite() || (game - c.fov).abs() < 0.5 {
        return;
    }
    host.set_setting(ID, KEY_FOV, &game.to_string());
    set_config(Config { fov: game, ..*c });
}

/// A hand swung since the last frame.
static SWUNG: AtomicBool = AtomicBool::new(false);

fn shared<R>(f: impl FnOnce(&mut Shared) -> R) -> R {
    let mut guard = SHARED.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut guard)
}

// ---- Free camera ----

fn view_index(view: CameraType) -> u8 {
    match view {
        CameraType::FirstPerson => 0,
        CameraType::ThirdPersonBack => 1,
        CameraType::ThirdPersonFront => 2,
    }
}

fn view_of(index: u8) -> CameraType {
    match index {
        1 => CameraType::ThirdPersonBack,
        2 => CameraType::ThirdPersonFront,
        _ => CameraType::FirstPerson,
    }
}

fn freecam_out(host: &dyn ClientHost) {
    let Some(player) = host.local_player() else {
        return;
    };
    let entity = player.entity();
    let (Some(eyes), Some([yaw, pitch])) =
        (entity.eye_position(host), entity.rotation(host))
    else {
        return;
    };
    let view = view_index(host.camera_type());
    freecam::with(|c| *c = Some(freecam::Cam::new(eyes, yaw, pitch, view)));
    // Behind the player: the game then draws the body and no first-person hand.
    host.set_camera_type(CameraType::ThirdPersonBack);
    motion::with(|m| m.settle());
}

fn freecam_back(host: &dyn ClientHost) {
    if let Some(cam) = freecam::with(|c| c.take()) {
        host.set_camera_type(view_of(cam.view));
    }
}

/// The mixin's question, once per mouse move: does the free camera take this turn?
pub fn freecam_takes_turn(xo: f64, yo: f64) -> bool {
    freecam::with(|c| match c {
        Some(cam) => {
            cam.turn(xo, yo);
            true
        }
        None => false,
    })
}

fn held(host: &dyn ClientHost, mapping: &str) -> bool {
    matches!(host.mapping(mapping), Ok(Some(m)) if m.down)
}

fn freecam_tick(host: &dyn ClientHost, c: &Config) {
    let input = freecam::Input {
        forward: held(host, "key.forward"),
        back: held(host, "key.back"),
        left: held(host, "key.left"),
        right: held(host, "key.right"),
        up: held(host, "key.jump"),
        down: held(host, "key.sneak"),
        fast: held(host, "key.sprint"),
    };
    let input = if host.playing() {
        input
    } else {
        freecam::Input::default()
    };
    let blocked = |min: [f64; 3], max: [f64; 3]| {
        let lo = min.map(|v| v.floor() as i32);
        let hi = max.map(|v| v.floor() as i32);
        (lo[0]..=hi[0]).any(|x| {
            (lo[1]..=hi[1]).any(|y| {
                (lo[2]..=hi[2]).any(|z| {
                    let at = [x, y, z];
                    freecam::overlaps(min, max, at, &host.block_collision(at))
                })
            })
        })
    };
    freecam::with(|cam| {
        if let Some(cam) = cam {
            cam.tick(
                input,
                f64::from(c.freecam_speed),
                c.freecam_glide,
                c.freecam_through,
                &blocked,
            );
        }
    });
    host.set_camera_type(CameraType::ThirdPersonBack);
}

// ---- Camera motion ----

fn situation(host: &dyn ClientHost, entity: mod_api::EntityRef, player: mod_api::PlayerRef) -> Situation {
    if entity.vehicle(host).is_some() {
        return Situation::Riding;
    }
    let flying = player.abilities(host).is_some_and(|a| a.flying);
    if flying || entity.pose(host) == Some("fall_flying") {
        Situation::Flying
    } else {
        Situation::Walking
    }
}

fn motion_tick(host: &dyn ClientHost, c: &Config) {
    let Some(player) = host.local_player() else {
        shared(|s| *s = Shared::default());
        return;
    };
    let entity = player.entity();
    let (Some(at), Some([yaw, _])) = (entity.position(host), entity.rotation(host))
    else {
        return;
    };
    let eyes = entity.eye_position(host);
    let last = shared(|s| {
        s.eyes = eyes;
        s.last.replace((at, yaw))
    });
    let Some((before, yaw_before)) = last else {
        return;
    };
    let moved = [at[0] - before[0], at[1] - before[1], at[2] - before[2]];
    let moving = moved.iter().map(|v| v.abs()).sum::<f64>() > 0.003 || wrap_turn(yaw - yaw_before).abs() > 0.5;
    shared(|s| s.moving = moving);
    // A teleport or a respawn is not a movement.
    if moved.iter().any(|v| v.abs() > 4.0) || freecam::active() || !c.motion {
        motion::with(|m| m.settle());
        return;
    }
    let reading = Reading {
        moved,
        yaw,
        turned: yaw - yaw_before,
        situation: situation(host, entity, player),
    };
    motion::with(|m| m.read(reading, c));
}

fn wrap_turn(degrees: f32) -> f32 {
    (degrees + 180.0).rem_euclid(360.0) - 180.0
}

fn sound_shake(id: &str, at: [f64; 3], c: &Config) {
    let Some(eyes) = shared(|s| s.eyes) else {
        return;
    };
    let distance = ((at[0] - eyes[0]).powi(2) + (at[1] - eyes[1]).powi(2) + (at[2] - eyes[2]).powi(2)).sqrt();
    match id {
        "minecraft:entity.generic.explode" if c.blast => motion::with(|m| m.blast(distance, 1.0)),
        "minecraft:entity.lightning_bolt.impact" if c.thunder => {
            motion::with(|m| m.blast(distance, 0.6))
        }
        "minecraft:entity.lightning_bolt.thunder" if c.thunder => {
            motion::with(|m| m.rumble(0.15))
        }
        _ => {}
    }
}

#[mod_api::sunrise_mod]
impl Hooks for SuperCam {
    fn register(&self, registrar: &mut dyn Registrar, _cx: &RegistrationContext) {
        registrar.register_key_mapping(
            KeyRegistration::new(ZOOM_KEY, ZOOM_KEY_DEFAULT)
                .label("supercam.key.zoom")
                .category("supercam"),
        );
        registrar.register_key_mapping(
            KeyRegistration::new(FREECAM_KEY, FREECAM_KEY_DEFAULT)
                .label("supercam.key.freecam")
                .category("supercam"),
        );
        let d = Config::default();
        // Zoom
        toggle(registrar, KEY_ZOOM, d.zoom);
        range(registrar, KEY_ZOOM_AMOUNT, f64::from(MIN_ZOOM), f64::from(MAX_ZOOM), 0.5, f64::from(d.zoom_amount));
        toggle(registrar, KEY_ZOOM_SCROLL, d.zoom_scroll);
        toggle(registrar, KEY_ZOOM_KEEP_SCROLL, d.zoom_keep_scroll);
        toggle(registrar, KEY_ZOOM_SMOOTH, d.zoom_smooth);
        range(registrar, KEY_ZOOM_SPEED, 1.0, 10.0, 1.0, f64::from(d.zoom_speed));
        toggle(registrar, KEY_ZOOM_SLOW_MOUSE, d.zoom_slow_mouse);
        toggle(registrar, KEY_ZOOM_HIDE_HAND, d.zoom_hide_hand);
        // Free camera
        toggle(registrar, KEY_FREECAM, d.freecam);
        range(registrar, KEY_FREECAM_SPEED, f64::from(MIN_FREECAM_SPEED), f64::from(MAX_FREECAM_SPEED), 1.0, f64::from(d.freecam_speed));
        toggle(registrar, KEY_FREECAM_GLIDE, d.freecam_glide);
        toggle(registrar, KEY_FREECAM_THROUGH, d.freecam_through);
        toggle(registrar, KEY_FREECAM_SHOW_PLAYER, d.freecam_show_player);
        // Camera motion
        toggle(registrar, KEY_MOTION, d.motion);
        toggle(registrar, KEY_MOTION_THIRD, d.motion_third);
        range(registrar, KEY_MOTION_SMOOTH, 1.0, 10.0, 1.0, f64::from(d.motion_smooth));
        toggle(registrar, KEY_STRAFE_ROLL, d.strafe_roll);
        percent(registrar, KEY_STRAFE_ROLL_AMT, 200.0, d.strafe_roll_amt);
        toggle(registrar, KEY_TURN_ROLL, d.turn_roll);
        percent(registrar, KEY_TURN_ROLL_AMT, 200.0, d.turn_roll_amt);
        toggle(registrar, KEY_MOVE_PITCH, d.move_pitch);
        percent(registrar, KEY_MOVE_PITCH_AMT, 200.0, d.move_pitch_amt);
        toggle(registrar, KEY_AIR_PITCH, d.air_pitch);
        percent(registrar, KEY_AIR_PITCH_AMT, 200.0, d.air_pitch_amt);
        toggle(registrar, KEY_SWAY, d.sway);
        percent(registrar, KEY_SWAY_AMT, 200.0, d.sway_amt);
        range(registrar, KEY_SWAY_DELAY, 1.0, 30.0, 1.0, f64::from(d.sway_delay));
        toggle(registrar, KEY_BLAST, d.blast);
        percent(registrar, KEY_BLAST_AMT, 200.0, d.blast_amt);
        toggle(registrar, KEY_THUNDER, d.thunder);
        percent(registrar, KEY_WALKING, 200.0, d.walking);
        percent(registrar, KEY_FLYING, 200.0, d.flying);
        percent(registrar, KEY_RIDING, 200.0, d.riding);
        // FOV
        range(registrar, KEY_FOV, f64::from(MIN_FOV), f64::from(MAX_FOV), 1.0, f64::from(d.fov));
        // Breathing hands
        toggle(registrar, KEY_BREATHING, d.breathing);
        percent(registrar, KEY_BREATHING_AMT, 200.0, d.breathing_amt);
        range(registrar, KEY_BREATHING_SPEED, 1.0, 10.0, 1.0, f64::from(d.breathing_speed));
        // The accent colour of supercam's cards (Sunrise orange by default).
        let _ = registrar.register_setting(
            mod_api::SettingSpec::color("accent", "supercam.setting.accent", 0xF9801D)
                .tooltip("supercam.setting.accent.tooltip"),
        );
    }

    fn setting_changed(&self, host: &mut dyn ClientHost, event: &mut SettingChangedEvent<'_>) {
        if event.mod_id == ID {
            let c = sync(host);
            if event.key == KEY_FOV {
                if let Ok(mut p) = PENDING_FOV.lock() {
                    *p = Some(c.fov);
                }
            }
        }
    }

    fn client_tick_post(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPostEvent) {
        let c = cfg(host);
        let on = host.on(ID);
        // The free camera key: a press while playing (Controls can rebind it).
        let pressed = host.consume_click(ID, FREECAM_KEY);
        if freecam::active() {
            if pressed || !on || !c.freecam || host.local_player().is_none() {
                freecam_back(host);
            } else {
                freecam_tick(host, &c);
            }
        } else if pressed && on && c.freecam {
            freecam_out(host);
        }
        if on {
            motion_tick(host, &c);
        }
        mirror_fov(host, &config());
    }

    fn render_frame_pre(&self, host: &mut dyn ClientHost, event: &mut RenderFramePreEvent) {
        let c = cfg(host);
        let seconds = event.elapsed.as_secs_f32();
        let on = host.on(ID);
        let held = on && host.playing() && host.is_down(ID, ZOOM_KEY);
        zoom::with(|z| {
            z.hold(held, &c);
            z.step(seconds, &c);
        });
        motion::with(|m| m.frame(seconds, &c));
        let swung = SWUNG.swap(false, Ordering::Relaxed);
        let busy = swung
            || shared(|s| s.moving)
            || host.local_item_use().is_some()
            || freecam::active()
            || !on;
        hands::with(|b| b.frame(seconds, busy, &c));
    }

    fn zoom(&self, _host: &dyn ClientHost, _event: &mut ZoomEvent) -> Option<mod_api::Zoom> {
        let c = config();
        zoom::with(|z| {
            z.zoomed().then(|| mod_api::Zoom {
                factor: z.now,
                // SuperCam eases the zoom itself, at its own speed.
                smooth: false,
                turn: z.turn(&c),
            })
        })
    }

    fn mouse_scrolling(&self, host: &mut dyn ClientHost, event: &mut MouseScrollingEvent) -> Flow {
        if !host.on(ID) {
            return Flow::Continue;
        }
        let notches = if event.accumulated[1] != 0 {
            event.accumulated[1] as f32
        } else {
            event.scroll[1] as f32
        };
        if notches == 0.0 {
            return Flow::Continue;
        }
        let c = cfg(host);
        if zoom::with(|z| z.scroll(notches, &c)) {
            return Flow::Cancel;
        }
        if freecam::active() {
            // The wheel sets the free camera's speed, and the page keeps it.
            let speed = (c.freecam_speed * 1.15f32.powf(notches))
                .clamp(MIN_FREECAM_SPEED, MAX_FREECAM_SPEED)
                .round();
            host.set_setting(ID, KEY_FREECAM_SPEED, &speed.to_string());
            set_config(Config {
                freecam_speed: speed,
                ..c
            });
            return Flow::Cancel;
        }
        Flow::Continue
    }

    fn render_hand(&self, _host: &dyn ClientHost, event: &mut RenderHandEvent<'_, '_>) -> Flow {
        let c = config();
        if c.zoom_hide_hand && zoom::with(|z| z.zoomed()) {
            return Flow::Cancel;
        }
        if event.render.swing > 0.0 {
            SWUNG.store(true, Ordering::Relaxed);
        }
        if c.breathing {
            let (lift, nod) = hands::with(|b| b.offset(&c));
            for i in 0..3 {
                event.render.translation[i] += lift[i];
            }
            event.render.rotation[0] += nod;
        }
        Flow::Continue
    }

    fn render_player_pre(&self, _host: &dyn ClientHost, event: &mut RenderPlayerPreEvent<'_>) -> Flow {
        if event.who.local && freecam::active() && !config().freecam_show_player {
            Flow::Cancel
        } else {
            Flow::Continue
        }
    }

    fn camera_pose(&self, _host: &dyn ClientHost, event: &mut CameraPoseEvent<'_>) {
        let posed = freecam::with(|c| c.map(|cam| (cam.at(event.partial), cam.yaw, cam.pitch)));
        if let Some((at, yaw, pitch)) = posed {
            *event.position = at;
            event.yaw = yaw;
            event.pitch = pitch;
        }
    }

    fn camera_shake(&self, host: &dyn ClientHost, event: &mut CameraShakeEvent<'_>) {
        let c = config();
        if !c.motion || freecam::active() || !host.on(ID) {
            return;
        }
        if !c.motion_third && !host.camera_type().is_first_person() {
            return;
        }
        let (a, offset) = motion::with(|m| (m.out, m.out_offset));
        event.shake.pitch += a.pitch;
        event.shake.yaw += a.yaw;
        event.shake.roll += a.roll;
        for i in 0..3 {
            event.shake.offset[i] += offset[i];
        }
    }

    fn movement_input(&self, _host: &dyn ClientHost, event: &mut MovementInputEvent) {
        if !freecam::active() {
            return;
        }
        // The keys fly the camera: the player stands still.
        event.forward = 0.0;
        event.strafe = 0.0;
        event.jump = false;
        event.sneak = false;
        event.forward_key = false;
        event.backward_key = false;
        event.left_key = false;
        event.right_key = false;
        event.sprint = false;
    }

    fn interaction_key_mapping_triggered(
        &self,
        _host: &mut dyn ClientHost,
        event: &mut InteractionKeyMappingTriggeredEvent,
    ) -> Flow {
        if freecam::active() {
            // No hitting, using or picking while the camera is away.
            event.swing_hand = false;
            Flow::Cancel
        } else {
            Flow::Continue
        }
    }

    fn play_sound(&self, _context: &dyn LoopContext, event: &mut PlaySoundEvent<'_>) {
        if !event.starting || event.original.relative {
            return;
        }
        let c = config();
        if !c.motion {
            return;
        }
        sound_shake(&event.original.identifier, event.original.position, &c);
    }

    fn client_level_unload(&self, host: &mut dyn ClientHost, _event: &mut ClientLevelUnloadEvent<'_>) {
        freecam_back(host);
        shared(|s| *s = Shared::default());
        motion::with(|m| m.settle());
    }
}
