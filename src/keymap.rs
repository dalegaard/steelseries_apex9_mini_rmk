use rmk::types::action::KeyAction;
use rmk::{k, layer, mo};
pub(crate) const COL: usize = 14;
pub(crate) const ROW: usize = 5;
pub(crate) const NUM_LAYER: usize = 2;

const TRPT: KeyAction = KeyAction::Transparent;

#[rustfmt::skip]
pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        // Base layer
        layer!([
            [k!(Escape), k!(Kc1), k!(Kc2), k!(Kc3), k!(Kc4), k!(Kc5), k!(Kc6), k!(Kc7), k!(Kc8), k!(Kc9), k!(Kc0), k!(Minus), k!(Equal), TRPT],
            [k!(Tab), k!(Q), k!(W), k!(E), k!(R), k!(T), k!(Y), k!(U), k!(I), k!(O), k!(P), k!(LeftBracket), k!(RightBracket), TRPT],
            [k!(CapsLock), k!(A), k!(S), k!(D), k!(F), k!(G), k!(H), k!(J), k!(K), k!(L), k!(Semicolon), k!(Quote), k!(NonusHash), k!(Enter)],
            [k!(LShift), k!(NonusBackslash), k!(Z), k!(X), k!(C), k!(V), k!(B), k!(N), k!(M), k!(Comma), k!(Dot), k!(Slash), TRPT, k!(RShift)],
            [k!(LCtrl), k!(LGui), k!(LAlt), TRPT, k!(Space), TRPT, TRPT, k!(RAlt), mo!(1), k!(RGui), k!(RCtrl), k!(Backspace), TRPT, TRPT]
        ]),

        // Fn layer
        layer!([
            [k!(Grave), k!(F1), k!(F2), k!(F3), k!(F4), k!(F5), k!(F6), k!(F7), k!(F8), k!(F9), k!(F10), k!(F11), k!(F12), TRPT], 
            [TRPT, TRPT, k!(Up), TRPT, TRPT, TRPT, TRPT, TRPT, TRPT, TRPT, k!(PrintScreen), k!(Home), k!(PageUp), TRPT], 
            [TRPT, k!(Left), k!(Down), k!(Right), TRPT, TRPT, TRPT, TRPT, TRPT, k!(Insert), k!(End), k!(PageDown), TRPT, TRPT], 
            [TRPT, TRPT, TRPT, TRPT, k!(BrightnessDown), k!(BrightnessUp), k!(AudioVolDown), k!(AudioVolUp), k!(AudioMute), k!(MediaRewind), k!(MediaFastForward), k!(MediaPlayPause), TRPT, TRPT], 
            [TRPT, TRPT, TRPT, TRPT, TRPT, TRPT, TRPT, TRPT, TRPT, TRPT, TRPT, k!(Delete), TRPT, TRPT]
        ]),
    ]
}
