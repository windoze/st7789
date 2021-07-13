use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::prelude::{DrawTarget, Size};
use embedded_graphics_core::{
    pixelcolor::raw::{RawData, RawU16},
    prelude::PointsIter,
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

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        // filled rect can be rendered into frame window directly

        if let Some(bottom_right) = area.bottom_right() {
            let mut colors = area.points().map(|_| RawU16::from(color).into_inner());

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

    // fn draw_image<'a, 'b, I>(&mut self, item: &'a Image<'b, I, Rgb565>) -> Result<(), Self::Error>
    // where
    //     &'b I: IntoPixelIter<Rgb565>,
    //     I: ImageDimensions,
    // {
    //     // TODO: this is inconsistent in embedded-graphics between Rectangle and Image
    //     // See: https://github.com/jamwaffles/embedded-graphics/issues/182
    //     let sx = item.top_left().x as u16;
    //     let sy = item.top_left().y as u16;
    //     let ex = (item.bottom_right().x - 1) as u16;
    //     let ey = (item.bottom_right().y - 1) as u16;
    //     let colors = item.into_iter().map(|p| RawU16::from(p.1).into_inner());

    //     self.set_pixels(sx, sy, ex, ey, colors)
    // }

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
