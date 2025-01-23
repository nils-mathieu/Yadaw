use {
    super::Length,
    crate::{
        ElemContext, Element, LayoutContext, SizeHint,
        event::{Event, EventResult},
    },
    core::f64,
    vello::{
        Scene,
        kurbo::{Point, Size, Vec2},
    },
};

/// The direction of a [`Flex`] element.
#[derive(Clone, Default, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// The children of the container are laid out horizontally.
    #[default]
    Horizontal,
    /// The children of the container are laid out vertically.
    Vertical,
}

/// The alignment of a child in a [`Flex`] element.
#[derive(Clone, Default, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    /// Align the child at the start of the flex box.
    #[default]
    Start,
    /// Align the child at the center of the flex box.
    Center,
    /// Align the child at the end of the flex box.
    End,
}

/// The child of a flex box.
#[derive(Debug, Clone, Default)]
pub struct FlexChild<E: ?Sized> {
    pub grow: f64,
    pub align_self: Option<Align>,

    /// Cached size hint of the child element.
    size_hint: SizeHint,

    pub child: E,
}

impl<E> FlexChild<E> {
    /// Sets the growth factor of this [`FlexChild`].
    pub fn grow(mut self, grow: f64) -> Self {
        self.grow = grow;
        self
    }

    /// Sets the alignment of this [`FlexChild`].
    pub fn align_self(mut self, align: Align) -> Self {
        self.align_self = Some(align);
        self
    }

    /// Sets the child of this [`FlexChild`].
    pub fn child<E2>(self, child: E2) -> FlexChild<E2> {
        FlexChild {
            grow: self.grow,
            align_self: self.align_self,
            size_hint: SizeHint::default(),
            child,
        }
    }
}

impl<E: Element> From<E> for FlexChild<E> {
    fn from(child: E) -> Self {
        FlexChild {
            grow: 0.0,
            align_self: None,
            size_hint: SizeHint::default(),
            child,
        }
    }
}

/// A flex box element that can contain other elements.
#[derive(Default)]
pub struct Flex<'a> {
    pub direction: Direction,
    pub gap: Length,
    pub align: Align,
    pub justify: Align,
    pub children: Vec<Box<FlexChild<dyn 'a + Element>>>,
}

impl<'a> Flex<'a> {
    /// The direction in which the children of this [`Flex`] box element are laid out.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the direction of this [`Flex`] box element to horizontal.
    pub fn horizontal(self) -> Self {
        self.direction(Direction::Horizontal)
    }

    /// Sets the direction of this [`Flex`] box element to vertical.
    pub fn vertical(self) -> Self {
        self.direction(Direction::Vertical)
    }

    /// Sets the gap of this [`Flex`] box element.
    ///
    /// # Remarks
    ///
    /// Passing a negative value here will reset the "basis" of the element, meaning that instead
    /// of starting from the element's preferred size, the element will instead start from zero.
    pub fn gap(mut self, gap: Length) -> Self {
        self.gap = gap;
        self
    }

    /// Sets the default alignment of the children in this [`Flex`] box element.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Aligns the children at the start of this [`Flex`] box element.
    pub fn align_start(self) -> Self {
        self.align(Align::Start)
    }

    /// Aligns the children at the center of this [`Flex`] box element.
    pub fn align_center(self) -> Self {
        self.align(Align::Center)
    }

    /// Aligns the children at the end of this [`Flex`] box element.
    pub fn align_end(self) -> Self {
        self.align(Align::End)
    }

    /// Sets the default justification of the children in this [`Flex`] box element.
    pub fn justify(mut self, justify: Align) -> Self {
        self.justify = justify;
        self
    }

    /// Justifies the children at the start of this [`Flex`] box element.
    pub fn justify_start(self) -> Self {
        self.justify(Align::Start)
    }

    /// Justifies the children at the center of this [`Flex`] box element.
    pub fn justify_center(self) -> Self {
        self.justify(Align::Center)
    }

    /// Justifies the children at the end of this [`Flex`] box element.
    pub fn justify_end(self) -> Self {
        self.justify(Align::End)
    }

    /// Adds a child to this [`Flex`] box element.
    pub fn child<E: Element + 'a>(mut self, child: impl Into<FlexChild<E>>) -> Self {
        self.children.push(Box::new(child.into()));
        self
    }

    /// Adds an empty space to this [`Flex`] box element.
    ///
    /// The associated value is the growth factor of the space.
    pub fn space(self, grow: f64) -> Self {
        self.child(FlexChild::<()>::default().grow(grow))
    }
}

impl std::fmt::Debug for Flex<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Flex")
            .field("direction", &self.direction)
            .field("gap", &self.gap)
            .field("align", &self.align)
            .field("justify", &self.justify)
            .field("children", &self.children.len())
            .finish()
    }
}

/// Stores information about the children of a [`Flex`] element.
struct ChildrenMetrics {
    /// The total growth of the childrem.
    total_growth: f64,
    /// The total length of the children.
    total_length: f64,
    /// The maximum cross length of a single child.
    max_cross_length: f64,
}

