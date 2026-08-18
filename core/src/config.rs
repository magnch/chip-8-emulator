pub struct Config {
    pub shift_uses_vy: bool,
    pub jmi_uses_vx: bool,
    pub adi_flags_overflow: bool,
    pub key_waits_for_release: bool,
    pub str_ldr_increments_index: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            shift_uses_vy: false,
            jmi_uses_vx: false,
            adi_flags_overflow: false,
            key_waits_for_release: false,
            str_ldr_increments_index: false
        }
    }
}