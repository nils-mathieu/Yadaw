use {
    crate::element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
    vello::{
        kurbo::{Point, Size},
        Scene,
    },
};

/// An element that displays nothing.
#[derive(Default, Debug, Clone)]
pub struct Empty {
    pos: Point,
    size: Size,
}

impl Element for Empty {
    #[inline]
    fn set_size(&mut self, _cx: &ElemCtx, size: SetSize) {
        self.size = size.or_zero();
    }

    #[inline]
    fn set_position(&mut self, _cx: &ElemCtx, position: Point) {
        self.pos = position;
    }

    #[inline]
    fn metrics(&mut self, _cx: &ElemCtx) -> Metrics {
        Metrics {
            size: self.size,
            position: self.pos,
            baseline: 0.0,
        }
    }

    #[inline]
    fn render(&mut self, _cx: &ElemCtx, _scene: &mut Scene) {}

    #[inline]
    fn hit_test(&mut self, _cx: &ElemCtx, _point: Point) -> bool {
        false
    }

    #[inline]
    fn event(&mut self, _cx: &ElemCtx, _event: &dyn Event) -> EventResult {
        EventResult::Ignored
    }
}
