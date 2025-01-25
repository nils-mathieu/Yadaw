use {
    crate::{
        ElemContext, Element, LayoutContext, SizeHint,
        elements::Length,
        event::{Event, EventResult},
    },
    smallvec::smallvec,
    vello::{
        kurbo::{
            Affine, Insets, Point, Rect, RoundedRect, RoundedRectRadii, Shape, Size, Stroke, Vec2,
        },
        peniko::{Brush, Fill, Mix},
    },
};

/// The style associated with a [`Div`] element.
///
/// The documentation for individual fields can be found in the builder-like methods of the
/// [`Div`] type.
#[derive(Clone, Debug)]
pub struct DivStyle {
    pub brush: Option<Brush>,
    pub top_left_radius: Length,
    pub top_right_radius: Length,
    pub bottom_left_radius: Length,
    pub bottom_right_radius: Length,
    pub border_brush: Option<Brush>,
    pub border_thickness: Length,
    pub border_dash: Length,
    pub border_dash_offset: Length,
    pub padding_left: Length,
    pub padding_right: Length,
    pub padding_top: Length,
    pub padding_bottom: Length,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub min_width: Option<Length>,
    pub min_height: Option<Length>,
    pub max_width: Option<Length>,
    pub max_height: Option<Length>,
    pub clip_content: bool,
    pub opacity: f32,
}

impl DivStyle {
    /// Resolves the horizontal padding.
    ///
    /// # Remarks
    ///
    /// This function does not take border thickness into account.
    pub fn resolve_horizontal_padding(&self, layout_context: &LayoutContext) -> f64 {
        self.padding_left.resolve(layout_context) + self.padding_right.resolve(layout_context)
    }

    /// Resolves the vertical padding.
    ///
    /// # Remarks
    ///
    /// This function does not take border thickness into account.
    pub fn resolve_vertical_padding(&self, layout_context: &LayoutContext) -> f64 {
        self.padding_top.resolve(layout_context) + self.padding_bottom.resolve(layout_context)
    }

    /// Resolves the vertical and horizontal padding into a [`Size`].
    ///
    /// # Remarks
    ///
    /// This function *does* take border thickness into account.
    pub fn resolve_padding_size(&self, layout_context: &LayoutContext) -> Size {
        let border_thickness = self.border_thickness.resolve(layout_context);
        Size::new(
            self.resolve_horizontal_padding(layout_context) + border_thickness * 2.0,
            self.resolve_vertical_padding(layout_context) + border_thickness * 2.0,
        )
    }

    /// Resolves the minimum size of the [`Div`] element.
    pub fn resolve_min_size(&self, layout_context: &LayoutContext) -> Size {
        Size::new(
            self.min_width
                .as_ref()
                .map_or(0.0, |min_width| min_width.resolve(layout_context)),
            self.min_height
                .as_ref()
                .map_or(0.0, |min_height| min_height.resolve(layout_context)),
        )
    }

    /// Resolves the maximum size of the [`Div`] element.
    pub fn resolve_max_size(&self, layout_context: &LayoutContext) -> Size {
        Size::new(
            self.max_width
                .as_ref()
                .map_or(f64::INFINITY, |max_width| max_width.resolve(layout_context)),
            self.max_height
                .as_ref()
                .map_or(f64::INFINITY, |max_height| {
                    max_height.resolve(layout_context)
                }),
        )
    }

    /// Resolves the size of the [`Div`] element.
    pub fn resolve_size(&self, fallback: Size, layout_context: &LayoutContext) -> Size {
        Size::new(
            self.width
                .as_ref()
                .map_or(fallback.width, |width| width.resolve(layout_context)),
            self.height
                .as_ref()
                .map_or(fallback.height, |height| height.resolve(layout_context)),
        )
    }
}

impl Default for DivStyle {
    fn default() -> Self {
        Self {
            brush: None,
            top_left_radius: Length::ZERO,
            top_right_radius: Length::ZERO,
            bottom_left_radius: Length::ZERO,
            bottom_right_radius: Length::ZERO,
            border_brush: None,
            border_thickness: Length::ZERO,
            border_dash: Length::ZERO,
            border_dash_offset: Length::ZERO,
            padding_left: Length::ZERO,
            padding_right: Length::ZERO,
            padding_top: Length::ZERO,
            padding_bottom: Length::ZERO,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            clip_content: false,
            opacity: 1.0,
        }
    }
}

/// The computed style of a [`Div`] element.
#[derive(Default, Clone, Debug)]
pub struct DivComputedStyle {
    pub position: Point,
    pub size: Size,
    pub corner_radiuses: RoundedRectRadii,
    pub border_thickness: f64,
    pub border_dash: f64,
    pub border_dash_offset: f64,
}

/// Works a bit like an HTML `<div>` element, except it does not provide any layout capabilities.
#[derive(Clone, Debug, Default)]
pub struct Div<E: ?Sized> {
    /// The style of the [`Div`] element.
    pub style: DivStyle,
    /// The computed style of the [`Div`] element.
    pub computed_style: DivComputedStyle,
    /// The child element of the [`Div`].
    pub child: E,
}

