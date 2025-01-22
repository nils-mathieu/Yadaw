use {
    super::Length,
    crate::{ElemContext, Element, LayoutContext, SizeHint},
    parley::{
        Alignment, FontStack, FontStyle, FontWeight, FontWidth, GenericFamily, Layout,
        PositionedLayoutItem, StyleProperty,
    },
    vello::{
        Glyph, Scene,
        kurbo::{Affine, Point, Size},
        peniko::{self, Brush, Color, Fill},
    },
};

/// A **resource** that is expected to be present in the context.
///
/// It contains the fonts that are available to the application, as well as some other
/// system-wide configuration options.
#[derive(Default)]
pub struct TextResource {
    /// The font context responsible for managing fonts.
    font_ctx: parley::FontContext,
    /// The layout context, allowing re-using allocations between text elements.
    layout_ctx: parley::LayoutContext<Brush>,
}

/// Allows running a function that will be used to style a [`Text`] element.
pub trait TextStyle {
    /// Styles the provided text.
    fn style(
        &self,
        layout_context: &LayoutContext,
        res: &mut TextResource,
        text: &str,
        output: &mut Layout<Brush>,
    );
}

impl TextStyle for () {
    fn style(
        &self,
        _layout_context: &LayoutContext,
        _res: &mut TextResource,
        _text: &str,
        _output: &mut Layout<Brush>,
    ) {
    }
}

#[derive(Clone, Debug)]
pub struct UniformStyle {
    pub brush: Brush,
    pub font_size: Length,
    pub font_stack: FontStack<'static>,
    pub font_width: FontWidth,
    pub font_style: FontStyle,
    pub font_weight: FontWeight,
    pub underline: bool,
    pub underline_offset: Option<Length>,
    pub underline_size: Option<Length>,
    pub underline_brush: Option<Brush>,
    pub strike_through: bool,
    pub strike_through_brush: Option<Brush>,
    pub strike_through_offset: Option<Length>,
    pub strike_through_size: Option<Length>,
    pub line_height: Option<Length>,
    pub word_spacing: Length,
    pub letter_spacing: Length,
}

impl Default for UniformStyle {
    fn default() -> Self {
        Self {
            brush: Color::BLACK.into(),
            font_size: Length::Pixels(16.0),
            font_stack: GenericFamily::Serif.into(),
            font_width: FontWidth::NORMAL,
            font_style: FontStyle::Normal,
            font_weight: FontWeight::NORMAL,
            underline: false,
            underline_offset: None,
            underline_size: None,
            underline_brush: None,
            strike_through: false,
            strike_through_brush: None,
            strike_through_offset: None,
            strike_through_size: None,
            line_height: None,
            word_spacing: Length::Pixels(0.0),
            letter_spacing: Length::Pixels(0.0),
        }
    }
}

impl TextStyle for UniformStyle {
    #[rustfmt::skip]
    fn style(
        &self,
        layout_context: &LayoutContext,
        res: &mut TextResource,
        text: &str,
        output: &mut Layout<Brush>,
    ) {
        let font_size = self.font_size.resolve(layout_context) ;

        let mut builder = res.layout_ctx.ranged_builder(&mut res.font_ctx, text, 1.0);
        builder.push_default(StyleProperty::Brush(self.brush.clone()));
        builder.push_default(StyleProperty::FontSize(font_size as f32));
        builder.push_default(StyleProperty::FontStack(self.font_stack.clone()));
        builder.push_default(StyleProperty::FontWidth(self.font_width));
        builder.push_default(StyleProperty::FontStyle(self.font_style));
        builder.push_default(StyleProperty::FontWeight(self.font_weight));
        builder.push_default(StyleProperty::Underline(self.underline));
        builder.push_default(StyleProperty::UnderlineOffset(self.underline_offset.as_ref().map(|l| l.resolve(layout_context) as f32)));
        builder.push_default(StyleProperty::UnderlineSize(self.underline_size.as_ref().map(|l| l.resolve(layout_context) as f32)));
        builder.push_default(StyleProperty::UnderlineBrush(self.underline_brush.clone()));
        builder.push_default(StyleProperty::Strikethrough(self.strike_through));
        builder.push_default(StyleProperty::StrikethroughOffset(self.strike_through_offset.as_ref().map(|l| l.resolve(layout_context) as f32)));
        builder.push_default(StyleProperty::StrikethroughSize(self.strike_through_size.as_ref().map(|l| l.resolve(layout_context) as f32)));
        builder.push_default(StyleProperty::StrikethroughBrush(self.strike_through_brush.clone()));
        builder.push_default(StyleProperty::LineHeight(self.line_height.as_ref().map_or(1.0, |l| l.resolve(layout_context) / font_size) as f32));
        builder.push_default(StyleProperty::WordSpacing(self.word_spacing.resolve(layout_context) as f32));
        builder.push_default(StyleProperty::LetterSpacing(self.letter_spacing.resolve(layout_context) as f32));
        builder.build_into(output, text);
    }
}

