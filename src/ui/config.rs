pub struct Colors {
    pub arc: u32,
    pub surface: u32,
    pub surface_hover: u32,
    pub text: u32,
}

impl Colors {
    pub const DEFAULT: Self = Self {
        arc: 0x4772b3,
        surface: 0x181818,
        surface_hover: 0x545454,
        text: 0xcdd6f4,
    };
}
