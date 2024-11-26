//! Drawing shapes is a common task in UI programming. This module provides a set of basic
//! traits to draw shapes.

use {
    crate::{
        elem::Length,
        element::{ElemCtx, Element, Event, EventResult, Metrics, SizeHint},
    },
    vello::{
        kurbo::{Affine, Point, Rect, RoundedRect, RoundedRectRadii, Shape},
        peniko::{Brush, Fill},
        Scene,
    },
};

/// Describes how to create a [`Shape`] from a [`Rect`].
pub trait ToShape {
    /// The type of shape that is created.
    type Shape: Shape;

    /// Creates a new shape from the provided rectangle.
    fn to_shape(&self, cx: &ElemCtx, rect: Rect) -> Self::Shape;
}

/// A rectangle shape.
#[derive(Default, Debug, Clone, Copy)]
pub struct Rectangle;

impl ToShape for Rectangle {
    type Shape = Rect;

    #[inline]
    fn to_shape(&self, _cx: &ElemCtx, rect: Rect) -> Rect {
        rect
    }
}

/// A rectangle with rounded corners.
#[derive(Default, Debug, Clone)]
pub struct RoundedRectangle {
    /// The radius of the top left corner.
    pub top_left: Length,
    /// The radius of the top right corner.
    pub top_right: Length,
    /// The radius of the bottom left corner.
    pub bottom_left: Length,
    /// The radius of the bottom right corner.
    pub bottom_right: Length,
}

impl RoundedRectangle {
    /// Creates a new [`RoundedRectangle`] with the provided corner lengths.
    pub fn to_rounded_rect(&self, rect: Rect, cx: &ElemCtx) -> RoundedRect {
        RoundedRect::from_rect(
            rect,
            RoundedRectRadii::new(
                self.top_left.resolve(cx),
                self.top_right.resolve(cx),
                self.bottom_right.resolve(cx),
                self.bottom_left.resolve(cx),
            ),
        )
    }
}

impl ToShape for RoundedRectangle {
    type Shape = RoundedRect;

    #[inline]
    fn to_shape(&self, cx: &ElemCtx, rect: Rect) -> RoundedRect {
        RoundedRect::from_rect(
            rect,
            RoundedRectRadii::new(
                self.top_left.resolve(cx),
                self.top_right.resolve(cx),
                self.bottom_right.resolve(cx),
                self.bottom_left.resolve(cx),
            ),
        )
    }
}

/// A shape that can be drawn.
#[derive(Debug, Default, Clone)]
pub struct ShapeElement<S: ?Sized> {
    /// The bounds associated with the shape.
    bounds: Rect,

    /// The brush to use for drawing the shape.
    pub brush: Brush,
    /// The transformation to apply to the brush.
    pub brush_transform: Option<Affine>,

    /// The shape to draw.
    pub shape: S,
}

impl<S> ShapeElement<S> {
    /// Sets the brush to use for drawing the shape.
    pub fn with_brush(mut self, brush: impl Into<Brush>) -> Self {
        self.brush = brush.into();
        self
    }

    /// Sets the transformation to apply to the brush.
    pub fn with_brush_transform(mut self, brush_transform: Affine) -> Self {
        self.brush_transform = Some(brush_transform);
        self
    }
}

impl ShapeElement<RoundedRectangle> {
    /// Sets the corner radius of the shape.
    pub fn with_corner_radius(mut self, radius: Length) -> Self {
        self.shape.top_left = radius.clone();
        self.shape.top_right = radius.clone();
        self.shape.bottom_left = radius.clone();
        self.shape.bottom_right = radius;
        self
    }

    /// Sets the radius of the top-left corner.
    pub fn with_top_left_radius(mut self, radius: Length) -> Self {
        self.shape.top_left = radius;
        self
    }

    /// Sets the radius of the top-right corner.
    pub fn with_top_right_radius(mut self, radius: Length) -> Self {
        self.shape.top_right = radius;
        self
    }

    /// Sets the radius of the bottom-left corner.
    pub fn with_bottom_left_radius(mut self, radius: Length) -> Self {
        self.shape.bottom_left = radius;
        self
    }

    /// Sets the radius of the bottom-right corner.
    pub fn with_bottom_right_radius(mut self, radius: Length) -> Self {
        self.shape.bottom_right = radius;
        self
    }
}

impl<S: ToShape> Element for ShapeElement<S> {
    #[inline]
    fn size_hint(&mut self, _cx: &ElemCtx) -> SizeHint {
        SizeHint::ANY
    }

    #[inline]
    fn place(&mut self, _cx: &ElemCtx, bounds: Rect) {
        self.bounds = bounds;
    }

    #[inline]
    fn metrics(&self, _cx: &ElemCtx) -> Metrics {
        Metrics {
            rect: self.bounds,
            baseline: 0.0,
        }
    }

    #[inline]
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &self.brush,
            self.brush_transform,
            &self.shape.to_shape(cx, self.bounds),
        );
    }

    #[inline]
    fn hit_test(&self, cx: &ElemCtx, point: Point) -> bool {
        self.shape.to_shape(cx, self.bounds).contains(point)
    }

    #[inline]
    fn event(&mut self, _cx: &ElemCtx, _event: &dyn Event) -> EventResult {
        EventResult::Ignored
    }
}

/// An [`Element`] that draws a background shape behind its child.
pub struct WithBackground<S, E: ?Sized> {
    /// The background shape to draw.
    pub background: ShapeElement<S>,
    /// The child element.
    pub child: E,
}

impl<S: Shape, E> WithBackground<S, E> {
    /// Creates a new [`WithBackground`] with the provided background shape and child element.
    pub fn new(background: ShapeElement<S>, child: E) -> Self {
        Self { background, child }
    }

    /// Sets the brush to use for drawing the background shape.
    pub fn with_background_brush(mut self, brush: impl Into<Brush>) -> Self {
        self.background.brush = brush.into();
        self
    }

    /// Sets the transformation to apply to the background brush.
    pub fn with_background_brush_transform(mut self, brush_transform: Affine) -> Self {
        self.background.brush_transform = Some(brush_transform);
        self
    }
}

impl<S: ToShape, E: ?Sized + Element> Element for WithBackground<S, E> {
    #[inline]
    fn size_hint(&mut self, cx: &ElemCtx) -> SizeHint {
        self.child.size_hint(cx)
    }

    fn place(&mut self, cx: &ElemCtx, bounds: Rect) {
        self.background.place(cx, bounds);
        self.child.place(cx, bounds);
    }

    #[inline]
    fn metrics(&self, cx: &ElemCtx) -> Metrics {
        self.child.metrics(cx)
    }

    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        self.background.render(cx, scene);
        self.child.render(cx, scene);
    }

    fn hit_test(&self, cx: &ElemCtx, point: Point) -> bool {
        self.child.hit_test(cx, point) || self.background.hit_test(cx, point)
    }

    #[inline]
    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        self.child.event(cx, event)
    }
}
