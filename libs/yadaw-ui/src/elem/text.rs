pub use parley::Alignment;

use {
    crate::{
        elem::Length,
        element::{ElemCtx, Element, Event, EventResult, Metrics, SetSize},
    },
    parley::{
        FontContext, FontStack, FontStyle, FontWeight, GenericFamily, LayoutContext,
        PositionedLayoutItem, RangedBuilder, StyleProperty,
    },
    vello::{
        kurbo::{Affine, Point, Size},
        peniko::{Brush, Color, Fill},
        Glyph, Scene,
    },
};

/// The dirty state of a text element.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DirtyState {
    /// The layout is clean and does not need to be rebuilt.
    #[default]
    Clean,
    /// The alignment of the element has changed. The lines must be broken again.
    Alignment,
    /// The layout of the element has changed without the text changing. The lines must be broken
    /// again.
    Lines,
    /// The text of the element has changed and the layout needs to be rebuilt.
    Text,
}

/// The unstyled part of a [`Text`] element.
///
/// This is mostly used to avoid the monomorphisation cost of [`Text`].
#[derive(Clone)]
struct UnstyledText {
    /// The current position of the text.
    position: Point,
    /// The current size of the text.
    size: SetSize,

    /// The alignment of the text.
    alignment: Alignment,
    /// Whether lines should be allowed to break.
    break_lines: bool,

    /// The dirty state of the text.
    dirty_state: DirtyState,
    /// The built layout of the text.
    layout: parley::Layout<Brush>,
}

impl UnstyledText {
    /// Adds dirt to the text element.
    #[inline]
    pub fn add_dirt(&mut self, dirt: DirtyState) {
        self.dirty_state = self.dirty_state.max(dirt);
    }

    /// Sets the alignment of the text.
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = alignment;
        self.add_dirt(DirtyState::Alignment);
    }

    /// Sets whether lines should be allowed to break.
    pub fn set_break_lines(&mut self, break_lines: bool) {
        self.break_lines = break_lines;
        self.add_dirt(DirtyState::Lines);
    }

    /// Sets the position of the text element.
    pub fn set_position(&mut self, _cx: &ElemCtx, position: Point) {
        self.position = position;
    }

    /// Sets the size of the text element.
    pub fn set_size(&mut self, _cx: &ElemCtx, size: SetSize, _style: &mut dyn TextStyle) {
        if self.size != size {
            self.size = size;
            self.add_dirt(DirtyState::Lines);
        }
    }

    /// Builds the layout of the text.
    pub fn build(&mut self, cx: &ElemCtx, text: &str, style: &mut dyn TextStyle) {
        if self.dirty_state >= DirtyState::Text {
            cx.app().with_resources_mut(|res| {
                res.get_or_insert_default::<FontContext>();
                res.get_or_insert_default::<LayoutContext<Brush>>();

                let (fcx, lcx) = res.get_many_mut::<(FontContext, LayoutContext<Brush>)>();
                let fcx = fcx.unwrap();
                let lcx = lcx.unwrap();

                let mut style_builder = lcx.ranged_builder(fcx, text, 1.0);
                style.build(cx, text, &mut style_builder);
                style_builder.build_into(&mut self.layout, text);
            });
        }

        let width = self.size.width().map_or(f32::INFINITY, |w| w as f32);

        if self.dirty_state >= DirtyState::Lines {
            self.layout
                .break_lines()
                .break_remaining(if self.break_lines {
                    width
                } else {
                    f32::INFINITY
                });
        }

        if self.dirty_state >= DirtyState::Alignment {
            let width = self.break_lines.then_some(width);
            self.layout.align(width, self.alignment);
        }

        self.dirty_state = DirtyState::Clean;
    }

    /// Returns the metrics of the text element.
    pub fn metrics(&mut self, cx: &ElemCtx, text: &str, style: &mut dyn TextStyle) -> Metrics {
        self.build(cx, text, style);

        let baseline = match self.layout.lines().last() {
            Some(line) => {
                let metrics = line.metrics();
                self.layout.height() - metrics.min_coord + metrics.baseline
            }
            None => 0.0,
        };

        let layout_size = Size::new(self.layout.width() as f64, self.layout.height() as f64);

        Metrics {
            position: self.position,
            size: self.size.or_fallback(layout_size),
            baseline: baseline as f64,
        }
    }

    /// Renders the text element.
    pub fn render(
        &mut self,
        cx: &ElemCtx,
        text: &str,
        scene: &mut Scene,
        style: &mut dyn TextStyle,
    ) {
        self.build(cx, text, style);

        for line in self.layout.lines() {
            for item in line.items() {
                match item {
                    PositionedLayoutItem::GlyphRun(run) => {
                        scene
                            .draw_glyphs(run.run().font())
                            .brush(&run.style().brush)
                            .font_size(run.run().font_size())
                            .transform(Affine::translate(self.position.to_vec2()))
                            .draw(
                                Fill::NonZero,
                                run.positioned_glyphs().map(|glyph| Glyph {
                                    id: glyph.id as u32,
                                    x: glyph.x,
                                    y: glyph.y,
                                }),
                            );
                    }
                    PositionedLayoutItem::InlineBox(_) => {
                        unimplemented!("Inline boxes are not supported");
                    }
                }
            }
        }
    }
}