impl ChildrenMetrics {
    /// Computes the metrics.
    ///
    /// # Parameters
    ///
    /// - `gap`: The gap between the children.
    ///
    /// - `dir`: The direction of the flex box.
    ///
    /// - `children`: The children of the flex box.
    ///
    /// - `elem_context`: The element context.
    ///
    /// - `child_layout_context`: The layout context of the children.
    ///
    /// # Remarks
    ///
    /// This function will store the size hints of the children in the `size_hint` field of the
    /// [`FlexChild`] struct.
    pub fn compute(
        gap: f64,
        dir: Direction,
        children: &mut [Box<FlexChild<dyn '_ + Element>>],
        elem_context: &ElemContext,
        child_layout_context: LayoutContext,
    ) -> Self {
        let mut total_growth: f64 = 0.0;
        let mut total_length: f64 = 0.0;
        let mut max_cross_length: f64 = 0.0;

        for child in children.iter_mut() {
            #[rustfmt::skip]
            let child_space = match dir {
                Direction::Horizontal => Size::new(f64::INFINITY, child_layout_context.parent.height),
                Direction::Vertical => Size::new(child_layout_context.parent.width, f64::INFINITY),
            };

            child.size_hint =
                child
                    .child
                    .size_hint(elem_context, child_layout_context, child_space);

            total_growth += child.grow.abs();
            total_length += gap;

            match dir {
                Direction::Horizontal => {
                    if child.grow >= 0.0 {
                        total_length += child.size_hint.preferred.width;
                    }
                    max_cross_length = max_cross_length.max(child.size_hint.preferred.height);
                }
                Direction::Vertical => {
                    if child.grow >= 0.0 {
                        total_length += child.size_hint.preferred.height;
                    }
                    max_cross_length = max_cross_length.max(child.size_hint.preferred.width);
                }
            }
        }

        if !children.is_empty() {
            total_length -= gap;
        }

        Self {
            total_growth,
            total_length,
            max_cross_length,
        }
    }
}

impl Element for Flex<'_> {
    fn size_hint(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        space: Size,
    ) -> SizeHint {
        let ChildrenMetrics {
            total_length,
            max_cross_length,
            ..
        } = ChildrenMetrics::compute(
            self.gap.resolve(&layout_context),
            self.direction,
            &mut self.children,
            elem_context,
            LayoutContext {
                parent: space,
                scale_factor: layout_context.scale_factor,
            },
        );

        SizeHint {
            preferred: space,
            min: match self.direction {
                Direction::Horizontal => Size::new(total_length, max_cross_length),
                Direction::Vertical => Size::new(max_cross_length, total_length),
            },
            max: Size::new(f64::INFINITY, f64::INFINITY),
        }
    }

    fn place(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        pos: Point,
        size: Size,
    ) {
        //
        // Iterate once through the children to compute the total growth factor and the total
        // minimum size.
        //

        let gap = self.gap.resolve(&layout_context);

        let max_length = match self.direction {
            Direction::Horizontal => size.width,
            Direction::Vertical => size.height,
        };

        let ChildrenMetrics {
            total_growth,
            total_length,
            ..
        } = ChildrenMetrics::compute(
            gap,
            self.direction,
            &mut self.children,
            elem_context,
            LayoutContext {
                parent: size,
                scale_factor: layout_context.scale_factor,
            },
        );

        let grow_factor = if total_growth > 0.0 && max_length > total_length {
            assert!(
                max_length.is_finite(),
                "A `Flex` element cannot have growing children without being constrained",
            );
            (max_length - total_length) / total_growth
        } else {
            0.0
        };

        //
        // Place the children.
        //

        let mut advance = if grow_factor != 0.0 {
            0.0
        } else {
            match self.justify {
                Align::Start => 0.0,
                Align::Center => (max_length - total_length) / 2.0,
                Align::End => max_length - total_length,
            }
        };

        let cross_size = match self.direction {
            Direction::Horizontal => size.height,
            Direction::Vertical => size.width,
        };

        for child in &mut self.children {
            let mut child_length = if child.grow < 0.0 {
                0.0
            } else {
                match self.direction {
                    Direction::Horizontal => child.size_hint.preferred.width,
                    Direction::Vertical => child.size_hint.preferred.height,
                }
            };
            child_length += child.grow.abs() * grow_factor;

            let child_size = (match self.direction {
                Direction::Horizontal => Size::new(child_length, child.size_hint.preferred.height),
                Direction::Vertical => Size::new(child.size_hint.preferred.width, child_length),
            })
            .clamp(child.size_hint.min, child.size_hint.max);

            let child_cross_length = match self.direction {
                Direction::Horizontal => child_size.height,
                Direction::Vertical => child_size.width,
            };

            let cross_axis_offset = match child.align_self.unwrap_or(self.align) {
                Align::Start => 0.0,
                Align::Center => (cross_size - child_cross_length) * 0.5,
                Align::End => cross_size - child_cross_length,
            };

            let child_offset = match self.direction {
                Direction::Horizontal => Vec2::new(advance, cross_axis_offset),
                Direction::Vertical => Vec2::new(cross_axis_offset, advance),
            };

            child.child.place(
                elem_context,
                LayoutContext {
                    parent: size,
                    scale_factor: layout_context.scale_factor,
                },
                pos + child_offset,
                child_size,
            );

            advance += gap;
            match self.direction {
                Direction::Horizontal => advance += child_size.width,
                Direction::Vertical => advance += child_size.height,
            }
        }
    }

    fn hit_test(&self, point: Point) -> bool {
        self.children
            .iter()
            .any(|child| child.child.hit_test(point))
    }

    fn draw(&mut self, elem_context: &ElemContext, scene: &mut Scene) {
        self.children
            .iter_mut()
            .for_each(|child| child.child.draw(elem_context, scene))
    }

    fn event(&mut self, elem_context: &ElemContext, event: &dyn Event) -> EventResult {
        for child in &mut self.children {
            if child.child.event(elem_context, event).is_handled() {
                return EventResult::Handled;
            }
        }
        EventResult::Continue
    }

    #[inline]
    fn begin(&mut self, elem_context: &ElemContext) {
        self.children
            .iter_mut()
            .for_each(|child| child.child.begin(elem_context));
    }
}