impl<E> Div<E> {
    /// Sets the background brush of the [`Div`] element.
    pub fn brush(mut self, brush: impl Into<Brush>) -> Self {
        self.style.brush = Some(brush.into());
        self
    }

    /// Sets the top-left radius of the [`Div`] element.
    pub fn top_left_radius(mut self, radius: Length) -> Self {
        self.style.top_left_radius = radius;
        self
    }

    /// Sets the top-right radius of the [`Div`] element.
    pub fn top_right_radius(mut self, radius: Length) -> Self {
        self.style.top_right_radius = radius;
        self
    }

    /// Sets the bottom-left radius of the [`Div`] element.
    pub fn bottom_left_radius(mut self, radius: Length) -> Self {
        self.style.bottom_left_radius = radius;
        self
    }

    /// Sets the bottom-right radius of the [`Div`] element.
    pub fn bottom_right_radius(mut self, radius: Length) -> Self {
        self.style.bottom_right_radius = radius;
        self
    }

    /// Sets the radius of all four corners of the [`Div`] element.
    pub fn radius(mut self, radius: Length) -> Self {
        self.style.top_left_radius = radius.clone();
        self.style.top_right_radius = radius.clone();
        self.style.bottom_left_radius = radius.clone();
        self.style.bottom_right_radius = radius;
        self
    }

    /// Sets the border brush of the [`Div`] element.
    pub fn border_brush(mut self, brush: impl Into<Brush>) -> Self {
        self.style.border_brush = Some(brush.into());
        self
    }

    /// Sets the border thickness of the [`Div`] element.
    pub fn border_thickness(mut self, thickness: Length) -> Self {
        self.style.border_thickness = thickness;
        self
    }

    /// Sets the border dash of the [`Div`] element.
    pub fn border_dash(mut self, dash: Length) -> Self {
        self.style.border_dash = dash;
        self
    }

    /// Sets the border dash offset of the [`Div`] element.
    pub fn border_dash_offset(mut self, offset: Length) -> Self {
        self.style.border_dash_offset = offset;
        self
    }

    /// Sets the width of the [`Div`] element.
    pub fn width(mut self, width: impl Into<Option<Length>>) -> Self {
        self.style.width = width.into();
        self
    }

    /// Sets the height of the [`Div`] element.
    pub fn height(mut self, height: impl Into<Option<Length>>) -> Self {
        self.style.height = height.into();
        self
    }

    /// Sets the minimum width of the [`Div`] element.
    pub fn min_width(mut self, min_width: impl Into<Option<Length>>) -> Self {
        self.style.min_width = min_width.into();
        self
    }

    /// Sets the minimum height of the [`Div`] element.
    pub fn min_height(mut self, min_height: impl Into<Option<Length>>) -> Self {
        self.style.min_height = min_height.into();
        self
    }

    /// Sets the maximum width of the [`Div`] element.
    pub fn max_width(mut self, max_width: impl Into<Option<Length>>) -> Self {
        self.style.max_width = max_width.into();
        self
    }

    /// Sets the maximum height of the [`Div`] element.
    pub fn max_height(mut self, max_height: impl Into<Option<Length>>) -> Self {
        self.style.max_height = max_height.into();
        self
    }

    /// Sets whether the content of the [`Div`] element should be clipped.
    pub fn clip_content(mut self, clip_content: bool) -> Self {
        self.style.clip_content = clip_content;
        self
    }

    /// The opacity value of the [`Div`] element.
    ///
    /// Note that this will only take effect when `clip_content` is set.
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.style.opacity = opacity;
        self
    }

    /// Sets the child of the [`Div`] element.
    pub fn child<E2>(self, child: E2) -> Div<E2> {
        Div {
            style: self.style,
            computed_style: DivComputedStyle::default(),
            child,
        }
    }

    /// The left padding of the [`Div`] element.
    pub fn padding_left(mut self, padding: Length) -> Self {
        self.style.padding_left = padding;
        self
    }

    /// The right padding of the [`Div`] element.
    pub fn padding_right(mut self, padding: Length) -> Self {
        self.style.padding_right = padding;
        self
    }

    /// The top padding of the [`Div`] element.
    pub fn padding_top(mut self, padding: Length) -> Self {
        self.style.padding_top = padding;
        self
    }

    /// The bottom padding of the [`Div`] element.
    pub fn padding_bottom(mut self, padding: Length) -> Self {
        self.style.padding_bottom = padding;
        self
    }

    /// The padding of the [`Div`] element.
    pub fn padding(mut self, padding: Length) -> Self {
        self.style.padding_left = padding.clone();
        self.style.padding_right = padding.clone();
        self.style.padding_top = padding.clone();
        self.style.padding_bottom = padding;
        self
    }
}

