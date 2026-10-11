use std::f32::consts::PI;

use chumsky::{
    prelude::*,
    text::{digits, ident, int, keyword, whitespace},
};
use freya::prelude::*;
use freya_engine::prelude::{RuntimeEffect, Shader};

// angle of 0. is down

fn float_parser<'a>() -> impl Parser<'a, &'a str, f32> {
    let digits = digits(10).to_slice();

    let frac = just('.').then(digits);

    just('-')
        .or_not()
        .then(int(10))
        .then(frac.or_not())
        .to_slice()
        .map(|s: &str| s.parse().unwrap())
        .boxed()
}

fn percentage<'a>() -> impl Parser<'a, &'a str, u8> {
    int::<&str, _>(10)
        .then_ignore(just("%"))
        .filter_map(|int| int.parse::<u8>().ok())
        .boxed()
}

fn named_color_parser<'a>() -> impl Parser<'a, &'a str, Color> {
    let uncased = |keyword: &'static str, color| {
        ident::<&str, _>()
            .filter(move |v| &v.to_lowercase() == keyword)
            .to(color)
            .boxed()
    };

    choice((
        uncased("red", Color::RED),
        uncased("green", Color::GREEN),
        uncased("blue", Color::BLUE),
        // TODO: more cases
    ))
}

fn direction_parser<'a>() -> impl Parser<'a, &'a str, Direction> {
    let direction = choice((
        keyword("top"),
        keyword("bottom"),
        keyword("left"),
        keyword("right"),
    ))
    .to_slice()
    .padded();

    just("to")
        .ignore_then(direction.clone().then(direction.or_not()))
        .map(|(a, b)| match (a, b) {
            ("bottom", None) => Direction::Bottom,
            ("bottom", Some("left")) | ("left", Some("bottom")) => Direction::BottomLeft,
            ("left", None) => Direction::Left,
            ("top", Some("left")) | ("left", Some("top")) => Direction::TopLeft,
            ("top", None) => Direction::Top,
            ("top", Some("right")) | ("right", Some("top")) => Direction::TopRight,
            ("right", None) => Direction::Right,
            ("bottom", Some("right")) | ("right", Some("bottom")) => Direction::BottomRight,
            _ => unreachable!(),
        })
}

fn base_angle_parser<'a>() -> impl Parser<'a, &'a str, f32> {
    let unit = choice((
        keyword("deg"),
        keyword("grad"),
        keyword("rad"),
        keyword("turn"),
    ))
    .to_slice();

    float_parser()
        .then(unit.padded())
        .map(|(amount, unit)| match unit {
            "deg" => amount,
            "grad" => amount.atan() * (180. / PI),
            "rad" => amount * (180. / PI),
            "turn" => amount * 360.,
            _ => unreachable!(),
        } + 180.)
}

fn angle_parser<'a>() -> impl Parser<'a, &'a str, Direction> {
    direction_parser().or(base_angle_parser().map(Direction::Angle))
}

fn hex_parser<'a>() -> impl Parser<'a, &'a str, Color> {
    just('#')
        .ignore_then(digits(16).to_slice())
        .filter_map(|hex: &str| {
            let (r, g, b, a) = match hex.len() {
                3 => (
                    u8::from_str_radix(&hex[0..1], 16).ok()?,
                    u8::from_str_radix(&hex[1..2], 16).ok()?,
                    u8::from_str_radix(&hex[2..3], 16).ok()?,
                    0xFF,
                ),
                4 => (
                    u8::from_str_radix(&hex[0..1], 16).ok()?,
                    u8::from_str_radix(&hex[1..2], 16).ok()?,
                    u8::from_str_radix(&hex[2..3], 16).ok()?,
                    u8::from_str_radix(&hex[3..4], 16).ok()?,
                ),
                6 => (
                    u8::from_str_radix(&hex[0..2], 16).ok()?,
                    u8::from_str_radix(&hex[2..4], 16).ok()?,
                    u8::from_str_radix(&hex[4..6], 16).ok()?,
                    0xFF,
                ),
                8 => (
                    u8::from_str_radix(&hex[0..2], 16).ok()?,
                    u8::from_str_radix(&hex[2..4], 16).ok()?,
                    u8::from_str_radix(&hex[4..6], 16).ok()?,
                    u8::from_str_radix(&hex[6..8], 16).ok()?,
                ),
                _ => return None,
            };

            Some(Color::from_argb(a, r, g, b))
        })
        .boxed()
}

