use {
    super::Length,
    crate::{
        ElemContext, Element, LayoutContext, SizeHint,
        event::{Event, EventResult},
    },
    core::f64,
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
}

impl Default for AnchorStyle {
    fn default() -> Self {
        Self {
            anchor_x: 0.0,
            anchor_y: 0.0,
            offset_x: Length::ZERO,
            offset_y: Length::ZERO,
        }
    }
}

/// An element that anchors its child to a specific position.
///
/// Anchor elements will attempt
#[derive(Clone, Debug, Default)]
pub struct Anchor<E: ?Sized> {
    pub style: AnchorStyle,
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

    /// Sets the child element of this [`Anchor`].
    pub fn child<E2>(self, child: E2) -> Anchor<E2> {
        Anchor {
            style: self.style,
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
    fn size_hint(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        space: Size,
    ) -> SizeHint {
        let child_size_hint = self.child.size_hint(
            elem_context,
            LayoutContext {
                parent: space,
                scale_factor: layout_context.scale_factor,
            },
            space,
        );

        SizeHint {
            preferred: child_size_hint.preferred,
            min: child_size_hint.min,
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
        let child_layout_context = LayoutContext {
            parent: size,
            scale_factor: layout_context.scale_factor,
        };
        let child_size_hint = self
            .child
            .size_hint(elem_context, child_layout_context, size);

        let offset_x = self.style.offset_x.resolve(&layout_context);
        let offset_y = self.style.offset_y.resolve(&layout_context);

        let child_size = child_size_hint.preferred;

        let child_offset = Vec2::new(
            pos.x + self.style.anchor_x * (size.width - child_size.width) + offset_x,
            pos.y + self.style.anchor_y * (size.height - child_size.height) + offset_y,
        );

        self.child.place(
            elem_context,
            child_layout_context,
            pos + child_offset,
            child_size,
        );
    }

    #[inline]
    fn hit_test(&self, point: Point) -> bool {
        self.child.hit_test(point)
    }

    #[inline]
    fn draw(&mut self, elem_context: &ElemContext, scene: &mut vello::Scene) {
        self.child.draw(elem_context, scene);
    }

    #[inline]
    fn event(&mut self, elem_context: &ElemContext, event: &dyn Event) -> EventResult {
        self.child.event(elem_context, event)
    }
}
