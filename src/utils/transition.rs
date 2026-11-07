use std::sync::LazyLock;

use bezier_easing::{BezierEasing, bezier_easing};
use freya::animation::Function;

static EASE_IN_OUT: LazyLock<BezierEasing<f32>> =
    LazyLock::new(|| bezier_easing(0.42, 0., 0.58, 1.).unwrap());
static EASE: LazyLock<BezierEasing<f32>> =
    LazyLock::new(|| bezier_easing(0.25, 0.1, 0.25, 1.).unwrap());
static SETINGS_EASE: LazyLock<BezierEasing<f32>> =
    LazyLock::new(|| bezier_easing(0.17, 0.67, 0.58, 0.98).unwrap());
static TEXT_ENTRY_EASE: LazyLock<BezierEasing<f32>> =
    LazyLock::new(|| bezier_easing(0.2, 0., 0., 1.).unwrap());
static DROPDOWN_EASE: LazyLock<BezierEasing<f32>> =
    LazyLock::new(|| bezier_easing(0.87, 0., 0.13, 1.).unwrap());

/// 100ms
pub fn fast_func() -> Function {
    Function::Fn(|i, _| EASE_IN_OUT.sample(i))
}

/// 200ms
pub fn medium_func() -> Function {
    Function::Fn(|i, _| EASE.sample(i))
}

/// 200ms
pub fn settings_func() -> Function {
    Function::Fn(|i, _| SETINGS_EASE.sample(i))
}

/// 200ms
pub fn text_entry_func() -> Function {
    Function::Fn(|i, _| TEXT_ENTRY_EASE.sample(i))
}

/// 200ms
pub fn dropdown_func() -> Function {
    Function::Fn(|i, _| DROPDOWN_EASE.sample(i))
}
