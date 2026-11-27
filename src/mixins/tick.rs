// After a SuperCam FOV slider change (queued from Hooks::setting_changed): write the game's own FOV.
// Applied on the client tick because native_setting_changed is now a shared &Game (cannot mutate settings there).
fn supercam_tick(game: &mut crate::game::Game<'_>) {
    crate::game::supercam_glue::setting_changed(game);
}
