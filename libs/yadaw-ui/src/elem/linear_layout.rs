use {
    crate::{
        elem::Length,
        element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
    },
    vello::{
        kurbo::{Point, Size, Vec2},
        Scene,
    },
};

/// A direction in which elements can be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    /// Elements are placed horizontally.
    Horizontal,
    /// Elements are placed vertically.
    Vertical,
}

impl Direction {
    /// Returns a vector representing this direction (positive x for horizontal, positive y for
    /// vertical).
    pub const fn to_vec2(self) -> Vec2 {
        match self {
            Self::Horizontal => Vec2::new(1.0, 0.0),
            Self::Vertical => Vec2::new(0.0, 1.0),
        }
    }
}

/// Describes how to justify the children of a layout along the main axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Justify {
    /// Chidren should be justified at the start of the layout.
    Start,
    /// Children should be justified at the center of the layout.
    Center,
    /// Children should be justified at the end of the layout.
    End,
}

/// Describes how to align the children of a layout along the cross axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Align {
    /// Children should be aligned at the start of the layout.
    Start,
    /// Children should be aligned at the center of the layout.
    Center,
    /// Children should be aligned at the end of the layout.
    End,
    /// The children should take up the full length of the cross-axis.
    Stretch,
}

/// The child of a [`LinearLayout<E>`].
pub struct Child<E: ?Sized> {
    /// The computed offset of the child element.
    offset: Vec2,

    /// The amount of growth that this element should receive
    /// when the layout is stretched along the main axis.
    ///
    /// Negative values mean that the element should not grow and instead, use its
    /// natural size (unconstrained layout).
    pub grow: f64,

    /// The self-alignment of the child element.
    ///
    /// If set, this overrides the alignment of the parent layout.
    pub align_self: Option<Align>,

    /// The child element.
    pub element: E,
}

impl<E> Child<E> {
    /// Creates a new child element with the provided element.
    pub fn new(element: E) -> Self {
        Self {
            offset: Vec2::ZERO,
            grow: -1.0,
            align_self: None,
            element,
        }
    }

    /// Sets the amount of growth that this element should receive.
    pub fn with_grow(mut self, grow: f64) -> Self {
        debug_assert!(grow >= 0.0, "Grow factor must be non-negative");
        self.grow = grow;
        self
    }

    /// Sets the self-alignment of the child element.
    pub fn with_align_self(mut self, align_self: Align) -> Self {
        self.align_self = Some(align_self);
        self
    }
}

impl<E: Element> From<E> for Child<E> {
    fn from(element: E) -> Self {
        Self::new(element)
    }
}

/// An element that lays out its children in a single direction.
pub struct LinearLayout<E> {
    /// The size used to calculate the child's layout.
    parent_size: Size,
    /// The position of the element.
    position: Point,
    /// The size of the element.
    size: Size,

    /// The length of the children, on the main axis.
    ///
    /// This is the sum of the lengths of all children, plus the gaps between them.
    children_main_length: f64,

    /// The length of the children, on the cross axis.
    ///
    /// This is the maximum length of all children.
    children_cross_length: f64,

    /// The elements that are laid out by this layout.
    pub children: Vec<Child<E>>,

    /// The direction in which the children are placed.
    pub direction: Direction,
    /// The gap between elements.
    pub gap: Length,
    /// The alignment of the children along the main axis.
    pub justify: Justify,
    /// The alignment of the children along the cross axis.
    pub align: Align,
}

