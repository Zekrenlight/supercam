// After a setting on a mod's page changed (and the mods heard it): SuperCam's FOV goes to the game.
fn supercam_setting_changed(value: (), game: &mut crate::game::Game<'_>) {
    let () = value;
    crate::game::supercam_glue::setting_changed(game);
}
