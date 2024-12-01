//! Drawing shapes is a common task in UI programming. This module provides a set of basic
//! traits to draw shapes.

use {
    super::{Empty, Length},
    crate::element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
    vello::{
        kurbo::{self, Affine, Point, Shape},
        peniko::{BlendMode, Brush, Color, Fill},
        Scene,
    },
};

/// Describes how to create a [`Shape`] from a [`Rect`].
///
/// This is the `S` generic parameter of [`ShapeElement`].
pub trait ToShape {
    /// The type of shape that is created.
    type Shape: Shape;

    /// Creates a new shape from the provided rectangle.
    fn to_shape(&self, cx: &ElemCtx, rect: kurbo::Rect) -> Self::Shape;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect;

impl ToShape for Rect {
    type Shape = kurbo::Rect;

    #[inline]
    fn to_shape(&self, _cx: &ElemCtx, rect: kurbo::Rect) -> Self::Shape {
        rect
    }
}

/// A rectangle with rounded corners.
#[derive(Debug, Clone, Default)]
pub struct RoundedRect {
    /// The radius of the top-left corner.
    pub top_left: Length,
    /// The radius of the top-right corner.
    pub top_right: Length,
    /// The radius of the bottom-right corner.
    pub bottom_right: Length,
    /// The radius of the bottom-left corner.
    pub bottom_left: Length,
}

impl ToShape for RoundedRect {
    type Shape = kurbo::RoundedRect;

    fn to_shape(&self, cx: &ElemCtx, rect: kurbo::Rect) -> Self::Shape {
        let cx = cx.inherit_parent_size(rect.size());

        let top_left = self.top_left.resolve(&cx);
        let top_right = self.top_right.resolve(&cx);
        let bottom_right = self.bottom_right.resolve(&cx);
        let bottom_left = self.bottom_left.resolve(&cx);

        let radii = kurbo::RoundedRectRadii::new(top_left, top_right, bottom_right, bottom_left);
        rect.to_rounded_rect(radii)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Ellipse;

impl ToShape for Ellipse {
    type Shape = kurbo::Ellipse;

    #[inline]
    fn to_shape(&self, _cx: &ElemCtx, rect: kurbo::Rect) -> Self::Shape {
        rect.to_ellipse()
    }
}

/// The trait that the `D` generic parameter of [`ShapeElement`] must implement.
pub trait DrawShape {
    /// Draws the shape.
    fn draw<S: Shape>(&mut self, shape: &S, scene: &mut Scene);
}

impl DrawShape for () {
    #[inline]
    fn draw<S: Shape>(&mut self, _shape: &S, _scene: &mut Scene) {}
}

/// An implementation of [`DrawShape`] that fills the shape with a color.
#[derive(Debug, Clone)]
pub struct FillShape {
    /// The fill mode to use when drawing the shape.
    pub fill: Fill,
    /// The brush to use when drawing the shape.
    pub brush: Brush,
    /// The brush transform to apply when drawing the shape.
    pub brush_transform: Option<Affine>,
}

impl DrawShape for FillShape {
    fn draw<S: Shape>(&mut self, shape: &S, scene: &mut Scene) {
        scene.fill(
            self.fill,
            Affine::IDENTITY,
            &self.brush,
            self.brush_transform,
            shape,
        );
    }
}

impl Default for FillShape {
    fn default() -> Self {
        Self {
            fill: Fill::NonZero,
            brush: Color::TRANSPARENT.into(),
            brush_transform: None,
        }
    }
}

/// The trait that the `C` generic parameter of [`ShapeElement`] must implement.
pub trait ClipShape {
    /// Determines whether a hit test succeeds here.
    fn hit_test<S: Shape>(&mut self, shape: &S, point: Point) -> bool;

    /// Pushes the clip layer.
    fn push_layer<S: Shape>(&mut self, shape: &S, scene: &mut Scene);

    /// Pop the clip layer.
    fn pop_layer(&mut self, scene: &mut Scene);

    /// Inherit the context.
    fn inherit_context(&self, bounds: kurbo::Rect, cx: &ElemCtx) -> ElemCtx;
}

impl ClipShape for () {
    #[inline]
    fn hit_test<S: Shape>(&mut self, _shape: &S, _point: Point) -> bool {
        false
    }

    #[inline]
    fn push_layer<S: Shape>(&mut self, _shape: &S, _scene: &mut Scene) {}

    #[inline]
    fn pop_layer(&mut self, _scene: &mut Scene) {}

    #[inline]
    fn inherit_context(&self, _bounds: kurbo::Rect, cx: &ElemCtx) -> ElemCtx {
        cx.clone()
    }
}

/// An implementation of [`ClipShape`] that do clip the shape.
#[derive(Debug, Clone)]
pub struct DoClipShape {
    /// The opacity used to blend the child element with the background.
    pub opacity: f32,
    /// The blend mode to use when blending the child element with the background.
    pub blend_mode: BlendMode,
}

impl ClipShape for DoClipShape {
    #[inline]
    fn hit_test<S: Shape>(&mut self, shape: &S, point: Point) -> bool {
        shape.contains(point)
    }

    fn push_layer<S: Shape>(&mut self, shape: &S, scene: &mut Scene) {
        scene.push_layer(self.blend_mode, self.opacity, Affine::IDENTITY, shape);
    }

    #[inline]
    fn pop_layer(&mut self, scene: &mut Scene) {
        scene.pop_layer()
    }

    #[inline]
    fn inherit_context(&self, bounds: kurbo::Rect, cx: &ElemCtx) -> ElemCtx {
        cx.inherit_clip_rect(bounds)
    }
}

impl Default for DoClipShape {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            blend_mode: BlendMode::default(),
        }
    }
}

/// An element that draws a shape.
pub struct ShapeElement<S, D, C, E: ?Sized> {
    /// The shape to draw.
    pub shape: S,
    /// Information about how to draw the shape.
    pub draw: D,
    /// Information about how to clip the shape.
    pub clip: C,
    /// Whether hits should be blocked without caring about the child element.
    pub block_all_hits: bool,
    /// The child element of the shape.
    pub child: E,
}

impl<S: Default, D: Default, C: Default> Default for ShapeElement<S, D, C, Empty> {
    fn default() -> Self {
        Self {
            shape: S::default(),
            draw: D::default(),
            clip: C::default(),
            block_all_hits: true,
            child: Empty::default(),
        }
    }
}

impl<S: Default> ShapeElement<S, (), (), Empty> {
    /// Creates a new shape element.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S, D, C, E> ShapeElement<S, D, C, E> {
    /// Sets whether the shape element should be allowed to block all hits
    /// without caring about the child element.
    pub fn with_no_block_hits(mut self) -> Self {
        self.block_all_hits = false;
        self
    }

