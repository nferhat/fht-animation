use std::time::Duration;

pub mod curve;
#[cfg(feature = "iced")]
pub mod iced;

pub use curve::cubic::Curve as CubicCurve;
pub use curve::spring::Curve as SpringCurve;
pub use curve::AnimationCurve;

/// A type that can be animated using [`Animation`]
///
/// This trait is intentionally not derivable, its up to you to actually implement scaling of your
/// custom struct members accordingly.
///
/// You are also responsible to manage variable overflow, if applicable.
pub trait Animable: Sized + Clone {
    /// Do a linear interpolation between the start and end of this type with a given `progress`.
    ///
    /// `progress` may overshoot/undershoot out of `[0.0, 1.0]` in the case of spring animations,
    /// for example. It is up to you to handle overflows and edge cases with your types.
    fn lerp(start: &Self, end: &Self, progress: f64) -> Self;
}

// Basic impls for rust numeric types
macro_rules! rust_builtin_impl {
    ($t:ty) => {
        impl Animable for $t {
            fn lerp(start: &Self, end: &Self, progress: f64) -> Self {
                (((*end - *start) as f64) * progress) as $t + *start
            }
        }
    };
}

// TODO: Figure out for unsigned types, since scale can sometimes be less than one
// (for example in the context of spring animations)
rust_builtin_impl!(i8);
rust_builtin_impl!(i16);
rust_builtin_impl!(i32);
rust_builtin_impl!(i64);
rust_builtin_impl!(f32);
rust_builtin_impl!(f64);

impl<T: Animable, const N: usize> Animable for [T; N] {
    fn lerp(start: &Self, end: &Self, progress: f64) -> Self {
        let Ok(ret) = start
            .iter()
            .zip(end)
            .map(|(start, end)| T::lerp(start, end, progress))
            .collect::<Vec<_>>()
            .try_into()
        else {
            unreachable!("vector len should match slice from iterator")
        };

        ret
    }
}

/// The state of an [`Animation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum AnimationState {
    /// The animation is running.
    Running,
    /// The animation is paused.
    ///
    /// If the animation is in this state, calling [`Animtion::set_current_time`] will not update
    /// the animation's current value. Instead the animation duration will increase for the time
    /// its paused.
    Paused,
}

impl std::ops::Neg for AnimationState {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Paused => Self::Running,
            Self::Running => Self::Paused,
        }
    }
}

/// An animatable variable, with a `start` and `end`.
///
/// This struct by itself does nothing, you should be calling [`Animation::tick`] on every frame
/// that you are using the animation with in order to update [`Animation::value`].
///
/// See [`Animatable`]
#[derive(Clone, Debug)]
pub struct Animation<T: Animable> {
    pub start: T,
    pub end: T,
    // We update the current value when we call [`Animation::tick`] so that calling
    // [`Animation::current_value`] is very very cheap
    current_value: T,

    // State and curve
    state: AnimationState,
    curve: AnimationCurve,

    // Animaton timing
    // started_at and last_tick = durations since unix epoch
    started_at: Duration,
    last_tick: Duration,
    duration: Duration,
}

impl<T: Animable> Animation<T> {
    /// Creates a new animation with given parameters.
    ///
    /// This returns None if `start == end`
    pub fn new(start: T, end: T, duration: Duration) -> Self {
        let started_at = get_monotonic_time();
        let current_value = start.clone();

        Self {
            start,
            end,
            current_value,

            state: AnimationState::Running,

            curve: AnimationCurve::default(),
            started_at,
            last_tick: started_at,
            duration,
        }
    }

    /// Change the animation state in-place.
    pub fn with_state(mut self, state: AnimationState) -> Self {
        self.state = state;
        self
    }

    /// Set the animation state.
    pub fn set_state(&mut self, state: AnimationState) {
        self.state = state;
    }

