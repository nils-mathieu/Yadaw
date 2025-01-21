use {
    crate::{
        ElemContext, Element, ElementMetrics, FocusDirection, FocusResult, LayoutInfo,
        SizeConstraint, elements::Length,
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
    pub clip_content: bool,
    pub opacity: f32,
    pub height: Option<Length>,
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
            clip_content: false,
            opacity: 1.0,
            height: None,
        }
    }
}

/// The computed style of a [`Div`] element.
#[derive(Default, Clone, Debug)]
pub struct DivComputedStyle {
    /// The size of the element.
    pub size: Size,
    /// The position of the element.
    pub position: Point,
    /// The offset of the child element, relative to the parent element.
    pub child_offset: Vec2,
    /// The corner radiuses of the element.
    pub corner_radiuses: RoundedRectRadii,
    /// The thickness of the element's border.
    pub border_thickness: f64,
    /// The length of the element's border dash.
    pub border_dash: f64,
    /// The offset between the dashes of the element's border.
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
    pub fn width(mut self, width: Length) -> Self {
        self.style.width = Some(width);
        self
    }

    /// Sets the height of the [`Div`] element.
    pub fn height(mut self, height: Length) -> Self {
        self.style.height = Some(height);
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
}

impl<E: ?Sized + Element> Div<E> {
    /// Computes the shape that the div element will be rendered with.
    pub fn computed_shape(&self) -> RoundedRect {
        Rect::from_origin_size(self.computed_style.position, self.computed_style.size)
            .to_rounded_rect(self.computed_style.corner_radiuses)
    }
}

impl<E: ?Sized + Element> Element for Div<E> {
    fn layout(&mut self, elem_context: &ElemContext, info: LayoutInfo) {
        let border_thickness = self.style.border_thickness.resolve(&info);

        let padding_left = self.style.padding_left.resolve(&info) + border_thickness;
        let padding_right = self.style.padding_right.resolve(&info) + border_thickness;
        let padding_top = self.style.padding_top.resolve(&info) + border_thickness;
        let padding_bottom = self.style.padding_bottom.resolve(&info) + border_thickness;

        let horizontal_padding = padding_left + padding_right;
        let vertical_padding = padding_top + padding_bottom;

        let requested_width = self.style.width.as_ref().map(|w| w.resolve(&info));
        let requested_height = self.style.height.as_ref().map(|h| h.resolve(&info));

        let content_width = requested_width
            .or(info.available.width())
            .map(|w| w - horizontal_padding);
        let content_height = requested_height
            .or(info.available.height())
            .map(|h| h - vertical_padding);

        self.child.layout(elem_context, LayoutInfo {
            parent: Size {
                width: content_width.unwrap_or_default(),
                height: content_height.unwrap_or_default(),
            },
            available: SizeConstraint::new(content_width, content_height),
            scale_factor: info.scale_factor,
        });

        let child_metrics = self.child.metrics();

        self.computed_style = DivComputedStyle {
            size: Size {
                width: requested_width.unwrap_or(child_metrics.size.width + horizontal_padding),
                height: requested_height.unwrap_or(child_metrics.size.height + vertical_padding),
            },
            position: Point::ORIGIN,
            child_offset: Vec2::new(padding_left, padding_top),
            corner_radiuses: RoundedRectRadii {
                top_left: self.style.top_left_radius.resolve(&info),
                top_right: self.style.top_right_radius.resolve(&info),
                bottom_right: self.style.bottom_left_radius.resolve(&info),
                bottom_left: self.style.bottom_right_radius.resolve(&info),
            },
            border_thickness,
            border_dash: self.style.border_dash.resolve(&info),
            border_dash_offset: self.style.border_dash_offset.resolve(&info),
        };
    }

    #[inline]
    fn place(&mut self, elem_context: &ElemContext, pos: Point) {
        self.computed_style.position = pos;
        self.child
            .place(elem_context, pos + self.computed_style.child_offset);
    }

    fn metrics(&self) -> ElementMetrics {
        ElementMetrics {
            position: self.computed_style.position,
            size: self.computed_style.size,
        }
    }

    fn hit_test(&self, elem_context: &ElemContext, point: Point) -> bool {
        if !self.style.clip_content && self.child.hit_test(elem_context, point) {
            return true;
        }

        if self.style.brush.is_some() || self.style.border_brush.is_some() {
            self.computed_shape().contains(point)
        } else {
            false
        }
    }

    #[inline]
    fn move_focus(&mut self, elem_context: &ElemContext, dir: FocusDirection) -> FocusResult {
        self.child.move_focus(elem_context, dir)
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
}
