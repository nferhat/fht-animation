# fht-animation

An animation library for `fht` projects.

> This is very experimental...

## Rationale

I already found different animation libraries for both [Iced](https://github.com/iced-rs) and managed
to implement something working with [Smithay](https://github.com/smithay) for my compositor.

The issue is what I found for iced-rs do no match what I implemented (not only in terms of settings,
but features and behaviour), so instead of banging my head with existing libraries, why not write my own?

So, this crate is basically the animation code from my compositor but put independently, with some
helpers.

## Features

- `serde`: Enable serializing animation types using [`serde`](https://github.com/serde-rs)
- `iced`: Enable animation support for types from [`Iced`](https://github/iced-rs/iced)

Currently supported crates

## Features

- Three types of curves:
    * `Simple` curves, for easings provided by [`keyframe`](https://docs.rs/keyframe/latest/keyframe/).
    * `Cubic` curves, with two control points (first and last are forced to `(0,0)` and `(1,1)`), implementation from [`Hyprland`](https://github.com/hyprwm/Hyprland/blob/main/src/helpers/BezierCurve.cpp).
    * `Spring` curves, implementation from [`libadwaita`](https://github.com/GNOME/libadwaita/blob/main/src/adw-spring-animation.c).

- [Iced](https://github.com/iced-rs) support, via stateful animations.

- [Serialization](https://github.com/serde-rs) support, if you have config files for example