/// Describes how to style a text element.
pub trait TextStyle {
    /// Builds the styled text.
    fn build(&self, cx: &ElemCtx, text: &str, builder: &mut RangedBuilder<Brush>);
}

impl TextStyle for () {
    #[inline]
    fn build(&self, _cx: &ElemCtx, _text: &str, _builder: &mut RangedBuilder<Brush>) {}
}

/// A basic implementation of [`TextStyle`] that sets basic text properties uniformly.
pub struct BasicTextStyle {
    /// The brush used to draw the text.
    pub brush: Brush,
    /// The weight of the text.
    pub weight: FontWeight,
    /// The style of the text.
    pub style: FontStyle,
    /// Whether the text should be underlined.
    pub underline: bool,
    /// Whether the text should be struck through.
    pub strikethrough: bool,
    /// The font size of the text.
    pub font_size: Length,
    /// The font family of the text.
    pub font_family: FontStack<'static>,
    /// The line height of the text.
    pub line_height: Length,
    /// The letter spacing of the text.
    pub letter_spacing: Length,
}

impl TextStyle for BasicTextStyle {
    #[rustfmt::skip]
    fn build(&self, cx: &ElemCtx, _text: &str, builder: &mut RangedBuilder<Brush>) {
        builder.push_default(StyleProperty::Brush(self.brush.clone()));
        builder.push_default(StyleProperty::FontSize(self.font_size.resolve(cx) as f32));
        builder.push_default(StyleProperty::FontStack(self.font_family.clone()));
        builder.push_default(StyleProperty::FontStyle(self.style));
        builder.push_default(StyleProperty::FontWeight(self.weight));
        builder.push_default(StyleProperty::Strikethrough(self.strikethrough));
        builder.push_default(StyleProperty::Underline(self.underline));
        builder.push_default(StyleProperty::LineHeight(self.line_height.resolve(cx) as f32));
        builder.push_default(StyleProperty::LetterSpacing(self.letter_spacing.resolve(cx) as f32));
    }
}

/// A text element.
#[derive(Clone)]
pub struct Text<Str, S: ?Sized> {
    /// The text content of the label.
    text: Str,
    /// The unstyled part of the text.
    unstyled: UnstyledText,
    /// The style of the text.
    style: S,
}

impl<Str> Text<Str, ()> {
    /// Creates a new text element.
    pub fn new(text: Str) -> Self {
        Self {
            text,
            unstyled: UnstyledText {
                position: Point::ZERO,
                size: SetSize::relaxed(),
                alignment: Alignment::Start,
                break_lines: true,
                dirty_state: DirtyState::Text,
                layout: parley::Layout::new(),
            },
            style: (),
        }
    }

    /// Creates a new text element with basic style.
    pub fn with_basic_style(self) -> Text<Str, BasicTextStyle> {
        Text {
            text: self.text,
            unstyled: self.unstyled,
            style: BasicTextStyle {
                brush: Color::BLACK.into(),
                weight: FontWeight::NORMAL,
                style: FontStyle::Normal,
                underline: false,
                strikethrough: false,
                font_size: Length::Pixels(16.0),
                font_family: GenericFamily::SansSerif.into(),
                line_height: Length::Pixels(1.0),
                letter_spacing: Length::Pixels(0.0),
            },
        }
    }
}