    /// Make the shape clip its child element.
    pub fn with_clip_shape(self) -> ShapeElement<S, D, DoClipShape, E> {
        ShapeElement {
            shape: self.shape,
            draw: self.draw,
            clip: DoClipShape::default(),
            block_all_hits: self.block_all_hits,
            child: self.child,
        }
    }

    /// Fill the shape.
    pub fn with_fill_shape(self) -> ShapeElement<S, FillShape, C, E> {
        ShapeElement {
            shape: self.shape,
            draw: FillShape::default(),
            clip: self.clip,
            block_all_hits: self.block_all_hits,
            child: self.child,
        }
    }

    /// Sets the child element of the shape.
    pub fn with_child<E2>(self, element: E2) -> ShapeElement<S, D, C, E2> {
        ShapeElement {
            shape: self.shape,
            draw: self.draw,
            clip: self.clip,
            block_all_hits: self.block_all_hits,
            child: element,
        }
    }
}

impl<S, C, E> ShapeElement<S, FillShape, C, E> {
    /// Sets the fill mode to use when drawing the shape.
    pub fn with_fill_mode(mut self, fill: Fill) -> Self {
        self.draw.fill = fill;
        self
    }

    /// Sets the brush to use when drawing the shape.
    pub fn with_brush(mut self, brush: impl Into<Brush>) -> Self {
        self.draw.brush = brush.into();
        self
    }

    /// Sets the brush transform to apply when drawing the shape.
    pub fn with_brush_transform(mut self, brush_transform: Affine) -> Self {
        self.draw.brush_transform = Some(brush_transform);
        self
    }
}

impl<S, D, E> ShapeElement<S, D, DoClipShape, E> {
    /// Sets the opacity used to blend the child element with the background.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.clip.opacity = opacity;
        self
    }

    /// Sets the blend mode to use when blending the child element with the background.
    pub fn with_blend_mode(mut self, blend_mode: impl Into<BlendMode>) -> Self {
        self.clip.blend_mode = blend_mode.into();
        self
    }
}

impl<D, C, E> ShapeElement<RoundedRect, D, C, E> {
    /// Sets the radius of all corners.
    pub fn with_radius(mut self, radius: Length) -> Self {
        self.shape.top_left = radius.clone();
        self.shape.top_right = radius.clone();
        self.shape.bottom_right = radius.clone();
        self.shape.bottom_left = radius;
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

    /// Sets the radius of the bottom-right corner.
    pub fn with_bottom_right_radius(mut self, radius: Length) -> Self {
        self.shape.bottom_right = radius;
        self
    }

    /// Sets the radius of the bottom-left corner.
    pub fn with_bottom_left_radius(mut self, radius: Length) -> Self {
        self.shape.bottom_left = radius;
        self
    }
}

/// A shape element that fills the shape with a color.
pub type WithBackground<S, E> = ShapeElement<S, FillShape, (), E>;

/// A shape element that clips its child element.
pub type ClipChild<S, E> = ShapeElement<S, (), DoClipShape, E>;

/// A shape element that fills the shape with a color and clips its child element.
pub type SolidShape<S> = ShapeElement<S, FillShape, (), Empty>;

impl<S, D, C, E> Element for ShapeElement<S, D, C, E>
where
    S: ToShape,
    D: DrawShape,
    C: ClipShape,
    E: ?Sized + Element,
{
    #[inline]
    fn ready(&mut self, cx: &ElemCtx) {
        self.child.ready(cx);
    }

    #[inline]
    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        self.child.set_size(cx, size);
    }

    #[inline]
    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        self.child.set_position(cx, position);
    }

    #[inline]
    fn metrics(&mut self, cx: &ElemCtx) -> Metrics {
        self.child.metrics(cx)
    }

    #[inline]
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        let rect = self.child.metrics(cx).rect();
        let shape = self.shape.to_shape(cx, rect);

        let cx = self.clip.inherit_context(rect, cx);
        self.draw.draw(&shape, scene);
        self.clip.push_layer(&shape, scene);
        self.child.render(&cx, scene);
        self.clip.pop_layer(scene);
    }

    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool {
        let rect = self.child.metrics(cx).rect();
        let shape = self.shape.to_shape(cx, rect);
        let cx = self.clip.inherit_context(rect, cx);

        if self.block_all_hits {
            shape.contains(point)
        } else {
            self.clip.hit_test(&shape, point) && self.child.hit_test(&cx, point)
        }
    }

    #[inline]
    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        let rect = self.child.metrics(cx).rect();
        let cx = self.clip.inherit_context(rect, cx);
        self.child.event(&cx, event)
    }
}
