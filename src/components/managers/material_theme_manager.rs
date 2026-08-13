use std::{
    mem,
    sync::{Arc, LazyLock, RwLock},
    time::Duration,
};

use freya::{
    animation::{
        AnimDirection, AnimNum, AnimatedValue, Ease, OnChange, OnCreation, ReadAnimatedValue,
        use_animation,
    },
    prelude::*,
    radio::IntoReadable,
};
use material_colors::{color::Rgb, scheme::Scheme};

use crate::{
    Config, StoatScheme, ThemeConfig, ThemeScheme,
    components::{StoatButtonColorsThemePreference, StoatButtonLayoutThemePreference},
    generate_theme, map_readable,
    theme::Theme,
    use_config,
};

fn generate(theme_config: &ThemeConfig) -> Theme {
    let material_theme = generate_theme(theme_config.theme_source);

    let material_scheme = match theme_config.scheme {
        ThemeScheme::Light => material_theme.schemes.light,
        ThemeScheme::Dark => material_theme.schemes.dark,
    };

    let stoat_scheme = StoatScheme::default();

    Theme {
        md: material_scheme,
        stoat: stoat_scheme,
    }
}

static INITIAL_THEME: LazyLock<Arc<RwLock<Option<Theme>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

fn update_freya_theme(theme: &mut freya::prelude::Theme, material: &Theme) {
    theme.set(
        "stoat_button",
        StoatButtonColorsThemePreference {
            background: Preference::Specific(Color::TRANSPARENT),
            hover_background: Preference::Specific(Color::TRANSPARENT),
            border_fill: Preference::Specific(Color::TRANSPARENT),
            focus_border_fill: Preference::Specific(Color::TRANSPARENT),
            color: Preference::Specific(Color::TRANSPARENT),
        },
    );

    theme.set(
        "stoat_button_layout",
        StoatButtonLayoutThemePreference {
            margin: Preference::Specific(Gaps::new_all(0.)),
            corner_radius: Preference::Specific(CornerRadius::new_all(0.)),
            width: Preference::Specific(Size::Inner),
            height: Preference::Specific(Size::Inner),
            padding: Preference::Specific(Gaps::new_all(0.)),
        },
    );

    // let hover_overlay_r = (material.md.on_surface.red as f32 * 0.08) as u8;
    // let hover_overlay_g = (material.md.on_surface.green as f32 * 0.08) as u8;
    // let hover_overlay_b = (material.md.on_surface.blue as f32 * 0.08) as u8;

    // let hover_r = (material.md.surface_container.red + hover_overlay_r).min(255);
    // let hover_g = (material.md.surface_container.green + hover_overlay_g).min(255);
    // let hover_b = (material.md.surface_container.blue + hover_overlay_b).min(255);

    // let hover_color = Color::from_rgb(hover_r, hover_g, hover_b);

    // theme.set(
    //     "menu_item",
    //     MenuItemThemePreference {
    //         background: Preference::Specific(Color::TRANSPARENT),
    //         hover_background: Preference::Specific(hover_color),
    //         select_background: Preference::Specific(hover_color),
    //         border_fill: Preference::Specific(Color::TRANSPARENT),
    //         select_border_fill: Preference::Reference("border_focus"),
    //         corner_radius: Preference::Specific(CornerRadius::new_all(6.)),
    //         color: Preference::Reference("text_primary"),
    //     },
    // );

    theme.set(
        "menu_container",
        MenuContainerThemePreference {
            background: Preference::Specific(material.md.surface_container.as_argb_u32().into()),
            padding: Preference::Specific((8., 1.).into()),
            shadow: Preference::Reference("shadow"),
            border_fill: Preference::Specific(Color::BLACK),
            corner_radius: Preference::Specific(CornerRadius::new_all(4.)),
        },
    );
}

#[derive(PartialEq)]
pub struct MaterialThemeProvider {
    children: Vec<Element>,
}

