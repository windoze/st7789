use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::prelude::{DrawTarget, Size};
use embedded_graphics_core::{
    pixelcolor::raw::{RawData, RawU16},
    primitives::Rectangle,
};
use embedded_graphics_core::{prelude::OriginDimensions, Pixel};

use embedded_hal::digital::v2::OutputPin;

use crate::{Error, Orientation, ST7789};
use display_interface::WriteOnlyDataCommand;

impl<DI, RST, PinE> DrawTarget for ST7789<DI, RST>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin<Error = PinE>,
{
    type Error = Error<PinE>;
    type Color = Rgb565;

    #[cfg(not(feature = "batch"))]
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let color = RawU16::from(pixel.1).into_inner();
            let x = pixel.0.x as u16;
            let y = pixel.0.y as u16;

            self.set_pixel(x, y, color)?;
        }

        Ok(())
    }

    #[cfg(feature = "batch")]
    fn draw_iter<T>(&mut self, item: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = Pixel<Rgb565>>,
    {
        use crate::batch::DrawBatch;

        self.draw_batch(item)
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        if let Some(bottom_right) = area.bottom_right() {
            let mut count = 0u32;
            let max = area.size.width * area.size.height;

            let mut colors = colors
                .into_iter()
                .take_while(|_| {
                    count += 1;
                    count <= max
                })
                .map(|color| RawU16::from(color).into_inner());

            let sx = area.top_left.x as u16;
            let sy = area.top_left.y as u16;
            let ex = bottom_right.x as u16;
            let ey = bottom_right.y as u16;
            self.set_pixels(sx, sy, ex, ey, &mut colors)
        } else {
            // nothing to draw
            Ok(())
        }
    }

    fn clear(&mut self, color: Rgb565) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        let colors = core::iter::repeat(RawU16::from(color).into_inner()).take(240 * 320); // blank entire HW RAM contents

        match self.orientation {
            Orientation::Portrait | Orientation::PortraitSwapped => {
                self.set_pixels(0, 0, 239, 319, colors)
            }
            Orientation::Landscape | Orientation::LandscapeSwapped => {
                self.set_pixels(0, 0, 319, 239, colors)
            }
        }
    }
}

impl<DI, RST, PinE> OriginDimensions for ST7789<DI, RST>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin<Error = PinE>,
{
    fn size(&self) -> Size {
        Size::new(self.size_x.into(), self.size_y.into()) // visible area, not RAM-pixel size
    }
}
