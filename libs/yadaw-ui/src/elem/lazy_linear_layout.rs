use {
    crate::{
        elem::{linear_layout::Direction, Length},
        element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
    },
    vello::{
        kurbo::{Point, Size, Vec2},
        Scene,
    },
};

/// A child entry in a [`LazyLinearLayout`].
struct ChildEntry<E: ?Sized> {
    index: usize,
    element: E,
}

/// An [`Element`] that lays out an infinite number of children
/// in a single direction.
pub struct LazyLinearLayout<E, F: ?Sized> {
    /// The current position of the layout.
    position: Point,
    /// The size of the layout.
    size: Size,

    /// The direction in which the children are placed.
    direction: Direction,
    /// The width of each child.
    child_width: Length,
    /// The height of each child.
    child_height: Length,
    /// The gap between each element.
    gap: Length,

    /// The children currently being laid out.
    children: Vec<ChildEntry<E>>,
    /// The function that is responsible for creating the child elements of the layout.
    ///
    /// ```rust, ignore
    /// fn make_children(index: usize) -> E;
    /// ```
    make_children: F,
}

impl<E, F> LazyLinearLayout<E, F> {
    /// Creates a new [`LazyLinearLayout`] with the provided function to create children.
    pub fn new(make_children: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self {
            position: Point::ZERO,
            size: Size::ZERO,
            direction: Direction::Horizontal,
            child_width: Length::ZERO,
            child_height: Length::ZERO,
            gap: Length::ZERO,
            children: Vec::new(),
            make_children,
        }
    }

    /// Sets the direction of the layout to horizontal.
    pub fn with_direction_horizontal(mut self) -> Self {
        self.direction = Direction::Horizontal;
        self
    }

    /// Sets the direction of the layout to vertical.
    pub fn with_direction_vertical(mut self) -> Self {
        self.direction = Direction::Vertical;
        self
    }

    /// Sets the width of each child.
    pub fn with_child_width(mut self, child_width: Length) -> Self {
        self.child_width = child_width;
        self
    }

    /// Sets the height of each child.
    pub fn with_child_height(mut self, child_height: Length) -> Self {
        self.child_height = child_height;
        self
    }

    /// Sets the gap between each element.
    pub fn with_gap(mut self, gap: Length) -> Self {
        self.gap = gap;
        self
    }
}

impl<E, F> LazyLinearLayout<E, F>
where
    E: Element,
    F: ?Sized + FnMut(usize) -> E,
{
    /// Refreshes the visible children.
    ///
    /// # Parameters
    ///
    /// - `cx`: The [`ElemCtx`] passed to the [`Element`].
    ///
    /// - `update_children`: A function to be called once the hidden elements have been removed,
    ///   but before the new ones are added.
    fn refresh_children(&mut self, cx: &ElemCtx, update_children: &mut dyn FnMut(&mut Self, Vec2)) {
        let gap = self.gap.resolve(cx);

        let child_width = self.child_width.resolve(cx);
        let child_height = self.child_height.resolve(cx);

        let stride = match self.direction {
            Direction::Horizontal => child_width + gap,
            Direction::Vertical => child_height + gap,
        };

        let (start, end) = match self.direction {
            Direction::Horizontal => (self.position.x, self.position.x + self.size.width),
            Direction::Vertical => (self.position.y, self.position.y + self.size.height),
        };

        let (visible_start, visible_end) = match self.direction {
            Direction::Horizontal => (cx.clip_rect().x0, cx.clip_rect().x1),
            Direction::Vertical => (cx.clip_rect().y0, cx.clip_rect().y1),
        };

        let true_start = start.max(visible_start);
        let true_end = end.min(visible_end);

        let skipped_children = ((true_start - start) / stride).floor() as usize;
        let visible_children = ((true_end - true_start) / stride).ceil() as usize + 1;

        // Remove children that are no longer visible.
        self.children.retain(|child| {
            child.index >= skipped_children && child.index <= skipped_children + visible_children
        });

        let stride_vec2 = self.direction.to_vec2() * stride;

        update_children(self, stride_vec2);

        // Add children that are now visible.
        let initial_len = self.children.len();
        for index in skipped_children..skipped_children + visible_children {
            if self.children[..initial_len]
                .iter()
                .any(|child| child.index == index)
            {
                continue;
            }

            self.children.push(ChildEntry {
                index,
                element: (self.make_children)(index),
            });
            let child = &mut self.children.last_mut().unwrap().element;
            child.set_size(
                cx,
                SetSize::from_specific(Size::new(child_width, child_height)),
            );
            child.set_position(cx, self.position + stride_vec2 * index as f64);
        }
    }
}

impl<E, F> Element for LazyLinearLayout<E, F>
where
    E: Element,
    F: ?Sized + FnMut(usize) -> E,
{
    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        self.position = position;

        self.refresh_children(cx, &mut |this, stride| {
            for child in &mut this.children {
                let pos = position + stride * child.index as f64;
                child.element.set_position(cx, pos);
            }
        });
    }

    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        match self.direction {
            Direction::Horizontal => {
                assert!(
                    size.has_specific_height(),
                    "Horizontal LazyLinearLayout does not support having a specific height"
                );

                let width = size.width().unwrap_or(f64::INFINITY);

                self.size = Size::new(width, self.child_height.resolve(cx));
            }
            Direction::Vertical => {
                assert!(
                    size.has_specific_width(),
                    "Vertical LazyLinearLayout does not support having a specific width"
                );

                let height = size.height().unwrap_or(f64::INFINITY);

                self.size = Size::new(self.child_width.resolve(cx), height);
            }
        }

        self.refresh_children(cx, &mut |_, _| {});
    }

    #[inline]
    fn metrics(&mut self, _cx: &ElemCtx) -> Metrics {
        Metrics {
            size: self.size,
            position: self.position,
            baseline: 0.0,
        }
    }

    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        for child in &mut self.children {
            child.element.render(cx, scene);
        }
    }

    fn hit_test(&mut self, cx: &ElemCtx, point: Point) -> bool {
        self.children
            .iter_mut()
            .any(|child| child.element.hit_test(cx, point))
    }

    fn event(&mut self, cx: &ElemCtx, event: &dyn Event) -> EventResult {
        for child in &mut self.children {
            if child.element.event(cx, event).is_handled() {
                return EventResult::Handled;
            }
        }

        EventResult::Ignored
    }
}