fn rgb_parser<'a>() -> impl Parser<'a, &'a str, Color> {
    let number = int::<&str, _>(10)
        .filter_map(|int| int.parse::<u8>().ok())
        .boxed();

    let percentage = percentage()
        .map(|f| (255. * (f as f32 / 100.)) as u8)
        .boxed();

    let value = percentage.or(number).boxed();

    let modern = value
        .clone()
        .separated_by(whitespace())
        .exactly(3)
        .collect_exactly::<[_; 3]>()
        .boxed()
        .then(just('/').padded().ignore_then(value.clone()).or_not());

    let legacy = value
        .clone()
        .padded()
        .separated_by(just(','))
        .exactly(3)
        .collect_exactly::<[_; 3]>()
        .boxed()
        .then(just(',').padded().ignore_then(value.clone()).or_not());

    let inner = modern
        .or(legacy)
        .map(|([r, g, b], a)| Color::from_argb(a.unwrap_or(255), r, g, b));

    choice((keyword("rgba"), keyword("rgb")))
        .ignore_then(inner.padded().delimited_by(just('('), just(')')))
}

fn inner_color_parser<'a>() -> impl Parser<'a, &'a str, Color> {
    choice((named_color_parser(), hex_parser(), rgb_parser()))
}

fn stop_parser<'a>() -> impl Parser<'a, &'a str, (Color, Option<f32>)> {
    inner_color_parser().padded().then(
        percentage()
            .then_ignore(percentage().or_not())
            .map(|percentage| percentage as f32)
            .or_not(),
    )
}

fn linear_gradient_parser<'a>() -> impl Parser<'a, &'a str, Fill> {
    let angle = angle_parser().padded().then_ignore(just(','));
    let stops = stop_parser()
        .padded()
        .separated_by(just(','))
        .allow_trailing()
        .collect::<Vec<_>>();

    just("linear-gradient")
        .ignore_then(
            angle
                .or_not()
                .then(stops)
                .delimited_by(just('('), just(')')),
        )
        .map(|(angle, stops)| {
            let mut gradient = LinearGradient::new().angle(0.);

            if stops.iter().all(|(_, p)| p.is_some()) {
                for (color, percentage) in stops {
                    gradient = gradient.stop((color, percentage.unwrap()))
                }
            } else {
                let incr = 100. / (stops.len() - 1) as f32;

                for (i, (color, _)) in stops.into_iter().enumerate() {
                    gradient = gradient.stop((color, incr * i as f32))
                }
            }

            generate_linear_shader(gradient, angle.unwrap_or_else(|| Direction::Bottom))
        })
        .boxed()
}

fn fill_parser<'a>() -> impl Parser<'a, &'a str, Fill> {
    choice((
        inner_color_parser().map(Fill::Color),
        linear_gradient_parser(),
    ))
}

pub fn parse_fill(input: &str) -> Option<Fill> {
    fill_parser().parse(input).into_result().ok()
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    Angle(f32),
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
}

struct Provider(LinearGradient, Direction);

impl ShaderProvider for Provider {
    fn prepare_shader(&self, _effect: &RuntimeEffect, bounds: Area) -> Option<Shader> {
        let angle = match &self.1 {
            Direction::Angle(angle) => *angle,
            Direction::Bottom => 0.,
            Direction::BottomLeft => (bounds.width() / bounds.height()).atan() * (180. / PI),
            Direction::Left => 90.,
            Direction::TopLeft => 90. + ((bounds.height() / bounds.width()).atan() * (180. / PI)),
            Direction::Top => 180.,
            Direction::TopRight => 180. + ((bounds.height() / bounds.width()).atan() * (180. / PI)),
            Direction::Right => 270.,
            Direction::BottomRight => {
                270. + ((bounds.width() / bounds.height()).atan() * (180. / PI))
            }
        };

        self.0.clone().angle(angle).prepare_shader(bounds)
    }
}

const FAKE_SHADER: &str = "float4 main(vec2 inCoords) { return float4(0.0, 0.0, 0.0, 0.0); }";

thread_local! {
    static S: RuntimeEffect = RuntimeEffect::make_for_shader(FAKE_SHADER, None).unwrap();
}

fn generate_linear_shader(gradient: LinearGradient, direction: Direction) -> Fill {
    let effect = S.with(|e| e.clone());

    Fill::Shader(Box::new(ShaderFill::from_provider(
        FAKE_SHADER,
        effect,
        Provider(gradient, direction),
    )))
}
