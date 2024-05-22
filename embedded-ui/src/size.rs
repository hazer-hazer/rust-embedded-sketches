use core::ops::{Add, Sub};

use embedded_graphics::{geometry::Point, primitives::Rectangle, transform::Transform};

use crate::align::Axis;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Length {
    /// Fills all the remaining space
    Fill,

    /// Shrink to the minimum space
    Shrink,

    /// Fill a portion of available space. Means `100% / Div(N)`
    Div(u16),

    /// Fixed pixels count
    Fixed(u32),
}

impl Length {
    pub fn div_factor(&self) -> u16 {
        match self {
            Length::Fill => 1,
            Length::Fixed(_) | Length::Shrink => 0,
            Length::Div(div) => *div,
        }
    }
}

impl From<u32> for Length {
    fn from(value: u32) -> Self {
        Self::Fixed(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Size<T = u32> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn new_equal(equal: T) -> Self
    where
        T: Copy,
    {
        Self { width: equal, height: equal }
    }

    pub fn new_width(self, width: T) -> Self {
        Self { width, height: self.height }
    }

    pub fn new_height(self, height: T) -> Self {
        Self { width: self.width, height }
    }

    pub fn axis(self, axis: Axis) -> T {
        match axis {
            Axis::X => self.width,
            Axis::Y => self.height,
        }
    }
}

impl Size<u32> {
    pub fn zero() -> Self {
        Self { width: 0, height: 0 }
    }

    pub fn expand(self, by: impl Into<Size>) -> Self {
        let by = by.into();

        Self::new(self.width + by.width, self.height + by.height)
    }
}

impl From<u32> for Size {
    fn from(value: u32) -> Self {
        Self::new(value, value)
    }
}

impl Add for Size<u32> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.width.saturating_add(rhs.width), self.height.saturating_add(rhs.height))
    }
}

impl Sub for Size<u32> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.width.saturating_sub(rhs.width), self.height.saturating_sub(rhs.height))
    }
}

impl Add<u32> for Size<u32> {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        self + Size::new_equal(rhs)
    }
}

impl Sub<u32> for Size<u32> {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        self - Size::new_equal(rhs)
    }
}

impl Size<Length> {
    pub fn fixed_length(width: u32, height: u32) -> Self {
        Self { width: Length::Fixed(width), height: Length::Fixed(height) }
    }

    pub fn shrink() -> Self {
        Self { width: Length::Shrink, height: Length::Shrink }
    }

    pub fn fill() -> Self {
        Self { width: Length::Fill, height: Length::Fill }
    }
}

impl Into<Size<Length>> for Size {
    fn into(self) -> Size<Length> {
        Size::new(Length::Fixed(self.width), Length::Fixed(self.height))
    }
}

impl From<embedded_graphics_core::geometry::Size> for Size {
    fn from(value: embedded_graphics_core::geometry::Size) -> Self {
        Self::new(value.width, value.height)
    }
}

impl Into<embedded_graphics_core::geometry::Size> for Size {
    fn into(self) -> embedded_graphics_core::geometry::Size {
        embedded_graphics_core::geometry::Size::new(self.width, self.height)
    }
}

#[derive(Clone, Copy)]
pub struct Bounds {
    pub position: Point,
    pub size: Size,
}

impl Transform for Bounds {
    fn translate(&self, by: Point) -> Self {
        Self { position: self.position + by, size: self.size }
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.position += by;
        self
    }
}

impl From<Rectangle> for Bounds {
    fn from(value: Rectangle) -> Self {
        Self { position: value.top_left.into(), size: value.size.into() }
    }
}

impl Into<Rectangle> for Bounds {
    fn into(self) -> Rectangle {
        Rectangle { top_left: self.position.into(), size: self.size.into() }
    }
}
