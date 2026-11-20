// SPDX-License-Identifier: MIT
// The mouse buttons on SuperCam's settings card.
fn supercam_card_options_mouse(
    game: &mut crate::game::Game<'_>,
    pressed: bool,
    right: bool,
) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::supercam_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::supercam_card::mouse(game, pressed, right))
}