impl<E> LinearLayout<E> {
    /// Adds a child element to the layout.
    pub fn with_child(mut self, child: impl Into<Child<E>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Sets the gap between elements.
    pub fn with_gap(mut self, gap: Length) -> Self {
        self.gap = gap;
        self
    }

    /// Sets the alignment of the children along the main axis.
    pub fn with_justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Sets the alignment of the children along the main axis to the start.
    #[inline]
    pub fn with_justify_start(self) -> Self {
        self.with_justify(Justify::Start)
    }

    /// Sets the alignment of the children along the main axis to the center.
    #[inline]
    pub fn with_justify_center(self) -> Self {
        self.with_justify(Justify::Center)
    }

    /// Sets the alignment of the children along the main axis to the end.
    #[inline]
    pub fn with_justify_end(self) -> Self {
        self.with_justify(Justify::End)
    }

    /// Sets the alignment of the children along the cross axis.
    pub fn with_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Sets the alignment of the children along the main axis to the start.
    #[inline]
    pub fn with_align_start(self) -> Self {
        self.with_align(Align::Start)
    }

    /// Sets the alignment of the children along the main axis to the center.
    #[inline]
    pub fn with_align_center(self) -> Self {
        self.with_align(Align::Center)
    }

    /// Sets the alignment of the children along the main axis to the end.
    #[inline]
    pub fn with_align_end(self) -> Self {
        self.with_align(Align::End)
    }

    /// Sets the alignment of the children along the main axis to the end.
    #[inline]
    pub fn with_align_stretch(self) -> Self {
        self.with_align(Align::Stretch)
    }

    /// Sets the direction in which the children are placed.
    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the direction of the layout to horizontal.
    #[inline]
    pub fn with_horizontal(self) -> Self {
        self.with_direction(Direction::Horizontal)
    }

    /// Sets the direction of the layout to vertical.
    #[inline]
    pub fn with_vertical(self) -> Self {
        self.with_direction(Direction::Vertical)
    }
}

impl<E> Default for LinearLayout<E> {
    fn default() -> Self {
        Self {
            parent_size: Size::ZERO,
            position: Point::ZERO,
            size: Size::ZERO,
            children_main_length: 0.0,
            children_cross_length: 0.0,
            children: Vec::new(),
            direction: Direction::Horizontal,
            gap: Length::ZERO,
            justify: Justify::Start,
            align: Align::Start,
        }
    }
}

trait WithChildren {
    fn with_children(&mut self, children: &mut dyn FnMut(&mut Child<dyn Element + '_>));
    fn any_children(
        &mut self,
        children: &mut dyn FnMut(&mut Child<dyn Element + '_>) -> bool,
    ) -> bool;
}

impl<E: Element> WithChildren for Vec<Child<E>> {
    fn with_children(&mut self, children: &mut dyn FnMut(&mut Child<dyn Element + '_>)) {
        self.iter_mut()
            .for_each(|x| children(x as &mut Child<dyn Element>));
    }

    fn any_children(
        &mut self,
        children: &mut dyn FnMut(&mut Child<dyn Element + '_>) -> bool,
    ) -> bool {
        self.iter_mut()
            .any(|x| children(x as &mut Child<dyn Element>))
    }
}

#[allow(clippy::too_many_arguments)]
fn dyn_set_size(
    cx: &ElemCtx,
    parent_size: Size,
    size: SetSize,
    direction: Direction,
    align: Align,
    justify: Justify,
    gap: &Length,
    children_count: usize,
    children: &mut dyn WithChildren,
) -> (f64, f64, Size) {
    let child_cx = cx.inherit_parent_size(parent_size);

    let gap = gap.resolve(&child_cx);

    let mut total_grow: f64 = 0.0;
    let mut has_growth: usize = 0;

    children.with_children(&mut |child| {
        if child.grow.is_sign_positive() {
            has_growth += 1;
            total_grow += child.grow;
        }
    });

    let mut children_main_length: f64 = 0.0;
    let mut children_cross_length: f64 = 0.0;

    children.with_children(&mut |child| {
        // We'll deal with grow factor later in a second pass.
        if child.grow.is_sign_positive() {
            return;
        }

        let align = child.align_self.unwrap_or(align);

        let mut child_set_size = SetSize::relaxed();
        if align == Align::Stretch {
            match direction {
                Direction::Horizontal => {
                    let height = size.height().expect("Horizontal LinearLayout with a stretch-aligned child must have a specific height");
                    child_set_size = child_set_size.with_height(height);
                }
                Direction::Vertical => {
                    let width = size.width().expect("Vertical LinearLayout with a stretch-aligned child must have a specific width");
                    child_set_size = child_set_size.with_width(width);
                }
            }
        }

        child.element.set_size(&child_cx, child_set_size);
        let child_metrics = child.element.metrics(&child_cx);

        match direction {
            Direction::Horizontal => {
                children_main_length += child_metrics.size.width + gap;
                children_cross_length = children_cross_length.max(child_metrics.size.height);
            }
            Direction::Vertical => {
                children_main_length += child_metrics.size.height + gap;
                children_cross_length = children_cross_length.max(child_metrics.size.width);
            }
        }
    });

    if has_growth != 0 {
        let mut growth_factor = match direction {
            Direction::Horizontal => {
                let width = size.width().expect(
                    "Horizontal LinearLayout with a grow factor must have a specific width",
                );
                let remaining_space = width - children_main_length - gap * (has_growth - 1) as f64;
                children_main_length = width;
                remaining_space / total_grow
            }
            Direction::Vertical => {
                let height = size
                    .height()
                    .expect("Vertical LinearLayout with a grow factor must have a specific height");
                let remaining_space = height - children_main_length - gap * (has_growth - 1) as f64;
                children_main_length = height;
                remaining_space / total_grow
            }
        };

        growth_factor = growth_factor.max(0.0);

        children.with_children(&mut |child| {
            // We've dealt with children with no growth factor is the previous loop.
            if child.grow.is_sign_negative() {
                return;
            }

            let align = child.align_self.unwrap_or(align);

            let mut child_set_size = SetSize::relaxed();
            if align == Align::Stretch {
                match direction {
                    Direction::Horizontal => {
                        let height = size.height().expect("Horizontal LinearLayout with a stretch-aligned child must have a specific height");
                        child_set_size = child_set_size.with_height(height);
                    }
                    Direction::Vertical => {
                        let width = size.width().expect("Vertical LinearLayout with a stretch-aligned child must have a specific width");
                        child_set_size = child_set_size.with_width(width);
                    }
                }
            }
            child_set_size = match direction {
                Direction::Horizontal => child_set_size.with_width(growth_factor * child.grow),
                Direction::Vertical => child_set_size.with_height(growth_factor * child.grow),
            };

            child.element.set_size(&child_cx, child_set_size);
        });
    }

    if children_count != 0 && has_growth == 0 {
        children_main_length -= gap;
    }

    let parent_size = match direction {
        Direction::Horizontal => Size::new(
            size.width().unwrap_or(children_cross_length),
            size.height().unwrap_or(children_cross_length),
        ),
        Direction::Vertical => Size::new(
            size.width().unwrap_or(children_main_length),
            size.height().unwrap_or(children_main_length),
        ),
    };

    let (parent_main_length, parent_cross_length) = match direction {
        Direction::Horizontal => (parent_size.width, parent_size.height),
        Direction::Vertical => (parent_size.height, parent_size.width),
    };

    let mut advance: f64 = match justify {
        Justify::Start => 0.0,
        Justify::Center => (parent_main_length - children_main_length) * 0.5,
        Justify::End => parent_main_length - children_main_length,
    };

    children.with_children(&mut |child| {
        let child_metrics = child.element.metrics(&child_cx);

        let (child_main_length, child_cross_length) = match direction {
            Direction::Horizontal => (child_metrics.size.width, child_metrics.size.height),
            Direction::Vertical => (child_metrics.size.height, child_metrics.size.width),
        };

        let child_cross_offset = match child.align_self.unwrap_or(align) {
            Align::Start => 0.0,
            Align::Center => (parent_cross_length - child_cross_length) * 0.5,
            Align::End => parent_cross_length - child_cross_length,
            Align::Stretch => 0.0,
        };

        child.offset = match direction {
            Direction::Horizontal => Vec2::new(advance, child_cross_offset),
            Direction::Vertical => Vec2::new(child_cross_offset, advance),
        };

        advance += child_main_length + gap;
    });

    (children_main_length, children_cross_length, parent_size)
}

fn dyn_set_position(
    cx: &ElemCtx,
    parent_size: Size,
    position: Point,
    children: &mut dyn WithChildren,
) {
    let child_cx = cx.inherit_parent_size(parent_size);

    children.with_children(&mut |child| {
        child
            .element
            .set_position(&child_cx, position + child.offset);
    });
}

fn dyn_render(cx: &ElemCtx, parent_size: Size, scene: &mut Scene, children: &mut dyn WithChildren) {
    let child_cx = cx.inherit_parent_size(parent_size);

    children.with_children(&mut |child| {
        child.element.render(&child_cx, scene);
    });
}

fn dyn_hit_test(
    cx: &ElemCtx,
    parent_size: Size,
    point: Point,
    children: &mut dyn WithChildren,
) -> bool {
    let child_cx = cx.inherit_parent_size(parent_size);
    children.any_children(&mut |child| child.element.hit_test(&child_cx, point))
}

fn dyn_event(
    cx: &ElemCtx,
    parent_size: Size,
    event: &dyn Event,
    children: &mut dyn WithChildren,
) -> EventResult {
    let child_cx = cx.inherit_parent_size(parent_size);

    let result =
        children.any_children(&mut |child| child.element.event(&child_cx, event).is_handled());

    if result {
        EventResult::Handled
    } else {
        EventResult::Ignored
    }
}

impl<E: Element> Element for LinearLayout<E> {
    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        self.parent_size = size.or_zero();

        let (children_main_length, children_cross_length, size) = dyn_set_size(
            cx,
            self.parent_size,
            size,
            self.direction,
            self.align,
            self.justify,
            &self.gap,
            self.children.len(),
            &mut self.children,
        );

        self.children_main_length = children_main_length;
        self.children_cross_length = children_cross_length;
        self.size = size;

        dyn_set_position(cx, self.parent_size, self.position, &mut self.children);
    }

    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        self.position = position;
        dyn_set_position(cx, self.parent_size, self.position, &mut self.children);
    }

    fn metrics(&mut self, _cx: &ElemCtx) -> Metrics {
        Metrics {
            size: self.size,
            position: self.position,
            baseline: 0.0,
        }
    }

    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        dyn_render(cx, self.parent_size, scene, &mut self.children);
    }

    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool {
        dyn_hit_test(cx, self.parent_size, point, &mut self.children)
    }

    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        dyn_event(cx, self.parent_size, event, &mut self.children)
    }
}
