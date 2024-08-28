use std::ops::Div;
use std::time::Duration;

#[cfg(feature = "serde")]
use serde::{de::Visitor, Deserialize, Serialize};

use super::AnimationCurve;

/// Delta by which we check durations and values.
const DELTA: f64 = 0.001;

/// Spring-based curve.
///
/// Implementation converted from the following:
/// - https://github.com/GNOME/libadwaita/blob/main/src/adw-spring-animation.c
/// - https://github.com/GNOME/libadwaita/blob/main/src/adw-spring-params.c
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub struct Curve {
    initial_velocity: f64,
    clamp: bool,
    // spring parameters
    mass: f64,
    damping: f64,
    stiffness: f64,
    epsilon: f64, /* this is also called precision in places like react spring
                   * unless you are really nitty gritty about your animations you wont touch
                   * this */
}

impl Into<AnimationCurve> for Curve {
    fn into(self) -> AnimationCurve {
        AnimationCurve::Spring(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Curve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "kebab-case")]
        enum Field {
            InitialVelocity,
            Clamp,
            Mass,
            DampingRatio,
            Stiffness,
            Epsilon,
        }

        struct AnimationVisitor;
        impl<'de> Visitor<'de> for AnimationVisitor {
            type Value = Curve;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Animation")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut initial_velocity = None;
                let mut clamp = None;
                let mut mass = None;
                let mut damping_ratio = None;
                let mut stiffness = None;
                let mut epsilon = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::InitialVelocity => {
                            if initial_velocity.is_some() {
                                return Err(serde::de::Error::duplicate_field("velocity"));
                            }
                            initial_velocity = Some(map.next_value()?);
                        }
                        Field::Clamp => {
                            if clamp.is_some() {
                                return Err(serde::de::Error::duplicate_field("clamp"));
                            }
                            clamp = Some(map.next_value()?);
                        }
                        Field::Mass => {
                            if mass.is_some() {
                                return Err(serde::de::Error::duplicate_field("mass"));
                            }
                            mass = Some(map.next_value()?);
                        }
                        Field::DampingRatio => {
                            if damping_ratio.is_some() {
                                return Err(serde::de::Error::duplicate_field("damping_ratio"));
                            }
                            damping_ratio = Some(map.next_value()?);
                        }
                        Field::Stiffness => {
                            if stiffness.is_some() {
                                return Err(serde::de::Error::duplicate_field("stiffness"));
                            }
                            stiffness = Some(map.next_value()?);
                        }
                        Field::Epsilon => {
                            if epsilon.is_some() {
                                return Err(serde::de::Error::duplicate_field("epsilon"));
                            }
                            epsilon = Some(map.next_value()?);
                        }
                    }
                }

                let initial_velocity =
                    initial_velocity.ok_or_else(|| serde::de::Error::missing_field("velocity"))?;
                let clamp = clamp.ok_or_else(|| serde::de::Error::missing_field("clamp"))?;
                let mass = mass.ok_or_else(|| serde::de::Error::missing_field("mass"))?;
                let damping_ratio: f64 = damping_ratio
                    .ok_or_else(|| serde::de::Error::missing_field("damping_ratio"))?;
                let stiffness =
                    stiffness.ok_or_else(|| serde::de::Error::missing_field("stiffness"))?;
                let epsilon = epsilon.unwrap_or(0.0001);
                // Calculate our damping based on the damping ratio.
                // Thats how libadwaita does i
                let critical_damping = 2.0 * f64::from(mass * stiffness).sqrt();
                let damping = damping_ratio * critical_damping;
                Ok(Curve {
                    initial_velocity,
                    clamp,
                    mass,
                    damping,
                    stiffness,
                    epsilon,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "initial_velocity",
            "clamp",
            "mass",
            "damping-ratio",
            "stiffness",
            "epsilon",
        ];
        deserializer.deserialize_struct("Animation", FIELDS, AnimationVisitor)
    }
}

impl Curve {
    /// Create a new spring animation curve.
    pub fn new(
        initial_velocity: f64,
        clamp: bool,
        mass: f64,
        damping: f64,
        stiffness: f64,
    ) -> Self {
        Self {
            initial_velocity,
            clamp,
            mass,
            damping,
            stiffness,
            epsilon: 0.0001,
        }
    }

    /// Change whether the spring clamps in place.
    ///
    /// Clamping the spring will make it not overshoot, but instead stopping at when it reaches the
    /// end value.
    pub fn with_clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Change the spring mass in place.
    ///
    /// The mass affects the spring's inertia, the higher the mass, the slower and "smoother" the
    /// spring will be
    pub fn with_mass(mut self, mass: f64) -> Self {
        self.mass = mass;
        self
    }

    /// Change the spring damping ratio in place.
    ///
    /// The damping ratio affects the spring's oscillations:
    /// * 0.0 < `damping_ratio` < 1.0 = Animation is underdamped, it does bounce and oscillates
    ///   around the target value before settling
    /// * damping_ratio == 1.0 = Animation is critically damped, it reaches the target value as
    ///   quickly as possible
    /// * damping_ratio > 1.0 = Animation is overdamped, it reaches the target value slowly without
    ///   oscillating around it.
    pub fn with_damping_ratio(mut self, damping_ratio: f64) -> Self {
        // Calculate our damping based on the damping ratio.
        // Thats how libadwaita does i
        let critical_damping = 2.0 * (self.mass * self.stiffness).sqrt();
        self.damping = damping_ratio * critical_damping;
        self
    }

