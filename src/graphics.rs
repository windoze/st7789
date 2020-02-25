use embedded_graphics::drawable::Pixel;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::image::{Image, ImageDimensions, IntoPixelIter};
use embedded_graphics::pixelcolor::raw::{RawData, RawU16};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{DrawTarget, Size};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::style::{PrimitiveStyle, Styled};

use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

use crate::{Error, Instruction, ST7789};

impl<SPI, DC, RST> DrawTarget<Rgb565> for ST7789<SPI, DC, RST>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    type Error = Error<SPI::Error, DC::Error, RST::Error>;

    fn draw_pixel(&mut self, pixel: Pixel<Rgb565>) -> Result<(), Self::Error> {
        let color = RawU16::from(pixel.1).into_inner();
        let x = pixel.0.x as u16;
        let y = pixel.0.y as u16;

        self.set_pixel(x, y, color)
    }

    fn draw_rectangle(
        &mut self,
        item: &Styled<Rectangle, PrimitiveStyle<Rgb565>>,
    ) -> Result<(), Self::Error> {
        // filled rect can be rendered into frame window directly
        if item.style.fill_color.is_some() {
            let sx = item.top_left().x as u16;
            let sy = item.top_left().y as u16;
            let ex = item.bottom_right().x as u16;
            let ey = item.bottom_right().y as u16;

            self.set_address_window(sx, sy, ex, ey)?;
            self.write_command(Instruction::RAMWR, None)?;
            self.start_data()?;

            for pixel in item.into_iter() {
                let color = Some(RawU16::from(pixel.1).into_inner());

                self.write_word(color.unwrap())?;
            }

            Ok(())
        } else if item.style.stroke_color.is_some() && item.style.stroke_width > 0 {
            
            // TODO: construct rectangle as 4 frames
            self.draw_iter(item)
        } else { // if we don't know what this rect is, draw individual pixels
            self.draw_iter(item)
        }
    }

    fn draw_image<'a, 'b, I>(&mut self, item: &'a Image<'b, I, Rgb565>) -> Result<(), Self::Error>
    where
        &'b I: IntoPixelIter<Rgb565>,
        I: ImageDimensions,
    {
        let sx = item.top_left().x as u16;
        let sy = item.top_left().y as u16;
        let ex = (item.bottom_right().x - 1) as u16;
        let ey = (item.bottom_right().y - 1) as u16;

        self.set_address_window(sx, sy, ex, ey)?;
        self.write_command(Instruction::RAMWR, None)?;
        self.start_data()?;

        for pixel in item.into_iter() {
            let color = RawU16::from(pixel.1).into_inner();
            self.write_word(color)?;
        }

        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(self.size_x.into(), self.size_y.into())
    }
}
