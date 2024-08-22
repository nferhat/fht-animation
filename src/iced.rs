//! A widget that helps you animate a value from your state.
//!
//! Your value start, end, and animation curve should be represented using
//! [`Animation<T>`](crate::Animation)
//!
//! This widget will emit [`AnimationEvent`] events that you can transform into messages for your
//! application to handle using [`AnimationWidget::on_update`]. When handling those events, it is
//! **your** responsiblity for them to reach the correct animation, through your custom message.
//!
//! ```rust
//! use fht_animation::{iced::AnimationWidget, Animation, AnimationCurve};
//! use iced::widget::{text}
//!
//! struct State {
//!     value: Animation<f64>,
//! }
//!
//! enum Message {
//!     UpdateValue(AnimationEvent)
//!     // ...
//! }
//!
//! impl State {
//!     fn update(&mut self, message: Message) {
//!         match message {
//!             Message::UpdateValue(event) => self.value.update(event),
//!             // ...
//!         }
//!     }
//!
//!     fn view(&self) -> iced::Element<Message> {
//!         Animation::new(&self.value, text(format!("The value is: {}", self.value.value())))
//!             .on_update(Message::UpdateValue)
//!             .into()
//!     }
//! }
//! ```

use std::time::Instant;

use iced::advanced::widget::Tree;
use iced::advanced::Widget;
use iced::border::Radius;
use iced::{Color, Element};

use crate::{Animable, Animation, AnimationState};

// Animation implementation for some iced-rs types
impl Animable for iced::Vector {
    fn lerp(start: &Self, end: &Self, progress: f64) -> Self {
        Self {
            x: f32::lerp(&start.x, &end.x, progress),
            y: f32::lerp(&start.y, &end.y, progress),
        }
    }
}

impl Animable for iced::Point {
    fn lerp(start: &Self, end: &Self, progress: f64) -> Self {
        Self {
            x: f32::lerp(&start.x, &end.x, progress),
            y: f32::lerp(&start.y, &end.y, progress),
        }
    }
}

impl Animable for iced::Size {
    fn lerp(start: &Self, end: &Self, progress: f64) -> Self {
        Self {
            width: f32::lerp(&start.width, &end.width, progress),
            height: f32::lerp(&start.height, &end.height, progress),
        }
    }
}

impl Animable for Color {
    fn lerp(start: &Self, end: &Self, progress: f64) -> Self {
        Self {
            r: f32::lerp(&start.r, &end.r, progress),
            g: f32::lerp(&start.g, &end.g, progress),
            b: f32::lerp(&start.b, &end.b, progress),
            a: f32::lerp(&start.a, &end.a, progress),
        }
    }
}

impl Animable for iced::Padding {
    fn lerp(start: &Self, end: &Self, progress: f64) -> Self {
        Self {
            top: f32::lerp(&start.top, &end.top, progress),
            bottom: f32::lerp(&start.bottom, &end.bottom, progress),
            left: f32::lerp(&start.left, &end.left, progress),
            right: f32::lerp(&start.right, &end.right, progress),
        }
    }
}

impl Animable for iced::border::Radius {
    fn lerp(start: &Self, end: &Self, progress: f64) -> Self {
        Self {
            top_left: f32::lerp(&start.top_left, &end.top_left, progress),
            top_right: f32::lerp(&start.top_right, &end.top_right, progress),
            bottom_left: f32::lerp(&start.bottom_left, &end.bottom_left, progress),
            bottom_right: f32::lerp(&start.bottom_right, &end.bottom_right, progress),
        }
    }
}

impl Animable for iced::Border {
    fn lerp(start: &Self, end: &Self, progress: f64) -> Self {
        Self {
            color: Color::lerp(&start.color, &end.color, progress),
            width: f32::lerp(&start.width, &end.width, progress),
            radius: Radius::lerp(&start.radius, &end.radius, progress),
        }
    }
}

impl Animable for iced::Pixels {
    fn lerp(start: &Self, end: &Self, progress: f64) -> Self {
        Self(f32::lerp(&start.0, &end.0, progress))
    }
}

impl Animable for iced::Rectangle {
    fn lerp(start: &Self, end: &Self, progress: f64) -> Self {
        Self {
            x: f32::lerp(&start.x, &end.x, progress),
            y: f32::lerp(&start.y, &end.y, progress),
            width: f32::lerp(&start.width, &end.width, progress),
            height: f32::lerp(&start.height, &end.height, progress),
        }
    }
}

/// An animation update event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationEvent {
    /// An animation tick.
    Tick(Instant),
    /// Set the state of the animation.
    SetState(AnimationState),
    /// The animation finished.
    ///
    /// It's up to **you** to handle this event.
    Finished,
}

impl<T: Animable> Animation<T> {
    /// Update to an [`AnimationEvent`]
    pub fn update(&mut self, event: AnimationEvent) {
        match event {
            AnimationEvent::Tick(now) => self.tick(now),
            AnimationEvent::SetState(state) => self.set_state(state),
            AnimationEvent::Finished => (), // Up to the user to handle
        }
    }
}

/// A widget that helps you animate a value from your state.
pub struct AnimationWidget<'a, T: Animable, Message, Theme, Renderer> {
    animation: &'a Animation<T>,
    content: Element<'a, Message, Theme, Renderer>,
    on_update: Option<Box<dyn Fn(AnimationEvent) -> Message>>,
}

impl<'a, T, Message, Theme, Renderer> AnimationWidget<'a, T, Message, Theme, Renderer>
where
    T: 'static + Animable,
    Message: 'a + Clone,
{
    pub fn new(
        animation: &'a Animation<T>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            animation,
            content: content.into(),
            on_update: None,
        }
    }

    pub fn on_update<F>(mut self, on_update: F) -> Self
    where
        F: Fn(AnimationEvent) -> Message + 'static,
    {
        self.on_update = Some(Box::new(on_update));
        self
    }
}

impl<'a, T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for AnimationWidget<'a, T, Message, Theme, Renderer>
where
    T: 'static + Animable,
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size_hint()
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn mouse_interaction(
        &self,
        state: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(state, layout, cursor, viewport, renderer)
    }

    fn operate(
        &self,
        state: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation<()>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content
                .as_widget()
                .operate(&mut state.children[0], layout, renderer, operation);
        })
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::None
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer, translation)
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        )
    }

    fn on_event(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> iced::advanced::graphics::core::event::Status {
        let status = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if self.animation.is_finished() {
            if let Some(on_update) = &self.on_update {
                shell.publish(on_update(AnimationEvent::Finished));
            }
            return status;
        }

        if let Some(on_update) = &self.on_update {
            let now = Instant::now();
            let event = AnimationEvent::Tick(now);
            shell.publish(on_update(event));
        }

        status
    }
}

impl<'a, T, Message, Theme, Renderer> From<AnimationWidget<'a, T, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: 'static + Animable,
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(animation: AnimationWidget<'a, T, Message, Theme, Renderer>) -> Self {
        Self::new(animation)
    }
}
