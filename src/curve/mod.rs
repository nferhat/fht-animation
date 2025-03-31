use std::f64::consts::PI;

use keyframe::EasingFunction;

pub mod cubic;
pub mod spring;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case", untagged))]
pub enum AnimationCurve {
    /// Use a preset easing provided by [`keyframe`]
    Simple(Easing),
    /// Use a spring-based animation.
    Spring(spring::Curve),
    /// Use a custom cubic animation with two control points:
    Cubic(cubic::Curve),
}

impl Default for AnimationCurve {
    fn default() -> Self {
        Self::Simple(Easing::default())
    }
}

/// Wrapper enum including all the easings [`keyframe`] provides.
#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum Easing {
    EaseIn,
    EaseInCubic,
    EaseInOut,
    EaseInOutCubic,
    EaseInOutQuart,
    EaseInOutQuint,
    EaseInQuad,
    EaseInQuart,
    EaseInQuint,
    EaseOut,
    EaseOutCubic,
    EaseOutQuad,
    EaseOutQuart,
    EaseOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    #[default]
    Linear,
}

impl Into<AnimationCurve> for Easing {
    fn into(self) -> AnimationCurve {
        AnimationCurve::Simple(self)
    }
}

impl Easing {
    /// Get the Y value at a given X coordinate, assuming that x is included in [0.0, 1.0]
    pub fn y(&self, x: f64) -> f64 {
        match self {
            Self::EaseIn => keyframe::functions::EaseIn.y(x),
            Self::EaseInCubic => keyframe::functions::EaseInCubic.y(x),
            Self::EaseInOut => keyframe::functions::EaseInOut.y(x),
            Self::EaseInOutCubic => keyframe::functions::EaseInOutCubic.y(x),
            Self::EaseInOutQuart => keyframe::functions::EaseInOutQuart.y(x),
            Self::EaseInOutQuint => keyframe::functions::EaseInOutQuint.y(x),
            Self::EaseInQuad => keyframe::functions::EaseInQuad.y(x),
            Self::EaseInQuart => keyframe::functions::EaseInQuart.y(x),
            Self::EaseInQuint => keyframe::functions::EaseInQuint.y(x),
            Self::EaseOut => keyframe::functions::EaseOut.y(x),
            Self::EaseOutCubic => keyframe::functions::EaseOutCubic.y(x),
            Self::EaseOutQuad => keyframe::functions::EaseOutQuad.y(x),
            Self::EaseOutQuart => keyframe::functions::EaseOutQuart.y(x),
            Self::EaseOutQuint => keyframe::functions::EaseOutQuint.y(x),
            // All other functions from https://easings.net
            Easing::EaseInSine => 1.0 - f64::cos((x * PI) / 2.0),
            Easing::EaseOutSine => 1.0 - f64::cos((x * PI) / 2.0),
            Easing::EaseInOutSine => -(f64::cos(PI * x) - 1.0) / 2.0,
            Easing::EaseInCirc => 1.0 - f64::sqrt(1.0 - (x - 1.0).powi(2)),
            Easing::EaseOutCirc => f64::sqrt(1.0 - (x - 1.0).powi(2)),
            Easing::EaseInOutCirc => {
                if x < 0.5 {
                    (1.0 - f64::sqrt(1.0 - (2.0 * x).powi(2))) / 2.0
                } else {
                    (1.0 + f64::sqrt(1.0 - (-2.0 * x + 2.0).powi(2))) / 2.0
                }
            }
            Easing::EaseInElastic => {
                const C4: f64 = (2.0 * PI) / 3.0;
                match x {
                    0.0 => 0.0,
                    1.0 => 1.0,
                    _ => 2f64.powf(10.0 * x - 10.0) * f64::sin((x * 10.0 - 10.75) * C4),
                }
            }
            Easing::EaseOutElastic => {
                const C4: f64 = (2.0 * PI) / 3.0;
                match x {
                    0.0 => 0.0,
                    1.0 => 1.0,
                    _ => 2f64.powf(-10.0 * x) * f64::sin((x * 10.0 - 0.75) * C4) + 1.0,
                }
            }
            Easing::EaseInOutElastic => {
                const C5: f64 = 2.0 * PI / 4.5;

                match x {
                    x if x == 0.0 => 0.0,
                    x if x == 1.0 => 1.0,
                    x if x < 0.5 => {
                        -(2.0f64.powf(20.0 * x - 10.0) * ((20.0 * x - 11.125) * C5).sin()) / 2.0
                    }
                    _ => {
                        (2.0f64.powf(-20.0 * x + 10.0) * ((20.0 * x - 11.125) * C5).sin()) / 2.0
                            + 1.0
                    }
                }
            }
            Easing::EaseInExpo => match x {
                0.0 => 0.0,
                _ => 2.0f64.powf(10.0 * x - 10.0),
            },
            Easing::EaseOutExpo => match x {
                1.0 => 1.0,
                _ => 1.0 - 2.0f64.powf(-10.0 * x),
            },
            Easing::EaseInOutExpo => match x {
                0.0 => 0.0,
                1.0 => 1.0,
                x if x < 0.5 => 2.0f64.powf(20.0 * x - 10.0) / 2.0,
                _ => (2.0 - 2.0f64.powf(-20.0 * x + 10.0)) / 2.0,
            },
            Easing::EaseInBack => {
                const C1: f64 = 1.70158;
                const C3: f64 = C1 + 1.0;
                C3 * x.powi(3) - C1 * x.powi(2)
            }
            Easing::EaseOutBack => {
                const C1: f64 = 1.70158;
                const C3: f64 = C1 + 1.0;
                1.0 + C3 * (x - 1.0).powi(3) + C1 * (x - 1.0).powi(2)
            }
            Easing::EaseInOutBack => {
                const C1: f64 = 1.70158;
                const C2: f64 = C1 * 1.525;
                match x {
                    x if x < 0.5 => ((2.0 * x).powi(2) * ((C2 + 1.0) * 2.0 * x - C2)) / 2.0,
                    _ => {
                        ((2.0 * x - 2.0).powi(2) * ((C2 + 1.0) * (x * 2.0 - 2.0) + C2) + 2.0) / 2.0
                    }
                }
            }
            Easing::EaseInBounce => 1.0 - Self::EaseOutBounce.y(1.0 - x),
            Easing::EaseOutBounce => {
                const N1: f64 = 7.5625;
                const D1: f64 = 2.75;
                match x {
                    x if x < 1.0 / D1 => N1 * x * x,
                    mut x if x < 2.0 / D1 => {
                        x -= 1.5 / D1;
                        N1 * x * x + 0.75
                    }
                    mut x if x < 2.5 / D1 => {
                        x -= 2.25 / D1;
                        N1 * x * x + 0.9375
                    }
                    mut x => {
                        x -= 2.625 / D1;
                        N1 * x * x + 0.984375
                    }
                }
            }
            Easing::EaseInOutBounce => match x {
                x if x < 0.5 => (1.0 - Self::EaseOutBounce.y(1.0 - 2.0 * x)) / 2.0,
                _ => (1.0 + Self::EaseOutBounce.y(2.0 * x - 1.0)) / 2.0,
            },
            Self::Linear => keyframe::functions::Linear.y(x),
        }
    }
}
