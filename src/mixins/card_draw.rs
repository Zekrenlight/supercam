// SPDX-License-Identifier: MIT
// SuperCam's settings page: its card is drawn instead of Sunrise's generic page.
fn supercam_card_options_draw(
    game: &mut crate::game::Game<'_>,
    batch: &mut sunrise_client::hud::HudBatch,
) -> ::core::ops::ControlFlow<()> {
    if !crate::game::supercam_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    crate::game::supercam_card::draw(game, batch);
    ::core::ops::ControlFlow::Break(())
}
