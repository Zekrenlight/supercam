// SPDX-License-Identifier: MIT
// Dragging on SuperCam's settings card: its sliders and scrollbar.
fn supercam_card_options_drag(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::supercam_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::supercam_card::drag(game))
}