/// Amount of "dirty" a text element can be.
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
enum TextDirtAmount {
    /// The text is completely clean and the layout is ready to be used
    /// for rendering.
    #[default]
    Clean,
    /// The alignment of the text has changed.
    Align,
    /// The lines must be recomputed, but the text itself is still the same.
    Lines,
    /// The text has changed and the layout must be entirely recomputed.
    Text,
}

/// Text that does not include any styling whatsoever.
#[derive(Clone, Default)]
struct UnstyledText {
    /// The text to render.
    text: String,
    /// Whether the label should attempt to wrap text.
    wrap: bool,
    /// The alignment of the text.
    align: Alignment,
    /// Whether the text should take the least amount of space possible vertically.
    inline: bool,

    /// The position of the text.
    position: Point,
    /// The layout context of the last call to `.place`.
    layout_context: LayoutContext,
    /// The width for which the text is expected to be laid out.
    container_width: f32,

    /// The amount of dirt the text has.
    dirt: TextDirtAmount,
    /// The laid out text (if built).
    layout: parley::Layout<peniko::Brush>,
}

impl UnstyledText {
    /// Adds dirt to the text layout.
    fn add_dirt(&mut self, amount: TextDirtAmount) {
        self.dirt = self.dirt.max(amount);
    }

    /// Sets the maximum width of the text.
    fn set_container_width(&mut self, width: f32) {
        if self.container_width != width {
            self.container_width = width;
            self.add_dirt(TextDirtAmount::Lines);
        }
    }

    /// Updates the layout context of the text.
    fn set_layout_context(&mut self, layout_context: LayoutContext) {
        if self.layout_context != layout_context {
            self.layout_context = layout_context;
            self.add_dirt(TextDirtAmount::Text);
        }
    }

    /// Makes sure that the layout of the text is properly computed.
    fn flush(&mut self, elem_context: &ElemContext, style: &mut dyn TextStyle) {
        if self.dirt == TextDirtAmount::Clean {
            return;
        }

        elem_context
            .ctx
            .with_resource_or_default(|text_res: &mut TextResource| {
                if self.dirt >= TextDirtAmount::Text {
                    style.style(&self.layout_context, text_res, &self.text, &mut self.layout);
                }

                if self.dirt >= TextDirtAmount::Lines {
                    let max_advance = if self.wrap {
                        self.container_width
                    } else {
                        f32::INFINITY
                    };
                    self.layout.break_lines().break_remaining(max_advance);
                }

                if self.dirt >= TextDirtAmount::Align {
                    let container_width = if self.inline {
                        None
                    } else {
                        Some(self.container_width)
                    };
                    self.layout.align(container_width, self.align);
                }

                self.dirt = TextDirtAmount::Clean;
            });
    }

    /// Computes the dimensions of the text for the provided space.
    fn size_hint(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        space: Size,
        style: &mut dyn TextStyle,
    ) -> SizeHint {
        self.set_container_width(space.width as f32);
        self.set_layout_context(layout_context);
        self.flush(elem_context, style);

        let preferred = if self.inline {
            Size::new(self.layout.width() as f64, self.layout.height() as f64)
        } else {
            Size::new(space.width, self.layout.height() as f64)
        };

        let min = if self.wrap { Size::ZERO } else { preferred };

        SizeHint {
            preferred,
            min,
            max: Size::new(f64::INFINITY, f64::INFINITY),
        }
    }

    /// Places the element at the provided position and size.
    fn place(&mut self, layout_context: LayoutContext, pos: Point, size: Size) {
        self.position = pos;
        self.set_container_width(size.width as f32);
        self.set_layout_context(layout_context);
    }

    /// Draws the text to the provided scene.
    fn draw(&mut self, elem_context: &ElemContext, scene: &mut Scene, style: &mut dyn TextStyle) {
        self.flush(elem_context, style);

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
                                run.positioned_glyphs().map(|g| Glyph {
                                    id: g.id as u32,
                                    x: g.x,
                                    y: g.y,
                                }),
                            );
                    }
                    PositionedLayoutItem::InlineBox(_box) => {
                        panic!("Inline boxes are not yet supported");
                    }
                }
            }
        }
    }
}

impl std::fmt::Debug for UnstyledText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnstyledText")
            .field("text", &self.text)
            .finish_non_exhaustive()
    }
}

/// An element responsible for rendering text.
#[derive(Clone, Debug, Default)]
pub struct Text<S: ?Sized> {
    /// The unstyled text to render.
    unstyled: UnstyledText,
    /// The instance responsible for adding style to the text.
    style: S,
}

