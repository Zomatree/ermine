use freya::prelude::{State, consume_context};
use material_colors::{
    color::Rgb,
    theme::{Theme, ThemeBuilder},
};

pub fn generate_theme(base_color: u32) -> Theme {
    ThemeBuilder::with_source(Rgb::from_u32(base_color)).build()
}

pub fn default_theme_source() -> u32 {
    0x5470ec
}

pub fn consume_material_theme() -> crate::theme::Theme {
    *consume_context::<State<crate::theme::Theme>>().read()
}
