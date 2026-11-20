// SuperCam in the game (not in the Cargo library): the FOV slider on SuperCam's page writes the
// game's own field of view, so it is the same value as Options > FOV and is saved to
// options.txt.
use super::Game;

/// After a setting on a mod's page changed: a new FOV from SuperCam's page goes to the game.
pub(crate) fn setting_changed(game: &mut Game<'_>) {
    let Some(fov) = mod_supercam::take_pending_fov() else {
        return;
    };
    let fov = fov.round().clamp(30.0, 110.0);
    if (game.settings.fov - fov).abs() < 0.5 {
        return;
    }
    game.settings.fov = fov;
    game.settings.save(&Game::options_path());
}
