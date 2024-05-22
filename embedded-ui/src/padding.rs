use core::ops::{Add, Sub};

use embedded_graphics::geometry::Point;

use crate::{align::Axis, size::Size};

#[derive(Clone, Copy, Default)]
pub struct Padding {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl Padding {
    pub fn new(top: u32, right: u32, bottom: u32, left: u32) -> Self {
        Self { left, right, top, bottom }
    }

    pub fn new_equal(padding: u32) -> Self {
        Self::new(padding, padding, padding, padding)
    }

    pub fn new_axis(padding_y: u32, padding_x: u32) -> Self {
        Self::new(padding_y, padding_x, padding_y, padding_x)
    }

    pub fn zero() -> Self {
        Self::new_equal(0)
    }

    pub fn total_x(&self) -> u32 {
        self.left + self.right
    }

    pub fn total_y(&self) -> u32 {
        self.top + self.bottom
    }

    pub fn top_left(&self) -> Point {
        Point::new(self.left as i32, self.top as i32)
    }

    pub fn total_axis(&self, axis: Axis) -> u32 {
        match axis {
            Axis::X => self.total_x(),
            Axis::Y => self.total_y(),
        }
    }

    pub fn fit(self, inner: Size, outer: Size) -> Self {
        let free = outer - inner;
        let fit_left = self.top.min(free.width);
        let fit_right = self.right.min(free.width - fit_left);
        let fit_top = self.top.min(free.height);
        let fit_bottom = self.bottom.min(free.height - fit_top);

        Self { left: fit_left, right: fit_right, top: fit_top, bottom: fit_bottom }
    }
}

impl Into<Size> for Padding {
    fn into(self) -> Size {
        Size::new(self.total_x(), self.total_y())
    }
}

impl From<u32> for Padding {
    fn from(value: u32) -> Self {
        Self::new_equal(value)
    }
}

impl From<[u32; 2]> for Padding {
    fn from(value: [u32; 2]) -> Self {
        Self::new_axis(value[0], value[1])
    }
}

impl From<[u32; 4]> for Padding {
    fn from(value: [u32; 4]) -> Self {
        Self::new(value[0], value[1], value[2], value[3])
    }
}

impl Add for Padding {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.top.saturating_add(rhs.top),
            self.right.saturating_add(rhs.right),
            self.bottom.saturating_add(rhs.bottom),
            self.left.saturating_add(rhs.left),
        )
    }
}

impl Sub for Padding {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.top.saturating_sub(rhs.top),
            self.right.saturating_sub(rhs.right),
            self.bottom.saturating_sub(rhs.bottom),
            self.left.saturating_sub(rhs.left),
        )
    }
}
