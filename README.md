# SuperCam

A Sunrise client mod by Zekrenlight (MIT).
Version 1.0.0 · Minecraft Java 26.3 · Sunrise API 0.2 · Mod SDK `9425651` · MIT licence.

Settings: Options > Mods... > SuperCam > Settings. The keys are in Options > Controls under
SuperCam. Every part has its own switch.

## Zoom
- Hold the zoom key (C at first) to look closer; let go to come back.
- Zoom Amount (1.5x to 10x; Sunrise allows a mod up to 10x).
- Scroll To Adjust: the mouse wheel goes closer or further while the key is held, instead of
  changing the hotbar slot. Remember Scrolling starts the next zoom where you left it.
- Smooth (with Speed 1 to 10), or instant.
- Slower Mouse: the view turns more slowly the closer you are. Hide Hand while zoomed.
- One zoom only: no second, cinematic zoom. The spyglass still uses its own view.

## Free camera
- The free camera key (F7 at first) sends the camera out; press it again to come back.
  The Free Camera switch turns the whole thing on or off.
- Your movement keys fly it (forward goes where you look, jump up, sneak down); hold sprint to
  go faster; the mouse wheel changes Speed (1 to 100 blocks a second).
- Your player stays exactly where you left them and can't hit, use or pick anything while the
  camera is out. Show Yourself, Glide and Through Blocks switches.
- Leaving the world brings the camera back.

## Camera motion
- Strafe Lean and Turn Lean (the view rolls into side steps and turns).
- Speed Dip (tips forward as you speed up) and Jump And Fall Tilt.
- Idle Sway after standing still (with Strength and Delay).
- Explosion Shake (closer is harder) and Lightning Shake (nearby strikes, distant rumble).
- A strength each for Walking And Swimming, Flying And Elytra, Riding And Boats.
- Smoothness, and In Third Person. The game's Distortion Effects setting scales all of it.
- The game's own damage tilt and view bobbing are left as they are; SuperCam doesn't add
  either.

## FOV
- An FOV slider (30 to 110) on SuperCam's page. It is the game's own FOV, the same one as the
  FOV slider at the top left of Options, and is saved with your options. Change either one and
  the other follows. Zoom works from whatever it is.

## Breathing hands
- Off at first. While you stand idle, your hand and what it holds rise and settle slowly, as if
  you were breathing (with Strength and Speed). It fades out as soon as you move, turn, swing or
  use something, and while the free camera is out.
- Some shader packs (Complementary, for one) do their own; use one or the other.

The Extras page's "Zoom: install from Mods" note is for Sunrise's official Zoom mod; SuperCam
works on its own and doesn't need it.

## Look and settings

SuperCam uses the shared Sunrise dark look: a `#0F1118` card with a 2 px accent edge, tabs and
charcoal chips. The accent is Sunrise orange `#F9801D` at first; change it with **Accent colour**
in the settings card (eight presets, or Reset).

The settings card opens from SunnyMods' **Settings** button, from Sunrise's own Mods list, or
from the mod's settings key. **Key binds › Rebind** switches to Sunrise's classic page to change keys.

## Compatibility

| Piece | Pin |
| --- | --- |
| Minecraft Java | 26.3 |
| Sunrise API | 0.2 |
| Mod SDK (`[native].sunrise`) | `94256510f74e6843754e0a93fd9e4da7e45b1056` (`9425651`, 19 Nov 2026) |
| Mixins (`[native].mixins`) | `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` |

SuperCam works on its own and alongside the other Sunrise mods by Zekrenlight. Its settings-card
hooks act only on its own Options page, so several mods can be installed together without any
conflicting replacements. A mod with native hooks works only on the exact SDK build it was made for.

## Building

```sh
cargo sunrise-mod           # fetches the pinned SDK into .sunrise/ (gitignored)
cargo sunrise-mod check     # type-checks against the pinned SDK
cargo test --lib            # unit tests, no game needed
cargo sunrise-mod build     # target/sunrise-mod/supercam.smod
```

To move to a newer SDK, run `cargo sunrise-mod update --sunrise <build>` (it rewrites the
`sunrise` and `mixins` lines in `sunrise-mod.toml`), then check and rebuild.

Install the `.smod` with sunrise-launcher (**Add from file**), or point **Add from GitHub** at
this repository. Native mods run with full access to your computer; read the source first.

## License

MIT © 2026 Zekrenlight. See [LICENSE](LICENSE).