    /// Change the animation duration in-place.
    ///
    /// NOTE: If you are using a `Spring` curve, this will change absolutely nothing, as the
    /// duration for springs is determined by their parameters instead.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        if !matches!(self.curve, AnimationCurve::Spring(_)) {
            self.duration = duration;
        }
        self
    }

    /// Set the animation duration.
    ///
    /// NOTE: If you are using a `Spring` curve, this will change absolutely nothing, as the
    /// duration for springs is determined by their parameters instead.
    pub fn set_duration(&mut self, duration: Duration) {
        if !matches!(self.curve, AnimationCurve::Spring(_)) {
            self.duration = duration;
        }
    }

    /// Change the animation curve in-place.
    pub fn with_curve(mut self, curve: impl Into<AnimationCurve>) -> Self {
        let curve = curve.into();
        if let AnimationCurve::Spring(spring) = &curve {
            self.duration = spring.duration();
        }
        self.curve = curve;
        self
    }

    /// Set the animation curve.
    pub fn set_curve(&mut self, curve: impl Into<AnimationCurve>) {
        let curve = curve.into();
        if let AnimationCurve::Spring(spring) = &curve {
            self.duration = spring.duration();
        }
        self.curve = curve;
    }

    /// Restart the time state of the animation.
    pub fn restart(&mut self) {
        self.last_tick = get_monotonic_time();
        self.started_at = self.last_tick;
    }

    /// Tick the animation at a given [`Duration`], relative to `UNIX_EPOCH`
    ///
    /// It is assumed that the value from `now` is coming from a monotonically increasing system
    /// clock, for example libc's `clock_gettime(CLOCK_MONOTONIC)` on UNIX.
    pub fn tick(&mut self, now: Duration) {
        if self.state == AnimationState::Paused {
            // This is adapted from slowdown animation code inside niri (yalter/niri)
            // But, to pause an animation, ANIMATION_SLOWDOWN must approach +inf, so adjusted_delta
            // (in the original niri code) becomes 0 (with limits)
            //
            // Since `now` is meant to be a monotonically increasing clock, we instead adjust the
            // start_time to match our offset.
            if self.last_tick <= now {
                let delta = now - self.last_tick;
                self.started_at += delta;
            } else {
                let delta = self.last_tick - now;
                self.started_at -= delta;
            }

            self.last_tick = now;
            // And we dont need to update the value needlessly.
            // It will be when we unpause the animation
            return;
        }

        let elapsed = (now - self.started_at).as_secs_f64();
        let total = self.duration.as_secs_f64();
        self.last_tick = now;

        self.current_value = match &mut self.curve {
            AnimationCurve::Simple(easing) => {
                // keyframe's easing function take an x value between [0.0, 1.0], so normalize out
                // x value to these.
                let x = (elapsed / total).clamp(0., 1.);
                let progress = easing.y(x);
                T::lerp(&self.start, &self.end, progress)
            }
            AnimationCurve::Cubic(cubic) => {
                // Cubic animations also take in X between [0.0, 1.0] and outputs a progress in
                // [0.0, 1.0]
                let x = (elapsed / total).clamp(0., 1.);
                let progress = cubic.y(x);
                T::lerp(&self.start, &self.end, progress)
            }
            AnimationCurve::Spring(spring) => {
                let progress = spring.oscillate(elapsed);
                T::lerp(&self.start, &self.end, progress)
            }
        };
    }

    /// Check whether the animation is finished or not.
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.last_tick - self.started_at >= self.duration
    }

    /// Get the current progress of this animation in time, from `0.0` to `1.0`.
    #[inline]
    pub fn time_progress(&self) -> f64 {
        let elapsed = (self.last_tick - self.started_at).as_secs_f64();
        let total = self.duration.as_secs_f64();
        (elapsed / total).clamp(0., 1.)
    }

    /// Get the last calculated value from [`Animation::tick`].
    #[inline]
    pub fn value(&self) -> &T {
        &self.current_value
    }
}

/// Get the monotonic time to tick an [`Animation`]
///
/// The duration value is the duration since UNIX_EPOCH
pub fn get_monotonic_time() -> Duration {
    let time = rustix::time::clock_gettime(rustix::time::ClockId::Monotonic);
    Duration::new(time.tv_sec as u64, time.tv_nsec as u32)
}
