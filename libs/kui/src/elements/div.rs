use {
    crate::{Element, ElementMetrics, LayoutInfo, elements::Length},
    vello::{kurbo::Point, peniko::Brush},
};

/// The style associated with a [`Div`] element.
///
/// The documentation for individual fields can be found in the builder-like methods of the
/// [`Div`] type.
#[derive(Clone, Debug, Default)]
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
    pub height: Option<Length>,
}

/// Works a bit like an HTML `<div>` element, except it does not provide any layout capabilities.
#[derive(Clone, Debug)]
pub struct Div<E: ?Sized> {
    /// The style of the [`Div`] element.
    pub style: DivStyle,
    /// The child element of the [`Div`].
    pub child: E,
}

impl Default for Div<()> {
    fn default() -> Self {
        Self {
            style: DivStyle::default(),
            child: (),
        }
    }
}

impl<E> Div<E> {
    /// Sets the background brush of the [`Div`] element.
    pub fn with_brush(mut self, brush: Brush) -> Self {
        self.style.brush = Some(brush);
        self
    }

    /// Sets the top-left radius of the [`Div`] element.
    pub fn with_top_left_radius(mut self, radius: Length) -> Self {
        self.style.top_left_radius = radius;
        self
    }

    /// Sets the top-right radius of the [`Div`] element.
    pub fn with_top_right_radius(mut self, radius: Length) -> Self {
        self.style.top_right_radius = radius;
        self
    }

    /// Sets the bottom-left radius of the [`Div`] element.
    pub fn with_bottom_left_radius(mut self, radius: Length) -> Self {
        self.style.bottom_left_radius = radius;
        self
    }

    /// Sets the bottom-right radius of the [`Div`] element.
    pub fn with_bottom_right_radius(mut self, radius: Length) -> Self {
        self.style.bottom_right_radius = radius;
        self
    }

    /// Sets the radius of all four corners of the [`Div`] element.
    pub fn with_radius(mut self, radius: Length) -> Self {
        self.style.top_left_radius = radius.clone();
        self.style.top_right_radius = radius.clone();
        self.style.bottom_left_radius = radius.clone();
        self.style.bottom_right_radius = radius;
        self
    }

    /// Sets the border brush of the [`Div`] element.
    pub fn with_border_brush(mut self, brush: Brush) -> Self {
        self.style.border_brush = Some(brush);
        self
    }

    /// Sets the border thickness of the [`Div`] element.
    pub fn with_border_thickness(mut self, thickness: Length) -> Self {
        self.style.border_thickness = thickness;
        self
    }

    /// Sets the border dash of the [`Div`] element.
    pub fn with_border_dash(mut self, dash: Length) -> Self {
        self.style.border_dash = dash;
        self
    }

    /// Sets the border dash offset of the [`Div`] element.
    pub fn with_border_dash_offset(mut self, offset: Length) -> Self {
        self.style.border_dash_offset = offset;
        self
    }

    /// Sets the width of the [`Div`] element.
    pub fn with_width(mut self, width: Length) -> Self {
        self.style.width = Some(width);
        self
    }

    /// Sets the height of the [`Div`] element.
    pub fn with_height(mut self, height: Length) -> Self {
        self.style.height = Some(height);
        self
    }

    /// Sets whether the content of the [`Div`] element should be clipped.
    pub fn with_clip_content(mut self, clip_content: bool) -> Self {
        self.style.clip_content = clip_content;
        self
    }

    /// Sets the child of the [`Div`] element.
    pub fn with_child<E2>(self, child: E2) -> Div<E2> {
        Div {
            style: self.style,
            child,
        }
    }
}

impl<E: ?Sized + Element> Element for Div<E> {
    fn layout(&mut self, info: LayoutInfo) {
        let total_width = self.style.width.as_ref().map_or(0.0, |w| w.resolve(&info));
        let total_height = self.style.height.as_ref().map_or(0.0, |h| h.resolve(&info));
    }

    fn metrics(&self) -> ElementMetrics {}

    fn place(&mut self, pos: Point) {}
}