impl<Str> Text<Str, BasicTextStyle> {
    /// Sets the brush used to draw the text.
    pub fn with_brush(mut self, brush: impl Into<Brush>) -> Self {
        self.style.brush = brush.into();
        self
    }

    /// Sets the weight of the text.
    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.style.weight = weight;
        self
    }

    /// Sets the style of the text.
    pub fn with_style(mut self, style: FontStyle) -> Self {
        self.style.style = style;
        self
    }

    /// Sets whether the text should be underlined.
    pub fn with_underline(mut self, underline: bool) -> Self {
        self.style.underline = underline;
        self
    }

    /// Sets whether the text should be struck through.
    pub fn with_strikethrough(mut self, strikethrough: bool) -> Self {
        self.style.strikethrough = strikethrough;
        self
    }

    /// Sets the font size of the text.
    pub fn with_font_size(mut self, font_size: Length) -> Self {
        self.style.font_size = font_size;
        self
    }

    /// Sets the font family of the text.
    pub fn with_font_family(mut self, font_family: impl Into<FontStack<'static>>) -> Self {
        self.style.font_family = font_family.into();
        self
    }

    /// Sets the line height of the text.
    pub fn with_line_height(mut self, line_height: Length) -> Self {
        self.style.line_height = line_height;
        self
    }

    /// Sets the letter spacing of the text.
    pub fn with_letter_spacing(mut self, letter_spacing: Length) -> Self {
        self.style.letter_spacing = letter_spacing;
        self
    }
}

impl<Str, S> Text<Str, S> {
    /// Sets the alignment of the text.
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.unstyled.alignment = alignment;
        self
    }

    /// Sets whether lines should be allowed to break.
    pub fn with_break_lines(mut self, break_lines: bool) -> Self {
        self.unstyled.break_lines = break_lines;
        self
    }
}

impl<Str, S: ?Sized> Text<Str, S>
where
    Str: AsRef<str>,
{
    /// Returns an exclusive reference to the text content of the text element.
    ///
    /// Calling this method will automatically invalidate the text content of the element.
    pub fn text_mut(&mut self) -> &mut Str {
        self.unstyled.add_dirt(DirtyState::Text);
        &mut self.text
    }

    /// Sets the alignment of the text.
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.unstyled.set_alignment(alignment);
    }

    /// Sets whether lines should be allowed to break.
    pub fn set_break_lines(&mut self, break_lines: bool) {
        self.unstyled.set_break_lines(break_lines);
    }

    /// Notifies the text element that the style it uses has changed.
    pub fn notify_style_changed(&mut self) {
        self.unstyled.add_dirt(DirtyState::Text);
    }

    /// Returns an exclusive reference to the style of the text.
    ///
    /// # Remarks
    ///
    /// After changing the style, you should call [`Text::notify_style_changed`] to notify the text
    /// element that the style has changed and that the layout should be rebuilt.
    #[inline]
    pub fn style_mut(&mut self) -> &mut S {
        &mut self.style
    }
}

impl<Str, S: TextStyle> Element for Text<Str, S>
where
    Str: AsRef<str>,
{
    #[inline]
    fn set_size(&mut self, cx: &ElemCtx, size: SetSize) {
        self.unstyled.set_size(cx, size, &mut self.style);
    }

    #[inline]
    fn set_position(&mut self, cx: &ElemCtx, position: Point) {
        self.unstyled.set_position(cx, position);
    }

    #[inline]
    fn metrics(&mut self, cx: &ElemCtx) -> Metrics {
        self.unstyled
            .metrics(cx, self.text.as_ref(), &mut self.style)
    }

    #[inline]
    fn render(&mut self, cx: &ElemCtx, scene: &mut Scene) {
        self.unstyled
            .render(cx, self.text.as_ref(), scene, &mut self.style);
    }

    #[inline]
    fn hit_test(&mut self, _cx: &ElemCtx, _point: Point) -> bool {
        false
    }

    #[inline]
    fn event(&mut self, _cx: &ElemCtx, _event: &dyn Event) -> EventResult {
        EventResult::Ignored
    }
}
