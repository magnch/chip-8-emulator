/// Compatibility options for CHIP-8 instruction variants.
pub struct Config {
    /// Use `VY` as the shift source instead of `VX`.
    pub shift_uses_vy: bool,
    /// Use `VX` as the offset source for `BNNN`.
    pub jmi_uses_vx: bool,
    /// Set `VF` when `FX1E` overflows the 12-bit address space.
    pub adi_flags_overflow: bool,
    /// Wait for a key to be released after `FX0A`.
    pub key_waits_for_release: bool,
    /// Increment `I` during `FX55` and `FX65`.
    pub str_ldr_increments_index: bool,
    /// Wrap sprites at the display edges.
    pub sprites_wrap_at_edge: bool,
}

impl Config {
    /// Create the default compatibility configuration.
    pub fn new() -> Self {
        Config {
            shift_uses_vy: false,
            jmi_uses_vx: false,
            adi_flags_overflow: false,
            key_waits_for_release: false,
            str_ldr_increments_index: false,
            sprites_wrap_at_edge: false,
        }
    }
}
