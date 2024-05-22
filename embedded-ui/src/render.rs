use embedded_graphics::{
    geometry::Point,
    pixelcolor::BinaryColor,
    primitives::{Line, PrimitiveStyleBuilder, RoundedRectangle, StyledDrawable},
    text::renderer::{CharacterStyle, TextRenderer},
};
use embedded_graphics_core::Drawable;
use embedded_graphics_core::{draw_target::DrawTarget, primitives::Rectangle};
use embedded_text::TextBox;

use crate::{
    block::Block,
    color::UiColor,
    size::{Bounds, Size},
};

pub trait Renderer {
    type Color: UiColor + Copy;

    // Renderer info
    fn clear(&mut self);
    fn bounds(&self) -> Bounds;

    // Primitives
    fn fill_rect(&mut self, rect: Rectangle, fill_color: Self::Color);
    fn line(&mut self, start: Point, end: Point, color: Self::Color, width: u32);

    // High-level primitives
    fn block(&mut self, block: &Block<Self::Color>)
    where
        Self: Sized;
    fn text<'a, S>(&mut self, text: &TextBox<'a, S>)
    where
        Self: Sized,
        S: TextRenderer<Color = Self::Color> + CharacterStyle<Color = Self::Color>;
}

pub struct NullRenderer;

impl Renderer for NullRenderer {
    type Color = BinaryColor;

    fn clear(&mut self) {}
    fn bounds(&self) -> Bounds {
        Bounds { position: Point::zero(), size: Size::zero() }
    }

    fn fill_rect(&mut self, _rect: Rectangle, _fill_color: Self::Color) {}
    fn line(&mut self, _from: Point, _to: Point, _color: Self::Color, _width: u32) {}

    fn block(&mut self, _block: &Block<Self::Color>)
    where
        Self: Sized,
    {
    }

    fn text<'a, S>(&mut self, _text: &TextBox<'a, S>)
    where
        Self: Sized,
        S: TextRenderer<Color = Self::Color> + CharacterStyle<Color = Self::Color>,
    {
    }
}

impl<D, C: UiColor> Renderer for D
where
    D: DrawTarget<Color = C>,
    D::Error: core::fmt::Debug,
{
    type Color = C;

    fn clear(&mut self) {
        self.clear(Self::Color::default_background()).unwrap()
    }

    fn bounds(&self) -> Bounds {
        self.bounding_box().into()
    }

    fn fill_rect(&mut self, rect: Rectangle, fill_color: Self::Color) {
        rect.draw_styled(&PrimitiveStyleBuilder::new().fill_color(fill_color).build(), self)
            .unwrap()
    }

    fn line(&mut self, start: Point, end: Point, color: Self::Color, width: u32) {
        Line::new(start, end)
            .draw_styled(
                &PrimitiveStyleBuilder::new().stroke_width(width).stroke_color(color).build(),
                self,
            )
            .unwrap();
    }

    fn block(&mut self, block: &Block<Self::Color>)
    where
        Self: Sized,
    {
        RoundedRectangle::new(block.rect, block.border.radius.into())
            .draw_styled(
                &PrimitiveStyleBuilder::new()
                    .fill_color(block.background)
                    .stroke_color(block.border.color)
                    .stroke_width(block.border.width)
                    .build(),
                self,
            )
            .unwrap();
    }

    fn text<'a, S>(&mut self, text: &TextBox<'a, S>)
    where
        Self: Sized,
        S: TextRenderer<Color = Self::Color> + CharacterStyle<Color = Self::Color>,
    {
        text.draw(self).unwrap();
        // let style = match &text.style.font {
        //     crate::font::Font::Mono(mono) => MonoTextStyleBuilder::<'a, Self::Color>::new()
        //         .font(&mono)
        //         .background_color(Self::Color::default_background())
        //         .text_color(Self::Color::default_foreground())
        //         .build(),
        // };

        // Text::with_text_style(
        //     text.text,
        //     text.position,
        //     style,
        //     TextStyleBuilder::new().alignment(text.align.into()).build(),
        // )
        // .draw(self)
        // .unwrap();
    }
}