impl MaterialThemeProvider {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl ChildrenExt for MaterialThemeProvider {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl Component for MaterialThemeProvider {
    fn render(&self) -> impl IntoElement {
        let config = use_config();

        let mut state = use_hook(|| {
            let theme_config = config.read().theme;
            let theme = generate(&theme_config);
            *INITIAL_THEME.write().unwrap() = Some(theme);
            let state = State::create(theme);
            provide_context(state);

            state
        });

        let theme_config =
            map_readable::<Config, ThemeConfig>(config.clone().into_readable(), |config| {
                &config.theme
            });

        let mut theme_change =
            use_state(|| (theme_config.read().clone(), theme_config.read().clone()));

        use_side_effect_with_deps(&*theme_config.read(), move |value| {
            let mut theme_change = theme_change.write();
            let old_current = mem::replace(&mut theme_change.1, value.clone());
            theme_change.0 = old_current;
        });

        let anim = use_animation(move |conf| {
            conf.on_change(OnChange::Rerun);
            conf.on_creation(OnCreation::Run);
            let (from, to) = theme_change.read().clone();

            AnimTheme::new(generate(&from), generate(&to))
        });

        let mut freya_theme = use_init_theme(|| {
            let mut theme = dark_theme();

            update_freya_theme(&mut theme, &anim.read().value);

            theme
        });

        use_side_effect(move || {
            let theme = anim.read().value();

            update_freya_theme(&mut freya_theme.write(), &theme);

            state.set(theme);
        });

        let theme = state.read();

        rect()
            .font_family("Inter")
            // .font_family("Fluent Emoji Color")
            .font_size(14.)
            .font_weight(400)
            .width(Size::Fill)
            .height(Size::Fill)
            .color(theme.md.on_surface.as_argb_u32())
            .background(theme.md.surface_container_high.as_argb_u32())
            .children(self.children.clone())
    }
}

#[derive(Clone, PartialEq)]
pub struct AnimTheme {
    origin: Theme,
    destination: Theme,
    inner: AnimNum,

    value: Theme,
}

impl Default for AnimTheme {
    fn default() -> Self {
        let theme = INITIAL_THEME.read().unwrap().unwrap();

        Self {
            origin: theme,
            destination: theme,
            inner: Default::default(),
            value: theme,
        }
    }
}

impl AnimTheme {
    pub fn new(origin: Theme, destination: Theme) -> Self {
        let origin = origin.into();
        Self {
            origin,
            destination,
            inner: AnimNum::new(0., 1.)
                .duration(Duration::from_millis(250))
                .ease(Ease::InOut),
            value: origin,
        }
    }

    pub fn value(&self) -> Theme {
        self.value
    }
}

impl AnimatedValue for AnimTheme {
    fn prepare(&mut self, direction: AnimDirection) {
        match direction {
            AnimDirection::Forward => self.value = self.origin,
            AnimDirection::Reverse => {
                self.value = self.destination;
            }
        }
    }

    fn is_finished(&self, index: u128, direction: AnimDirection) -> bool {
        self.inner.is_finished(index, direction)
    }

    fn advance(&mut self, index: u128, direction: AnimDirection) {
        let (origin, destination) = match direction {
            AnimDirection::Forward => (self.origin, self.destination),
            AnimDirection::Reverse => (self.destination, self.origin),
        };

        self.inner.advance(index, direction);
        let percentage = self.inner.value();

        let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * percentage).round() as u8;
        let lerp = |a: Rgb, b: Rgb| {
            Rgb::new(
                mix(a.red, b.red),
                mix(a.green, b.green),
                mix(a.blue, b.blue),
            )
        };

