// The mouse turns the player: while the free camera is out it turns the camera instead.
fn supercam_turn(xo: f64, yo: f64) -> ::core::ops::ControlFlow<()> {
    if mod_supercam::freecam_takes_turn(xo, yo) {
        ::core::ops::ControlFlow::Break(())
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
