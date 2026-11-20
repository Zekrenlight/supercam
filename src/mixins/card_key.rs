// SPDX-License-Identifier: MIT
// Keys on SuperCam's settings card; Escape (search let go) is Sunrise's, to leave.
fn supercam_card_options_key(
    game: &mut crate::game::Game<'_>,
    key: KeyCode,
    text: Option<&str>,
) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::supercam_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    match crate::game::supercam_card::key(game, key, text) {
        Some(done) => ::core::ops::ControlFlow::Break(done),
        None => ::core::ops::ControlFlow::Continue(()),
    }
}
