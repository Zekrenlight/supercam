// SPDX-License-Identifier: MIT
// The wheel on SuperCam's settings card.
fn supercam_card_options_scroll(
    game: &mut crate::game::Game<'_>,
    amount: f32,
) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::supercam_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::supercam_card::scroll(game, amount))
}