impl<E: ?Sized + Element> Div<E> {
    /// Computes the shape that the div element will be rendered with.
    pub fn computed_shape(&self) -> RoundedRect {
        Rect::from_origin_size(self.computed_style.position, self.computed_style.size)
            .to_rounded_rect(self.computed_style.corner_radiuses)
    }
}

fn size_min(a: Size, b: Size) -> Size {
    Size::new(a.width.min(b.width), a.height.min(b.height))
}

fn size_max(a: Size, b: Size) -> Size {
    Size::new(a.width.max(b.width), a.height.max(b.height))
}

impl<E: ?Sized + Element> Element for Div<E> {
    fn size_hint(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        space: Size,
    ) -> SizeHint {
        let padding = self.style.resolve_padding_size(&layout_context);

        let min_size = self.style.resolve_min_size(&layout_context);
        let max_size = self.style.resolve_max_size(&layout_context);

        let mut child_space = self.style.resolve_size(space, &layout_context);
        child_space = child_space.clamp(min_size, max_size);
        child_space -= padding;
        child_space.width = child_space.width.max(0.0);
        child_space.height = child_space.height.max(0.0);

        let child_size_hint = self.child.size_hint(
            elem_context,
            LayoutContext {
                parent: child_space,
                scale_factor: layout_context.scale_factor,
            },
            child_space,
        );

        SizeHint {
            preferred: self
                .style
                .resolve_size(child_size_hint.preferred + padding, &layout_context),
            min: size_max(min_size, child_size_hint.min + padding),
            max: size_min(max_size, child_size_hint.max + padding),
        }
    }

    fn place(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        position: Point,
        size: Size,
    ) {
        let border_thickness = self.style.border_thickness.resolve(&layout_context);

        let padding_left = self.style.padding_left.resolve(&layout_context) + border_thickness;
        let padding_right = self.style.padding_right.resolve(&layout_context) + border_thickness;
        let padding_top = self.style.padding_top.resolve(&layout_context) + border_thickness;
        let padding_bottom = self.style.padding_bottom.resolve(&layout_context) + border_thickness;

        let horizontal_padding = padding_left + padding_right;
        let vertical_padding = padding_top + padding_bottom;

        let content_size = size - Size::new(horizontal_padding, vertical_padding);

        self.child.place(
            elem_context,
            LayoutContext {
                parent: content_size,
                scale_factor: layout_context.scale_factor,
            },
            position + Vec2::new(padding_left, padding_top),
            content_size,
        );

        self.computed_style = DivComputedStyle {
            size,
            position,
            corner_radiuses: RoundedRectRadii {
                top_left: self.style.top_left_radius.resolve(&layout_context),
                top_right: self.style.top_right_radius.resolve(&layout_context),
                bottom_right: self.style.bottom_left_radius.resolve(&layout_context),
                bottom_left: self.style.bottom_right_radius.resolve(&layout_context),
            },
            border_thickness,
            border_dash: self.style.border_dash.resolve(&layout_context),
            border_dash_offset: self.style.border_dash_offset.resolve(&layout_context),
        };
    }

    fn hit_test(&self, point: Point) -> bool {
        if !self.style.clip_content && self.child.hit_test(point) {
            return true;
        }

        if self.style.brush.is_some() || self.style.border_brush.is_some() {
            self.computed_shape().contains(point)
        } else {
            false
        }
    }

    fn draw(&mut self, elem_context: &ElemContext, scene: &mut vello::Scene) {
        let outer_shape = self.computed_shape();

        if let Some(brush) = self.style.brush.as_ref() {
            scene.fill(Fill::NonZero, Affine::IDENTITY, brush, None, &outer_shape);
        }

        if let Some(border_brush) = self.style.border_brush.as_ref() {
            scene.stroke(
                &Stroke {
                    width: self.computed_style.border_thickness,
                    dash_pattern: if self.computed_style.border_dash == 0.0 {
                        smallvec![]
                    } else {
                        smallvec![self.computed_style.border_dash]
                    },
                    dash_offset: self.computed_style.border_dash_offset,
                    ..Default::default()
                },
                Affine::IDENTITY,
                border_brush,
                None,
                &(outer_shape.rect() - Insets::uniform(self.computed_style.border_thickness / 2.0))
                    .to_rounded_rect(self.computed_style.corner_radiuses),
            );
        }

        if self.style.clip_content {
            scene.push_layer(
                if self.style.opacity == 1.0 {
                    Mix::Clip
                } else {
                    Mix::Normal
                },
                self.style.opacity,
                Affine::IDENTITY,
                &outer_shape,
            );
        }

        self.child.draw(elem_context, scene);

        if self.style.clip_content {
            scene.pop_layer();
        }
    }

    #[inline]
    fn event(&mut self, elem_context: &ElemContext, event: &dyn Event) -> EventResult {
        self.child.event(elem_context, event)
    }

    #[inline]
    fn begin(&mut self, elem_context: &ElemContext) {
        self.child.begin(elem_context);
    }
}