    /// Change the spring stiffness in place.
    ///
    /// The stiffness affects how strong the force exerted by the spring is.
    pub fn with_stiffness(mut self, stiffness: f64) -> Self {
        self.stiffness = stiffness;
        self
    }

    /// Change the spring's epsilon in place.
    ///
    /// It determines how precise the spring oscillations and simulation should be. If its too
    /// small, the animation will take a long time before setting to the target value **exactly**
    pub fn with_epsilon(mut self, epsilon: f64) -> Self {
        self.epsilon = epsilon;
        self
    }

    /// Get the duration of this spring animation.
    pub fn duration(&self) -> Duration {
        let beta = self.damping / (2.0 * self.mass);

        // Spring never ends, too bad
        if beta < 0.0 || beta.abs() <= f64::EPSILON {
            return Duration::MAX;
        }

        if self.clamp {
            return self.first_zero();
        }

        let omega0 = (self.stiffness / self.mass).sqrt();
        // As a first anstaz for the overclamped solution,
        // and a general estimation for the oscillating ones
        // we take the value of the envelope when its below epsilon.
        let mut x0 = -self.epsilon.ln() / beta;

        // Using f64::EPSILON is too small for this comparaison
        // f32::EPSILON even though it's doubles.
        if (beta - omega0).abs() < f64::from(f32::EPSILON) || beta < omega0 {
            return Duration::from_secs_f64(x0);
        }

        // Since the overdamped solution decays way slower than the envelope
        // we need to use the value of the oscillation itself.
        // Netwon's root finding method is a good candidate in this case:
        // https://en.wikipedia.org/wiki/Newton%27s_method
        let mut y0 = self.oscillate(x0);
        let m = (self.oscillate(x0 + DELTA) - y0) / DELTA;

        let mut x1 = (1.0 - y0 + m * x0).div(m);
        let mut y1 = self.oscillate(x1);
        let mut i = 1;

        while (1.0 - y1).abs() > self.epsilon {
            if i > 5000 {
                // too much iterations, just abandon.
                return Duration::ZERO;
            }

            x0 = x1;
            y0 = y1;

            let m = (self.oscillate(x0 + DELTA) - y0) / DELTA;
            x1 = (1.0 - y0 + m * x0).div(m);
            y1 = self.oscillate(x1);
            i += 1;
        }

        Duration::from_secs_f64(x1)
    }

    /// Get the first 0 value of this animation.
    pub fn first_zero(&self) -> Duration {
        // The first frame is not that important and we avoid finding the trivial 0
        // for in-place animations.
        let mut x = 0.001;
        let mut y = self.oscillate(x);

        // A difference from libadwaita is that we don't check if the start and end are greater
        // than f64::EPSILON since they are constant (0.0 and 1.0 respectively)
        while (1.0 - y > self.epsilon) || (y - 1.0 > self.epsilon) {
            if x > 200.0
            /* 20000 (max iters) * 0.001 (1ms) */
            {
                // To much iterations, just give up.
                return Duration::ZERO;
            }

            x += DELTA;
            y = self.oscillate(x)
        }

        Duration::from_secs_f64(x)
    }

    /// Get a value inside for a `t` seconds passed since the spring started
    pub fn oscillate(&self, t: f64) -> f64 {
        let v0 = self.initial_velocity;
        let x0 = -1.0; // x0 is start - end, but start is always 0.0, soo.
        let end = 1.0;

        let beta = self.damping / (2.0 * self.mass);
        let omega0 = (self.stiffness / self.mass).sqrt();
        let envelope = (-beta * t).exp();

        // Solutions of the differential equation take the form of:
        //    C1 * e ^ (lambda_1 * x)
        //    C2 * e ^ (lambda_2 * x)

        // Using f64::EPSILON is too small for this comparaison
        // f32::EPSILON even though it's doubles.
        if (beta - omega0).abs() <= f64::from(f32::EPSILON) {
            // First possibility: animation is critically damped.
            end + envelope * (x0 + (beta * x0 + v0) * t)
        } else if beta < omega0 {
            // Second possibility: animation is underdamped.
            let omega1 = (omega0.powf(2.0) - beta.powf(2.0)).sqrt();
            end + envelope
                * (x0 * (omega1 * t).cos() + ((beta + x0 * v0) / omega1) * (omega1 * t).sin())
        } else if beta > omega0 {
            // Third possibility: animation is overmapped.
            let omega2 = (beta.powf(2.0) - omega0.powf(2.0)).sqrt();
            end + envelope
                * (x0 * (omega2 * t).cosh() + ((beta * x0 + v0) / omega2) * (omega2 * t).sinh())
        } else {
            unreachable!("Something really wrong happened with spring animations...");
        }
    }
}