        self.value = Theme {
            md: Scheme {
                primary: lerp(origin.md.primary, destination.md.primary),
                on_primary: lerp(origin.md.on_primary, destination.md.on_primary),
                primary_container: lerp(
                    origin.md.primary_container,
                    destination.md.primary_container,
                ),
                on_primary_container: lerp(
                    origin.md.on_primary_container,
                    destination.md.on_primary_container,
                ),
                inverse_primary: lerp(origin.md.inverse_primary, destination.md.inverse_primary),
                primary_fixed: lerp(origin.md.primary_fixed, destination.md.primary_fixed),
                primary_fixed_dim: lerp(
                    origin.md.primary_fixed_dim,
                    destination.md.primary_fixed_dim,
                ),
                on_primary_fixed: lerp(origin.md.on_primary_fixed, destination.md.on_primary_fixed),
                on_primary_fixed_variant: lerp(
                    origin.md.on_primary_fixed_variant,
                    destination.md.on_primary_fixed_variant,
                ),
                secondary: lerp(origin.md.secondary, destination.md.secondary),
                on_secondary: lerp(origin.md.on_secondary, destination.md.on_secondary),
                secondary_container: lerp(
                    origin.md.secondary_container,
                    destination.md.secondary_container,
                ),
                on_secondary_container: lerp(
                    origin.md.on_secondary_container,
                    destination.md.on_secondary_container,
                ),
                secondary_fixed: lerp(origin.md.secondary_fixed, destination.md.secondary_fixed),
                secondary_fixed_dim: lerp(
                    origin.md.secondary_fixed_dim,
                    destination.md.secondary_fixed_dim,
                ),
                on_secondary_fixed: lerp(
                    origin.md.on_secondary_fixed,
                    destination.md.on_secondary_fixed,
                ),
                on_secondary_fixed_variant: lerp(
                    origin.md.on_secondary_fixed_variant,
                    destination.md.on_secondary_fixed_variant,
                ),
                tertiary: lerp(origin.md.tertiary, destination.md.tertiary),
                on_tertiary: lerp(origin.md.on_tertiary, destination.md.on_tertiary),
                tertiary_container: lerp(
                    origin.md.tertiary_container,
                    destination.md.tertiary_container,
                ),
                on_tertiary_container: lerp(
                    origin.md.on_tertiary_container,
                    destination.md.on_tertiary_container,
                ),
                tertiary_fixed: lerp(origin.md.tertiary_fixed, destination.md.tertiary_fixed),
                tertiary_fixed_dim: lerp(
                    origin.md.tertiary_fixed_dim,
                    destination.md.tertiary_fixed_dim,
                ),
                on_tertiary_fixed: lerp(
                    origin.md.on_tertiary_fixed,
                    destination.md.on_tertiary_fixed,
                ),
                on_tertiary_fixed_variant: lerp(
                    origin.md.on_tertiary_fixed_variant,
                    destination.md.on_tertiary_fixed_variant,
                ),
                error: lerp(origin.md.error, destination.md.error),
                on_error: lerp(origin.md.on_error, destination.md.on_error),
                error_container: lerp(origin.md.error_container, destination.md.error_container),
                on_error_container: lerp(
                    origin.md.on_error_container,
                    destination.md.on_error_container,
                ),
                surface_dim: lerp(origin.md.surface_dim, destination.md.surface_dim),
                surface: lerp(origin.md.surface, destination.md.surface),
                surface_tint: lerp(origin.md.surface_tint, destination.md.surface_tint),
                surface_bright: lerp(origin.md.surface_bright, destination.md.surface_bright),
                surface_container_lowest: lerp(
                    origin.md.surface_container_lowest,
                    destination.md.surface_container_lowest,
                ),
                surface_container_low: lerp(
                    origin.md.surface_container_low,
                    destination.md.surface_container_low,
                ),
                surface_container: lerp(
                    origin.md.surface_container,
                    destination.md.surface_container,
                ),
                surface_container_high: lerp(
                    origin.md.surface_container_high,
                    destination.md.surface_container_high,
                ),
                surface_container_highest: lerp(
                    origin.md.surface_container_highest,
                    destination.md.surface_container_highest,
                ),
                on_surface: lerp(origin.md.on_surface, destination.md.on_surface),
                on_surface_variant: lerp(
                    origin.md.on_surface_variant,
                    destination.md.on_surface_variant,
                ),
                outline: lerp(origin.md.outline, destination.md.outline),
                outline_variant: lerp(origin.md.outline_variant, destination.md.outline_variant),
                inverse_surface: lerp(origin.md.inverse_surface, destination.md.inverse_surface),
                inverse_on_surface: lerp(
                    origin.md.inverse_on_surface,
                    destination.md.inverse_on_surface,
                ),
                surface_variant: lerp(origin.md.surface_variant, destination.md.surface_variant),
                background: lerp(origin.md.background, destination.md.background),
                on_background: lerp(origin.md.on_background, destination.md.on_background),
                shadow: lerp(origin.md.shadow, destination.md.shadow),
                scrim: lerp(origin.md.scrim, destination.md.scrim),
            },
            stoat: StoatScheme {
                presence_online: Color::lerp(
                    origin.stoat.presence_online,
                    destination.stoat.presence_online,
                    percentage,
                ),
                presence_idle: Color::lerp(
                    origin.stoat.presence_idle,
                    destination.stoat.presence_idle,
                    percentage,
                ),
                presence_busy: Color::lerp(
                    origin.stoat.presence_busy,
                    destination.stoat.presence_busy,
                    percentage,
                ),
                presence_focus: Color::lerp(
                    origin.stoat.presence_focus,
                    destination.stoat.presence_focus,
                    percentage,
                ),
                presence_invisible: Color::lerp(
                    origin.stoat.presence_invisible,
                    destination.stoat.presence_invisible,
                    percentage,
                ),
            },
        }
    }

    fn finish(&mut self, direction: AnimDirection) {
        self.advance(500, direction);
    }

    fn into_reversed(self) -> Self {
        Self {
            origin: self.destination,
            destination: self.origin,
            ..self
        }
    }
}

impl ReadAnimatedValue for AnimTheme {
    type Output = Theme;
    fn value(&self) -> Self::Output {
        self.value()
    }
}
