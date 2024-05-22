use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics_core::pixelcolor::PixelColor;

pub trait UiColor: PixelColor + Default {
    fn default_background() -> Self;
    fn default_foreground() -> Self;
    fn transparent() -> Self;
}

impl UiColor for BinaryColor {
    fn default_background() -> Self {
        Self::Off
    }

    fn default_foreground() -> Self {
        Self::On
    }

    fn transparent() -> Self {
        Self::Off
    }
}
