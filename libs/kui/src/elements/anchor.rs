use {
    super::Length,
    crate::{Element, ElementMetrics, FocusDirection, FocusResult, LayoutInfo, SizeConstraint},
    vello::kurbo::{Point, Size, Vec2},
};

/// The style associated with an [`Anchor`] element.
///
/// See the documentation for the builder-like methods of [`Anchor`] for more information.
#[derive(Clone, Debug)]
pub struct AnchorStyle {
    pub anchor_x: f64,
    pub anchor_y: f64,
    pub offset_x: Length,
    pub offset_y: Length,
    pub fill_width: bool,
    pub fill_height: bool,
    pub fit_child: bool,
}

impl Default for AnchorStyle {
    fn default() -> Self {
        Self {
            anchor_x: 0.0,
            anchor_y: 0.0,
            offset_x: Length::ZERO,
            offset_y: Length::ZERO,
            fill_width: true,
            fill_height: true,
            fit_child: true,
        }
    }
}

/// The computed style of an [`Anchor`] element.
#[derive(Clone, Debug, Default)]
pub struct AnchorComputedStyle {
    /// The position of the element.
    pub position: Point,
    /// The size of the element.
    pub size: Size,
    /// The computed offset of the child element, relative to its parent.
    pub child_offset: Vec2,
}

/// An element that anchors its child to a specific position.
///
/// Anchor elements will attempt
#[derive(Clone, Debug, Default)]
pub struct Anchor<E: ?Sized> {
    pub style: AnchorStyle,
    pub computed_style: AnchorComputedStyle,
    pub child: E,
}

impl<E> Anchor<E> {
    /// Sets the anchor point of the child element of this [`Anchor`].
    pub fn anchor(mut self, x: f64, y: f64) -> Self {
        self.style.anchor_x = x;
        self.style.anchor_y = y;
        self
    }

    /// Sets the offset of the child element of this [`Anchor`].
    pub fn offset(mut self, x: Length, y: Length) -> Self {
        self.style.offset_x = x;
        self.style.offset_y = y;
        self
    }

    /// Sets whether this [`Anchor`] element should fill the width of its parent.
    pub fn fill_width(mut self, fill: bool) -> Self {
        self.style.fill_width = fill;
        self
    }

    /// Sets whether this [`Anchor`] element should fill the height of its parent.
    pub fn fill_height(mut self, fill: bool) -> Self {
        self.style.fill_height = fill;
        self
    }

    /// Sets whether this [`Anchor`] element should fit its child element.
    pub fn fit_child(mut self, fit: bool) -> Self {
        self.style.fit_child = fit;
        self
    }

    /// Sets the child element of this [`Anchor`].
    pub fn child<E2>(self, child: E2) -> Anchor<E2> {
        Anchor {
            style: self.style,
            computed_style: AnchorComputedStyle::default(),
            child,
        }
    }

    /// Aligns the child element of this [`Anchor`] at the center.
    pub fn align_center(mut self) -> Self {
        self.style.anchor_x = 0.5;
        self.style.anchor_y = 0.5;
        self.style.offset_x = Length::ZERO;
        self.style.offset_y = Length::ZERO;
        self
    }
}

impl<E: ?Sized + Element> Element for Anchor<E> {
    fn layout(&mut self, info: LayoutInfo) {
        let mut my_width = if self.style.fill_width {
            info.available.width().expect("Can't use a space-filling `Anchor` element when no max-width constraint is available")
        } else {
            0.0
        };

        let mut my_height = if self.style.fill_height {
            info.available.height().expect("Can't use a space-filling `Anchor` element when no max-height constraint is available")
        } else {
            0.0
        };

        let available_width = if self.style.fill_width {
            Some(my_width)
        } else {
            info.available.width()
        };

        let available_height = if self.style.fill_height {
            Some(my_height)
        } else {
            info.available.height()
        };

        self.child.layout(LayoutInfo {
            parent: Size::new(my_width, my_height),
            available: SizeConstraint::new(available_width, available_height),
            scale_factor: info.scale_factor,
        });

        let child_metrics = self.child.metrics();

        if self.style.fit_child {
            my_width = child_metrics.size.width.max(my_width);
            my_height = child_metrics.size.height.max(my_height);
        }

        let offset_x = self.style.offset_x.resolve(&info);
        let offset_y = self.style.offset_y.resolve(&info);

        let child_offset = Vec2 {
            x: (my_width - child_metrics.size.width) * self.style.anchor_x + offset_x,
            y: (my_height - child_metrics.size.height) * self.style.anchor_y + offset_y,
        };

        self.computed_style = AnchorComputedStyle {
            position: Point::ORIGIN,
            size: Size::new(my_width, my_height),
            child_offset,
        };
    }

    fn place(&mut self, pos: Point) {
        self.computed_style.position = pos;
        self.child.place(pos + self.computed_style.child_offset);
    }

    #[inline]
    fn metrics(&self) -> ElementMetrics {
        ElementMetrics {
            position: self.computed_style.position,
            size: self.computed_style.size,
        }
    }

    #[inline]
    fn hit_test(&self, point: Point) -> bool {
        self.child.hit_test(point)
    }

    #[inline]
    fn move_focus(&mut self, dir: FocusDirection) -> FocusResult {
        self.child.move_focus(dir)
    }

    #[inline]
    fn draw(&mut self, scene: &mut vello::Scene) {
        self.child.draw(scene);
    }
}
