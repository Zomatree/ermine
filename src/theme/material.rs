use freya::prelude::{State, consume_context};
use material_colors::{
    color::Rgb,
    dynamic_color::Variant,
    theme::{Theme, ThemeBuilder},
};
use serde::{Deserialize, Serialize};

pub fn generate_theme(base_color: u32, variant: Variant) -> Theme {
    ThemeBuilder::with_source(Rgb::from_u32(base_color))
        .variant(variant)
        .build()
}

pub fn default_theme_source() -> u32 {
    0x5470ec
}

pub fn consume_material_theme() -> crate::theme::Theme {
    *consume_context::<State<crate::theme::Theme>>().read()
}

pub fn peak_material_theme() -> crate::theme::Theme {
    *consume_context::<State<crate::theme::Theme>>().peek()
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum ThemeVariant {
    Monochrome,
    Neutral,
    #[default]
    TonalSpot,
    Vibrant,
    Expressive,
    Fidelity,
    Content,
    Rainbow,
    FruitSalad,
}

impl From<Variant> for ThemeVariant {
    fn from(value: Variant) -> Self {
        match value {
            Variant::Monochrome => ThemeVariant::Monochrome,
            Variant::Neutral => ThemeVariant::Neutral,
            Variant::TonalSpot => ThemeVariant::TonalSpot,
            Variant::Vibrant => ThemeVariant::Vibrant,
            Variant::Expressive => ThemeVariant::Expressive,
            Variant::Fidelity => ThemeVariant::Fidelity,
            Variant::Content => ThemeVariant::Content,
            Variant::Rainbow => ThemeVariant::Rainbow,
            Variant::FruitSalad => ThemeVariant::FruitSalad,
        }
    }
}

impl From<ThemeVariant> for Variant {
    fn from(value: ThemeVariant) -> Self {
        match value {
            ThemeVariant::Monochrome => Variant::Monochrome,
            ThemeVariant::Neutral => Variant::Neutral,
            ThemeVariant::TonalSpot => Variant::TonalSpot,
            ThemeVariant::Vibrant => Variant::Vibrant,
            ThemeVariant::Expressive => Variant::Expressive,
            ThemeVariant::Fidelity => Variant::Fidelity,
            ThemeVariant::Content => Variant::Content,
            ThemeVariant::Rainbow => Variant::Rainbow,
            ThemeVariant::FruitSalad => Variant::FruitSalad,
        }
    }
}