impl<S> Text<S> {
    /// The string that this [`Text`] element will render.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.unstyled.text = text.into();
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Whether the [`Text`] element should wrap text or not.
    pub fn wrap(mut self, yes: bool) -> Self {
        self.unstyled.wrap = yes;
        self.unstyled.add_dirt(TextDirtAmount::Lines);
        self
    }

    /// The alignment of the [`Text`] element.
    pub fn align(mut self, align: Alignment) -> Self {
        self.unstyled.align = align;
        self.unstyled.add_dirt(TextDirtAmount::Align);
        self
    }

    /// Whether the [`Text`] element should take the least amount of space possible vertically.
    pub fn inline(mut self, yes: bool) -> Self {
        self.unstyled.inline = yes;
        self.unstyled.add_dirt(TextDirtAmount::Lines);
        self
    }
}

impl Text<UniformStyle> {
    /// Sets the brush of this [`Text`] element.
    pub fn brush(mut self, brush: impl Into<Brush>) -> Self {
        self.style.brush = brush.into();
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the font size of this [`Text`] element.
    pub fn font_size(mut self, size: Length) -> Self {
        self.style.font_size = size;
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the font stack of this [`Text`] element.
    pub fn font_stack(mut self, stack: FontStack<'static>) -> Self {
        self.style.font_stack = stack;
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the font width of this [`Text`] element.
    pub fn font_width(mut self, width: FontWidth) -> Self {
        self.style.font_width = width;
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the font style of this [`Text`] element.
    pub fn font_style(mut self, style: FontStyle) -> Self {
        self.style.font_style = style;
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the font weight of this [`Text`] element.
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.style.font_weight = weight;
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets whether this [`Text`] element should have an underline.
    pub fn underline(mut self, yes: bool) -> Self {
        self.style.underline = yes;
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the offset of the underline of this [`Text`] element.
    pub fn underline_offset(mut self, offset: Length) -> Self {
        self.style.underline_offset = Some(offset);
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the size of the underline of this [`Text`] element.
    pub fn underline_size(mut self, size: Length) -> Self {
        self.style.underline_size = Some(size);
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the brush of the underline of this [`Text`] element.
    pub fn underline_brush(mut self, brush: impl Into<Brush>) -> Self {
        self.style.underline_brush = Some(brush.into());
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets whether this [`Text`] element should have a strike-through.
    pub fn strike_through(mut self, yes: bool) -> Self {
        self.style.strike_through = yes;
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the offset of the strike-through of this [`Text`] element.
    pub fn strike_through_offset(mut self, offset: Length) -> Self {
        self.style.strike_through_offset = Some(offset);
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the size of the strike-through of this [`Text`] element.
    pub fn strike_through_size(mut self, size: Length) -> Self {
        self.style.strike_through_size = Some(size);
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the brush of the strike-through of this [`Text`] element.
    pub fn strike_through_brush(mut self, brush: impl Into<Brush>) -> Self {
        self.style.strike_through_brush = Some(brush.into());
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the line height of this [`Text`] element.
    pub fn line_height(mut self, height: Length) -> Self {
        self.style.line_height = Some(height);
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the word spacing of this [`Text`] element.
    pub fn word_spacing(mut self, spacing: Length) -> Self {
        self.style.word_spacing = spacing;
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }

    /// Sets the letter spacing of this [`Text`] element.
    pub fn letter_spacing(mut self, spacing: Length) -> Self {
        self.style.letter_spacing = spacing;
        self.unstyled.add_dirt(TextDirtAmount::Text);
        self
    }
}

impl<S: TextStyle> Element for Text<S> {
    fn size_hint(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        space: Size,
    ) -> SizeHint {
        self.unstyled
            .size_hint(elem_context, layout_context, space, &mut self.style)
    }

    fn place(
        &mut self,
        _elem_context: &ElemContext,
        layout_context: LayoutContext,
        pos: Point,
        size: Size,
    ) {
        self.unstyled.place(layout_context, pos, size);
    }

    fn draw(&mut self, elem_context: &ElemContext, scene: &mut Scene) {
        self.unstyled.draw(elem_context, scene, &mut self.style);
    }
}

impl Element for Text<dyn TextStyle> {
    fn size_hint(
        &mut self,
        elem_context: &ElemContext,
        layout_context: LayoutContext,
        space: Size,
    ) -> SizeHint {
        self.unstyled
            .size_hint(elem_context, layout_context, space, &mut self.style)
    }

    fn place(
        &mut self,
        _elem_context: &ElemContext,
        layout_context: LayoutContext,
        pos: Point,
        size: Size,
    ) {
        self.unstyled.place(layout_context, pos, size);
    }

    fn draw(&mut self, elem_context: &ElemContext, scene: &mut Scene) {
        self.unstyled.draw(elem_context, scene, &mut self.style);
    }
}
