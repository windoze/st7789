#![no_std]

//! This crate provides a ST7789 driver to connect to TFT displays.

pub mod instruction;

use crate::instruction::Instruction;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

#[cfg(feature = "graphics")]
mod graphics;

/// ST7789 driver to connect to TFT displays.
pub struct ST7789<SPI, DC, RST>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    /// SPI
    spi: SPI,

    /// Data/command pin.
    dc: DC,

    /// Reset pin.
    rst: RST,

    /// Screen size
    size_x: u16,
    size_y: u16,
}

/// Display orientation.
#[derive(ToPrimitive)]
pub enum Orientation {
    Portrait = 0b00000000,
    Landscape = 0b01100000,
    PortraitSwapped = 0b11000000,
    LandscapeSwapped = 0b10100000,
}

/// An error holding its source (pins or SPI)
#[derive(Debug)]
pub enum Error<SPIE, DCE, RSTE> {
    Spi(SPIE),
    Dc(DCE),
    Rst(RSTE),
}

impl<SPI, DC, RST> ST7789<SPI, DC, RST>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    ///
    /// Creates a new ST7789 driver instance
    ///
    /// # Arguments
    ///
    /// * `spi` - an SPI interface to use for talking to the display
    /// * `dc` - data/clock pin switch
    /// * `rst` - display hard reset pin
    /// * `size_x` - x axis resolution of the display in pixels
    /// * `size_y` - y axis resolution of the display in pixels
    ///
    pub fn new(spi: SPI, dc: DC, rst: RST, size_x: u16, size_y: u16) -> Self {
        ST7789 {
            spi,
            dc,
            rst,
            size_x,
            size_y,
        }
    }

    ///
    /// Runs commands to initialize the display
    ///
    /// # Arguments
    ///
    /// * `delay` - a delay provided for the MCU/MPU this is running on
    pub fn init<DELAY>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>>
    where
        DELAY: DelayMs<u8>,
    {
        self.hard_reset()?;
        self.write_command(Instruction::SWRESET, None)?; // reset display
        delay.delay_ms(150);
        self.write_command(Instruction::SLPOUT, None)?; // turn off sleep
        delay.delay_ms(10);
        self.write_command(Instruction::INVOFF, None)?; // turn off invert
        self.write_command(Instruction::MADCTL, Some(&[0b00000000]))?; // left -> right, bottom -> top RGB
        self.write_command(Instruction::COLMOD, Some(&[0b01010101]))?; // 16bit 65k colors
        self.write_command(Instruction::INVON, None)?; // hack?
        delay.delay_ms(10);
        self.write_command(Instruction::NORON, None)?; // turn on display
        delay.delay_ms(10);
        self.write_command(Instruction::DISPON, None)?; // turn on display
        delay.delay_ms(10);
        Ok(())
    }

    pub fn hard_reset(&mut self) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.rst.set_high().map_err(Error::Rst)?;
        self.rst.set_low().map_err(Error::Rst)?;
        self.rst.set_high().map_err(Error::Rst)
    }

    fn write_command(
        &mut self,
        command: Instruction,
        params: Option<&[u8]>,
    ) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.dc.set_low().map_err(Error::Dc)?;
        self.spi
            .write(&[command.to_u8().unwrap()])
            .map_err(Error::Spi)?;
        if let Some(params) = params {
            self.start_data()?;
            self.write_data(params)?;
        }
        Ok(())
    }

    fn start_data(&mut self) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.dc.set_high().map_err(Error::Dc)
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.spi.write(data).map_err(Error::Spi)
    }

    /// Writes a data word to the display.
    fn write_word(&mut self, value: u16) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.write_data(&value.to_be_bytes())
    }

    /// Sets display orientation
    pub fn set_orientation(
        &mut self,
        orientation: &Orientation,
    ) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.write_command(Instruction::MADCTL, Some(&[orientation.to_u8().unwrap()]))?;
        Ok(())
    }

    /// Sets the address window for the display.
    fn set_address_window(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
    ) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.write_command(Instruction::CASET, None)?;
        self.start_data()?;
        self.write_word(sx)?;
        self.write_word(ex)?;
        self.write_command(Instruction::RASET, None)?;
        self.start_data()?;
        self.write_word(sy)?;
        self.write_word(ey)
    }

    /// Sets a pixel color at the given coords.
    pub fn set_pixel(
        &mut self,
        x: u16,
        y: u16,
        color: u16,
    ) -> Result<(), Error<SPI::Error, DC::Error, RST::Error>> {
        self.set_address_window(x, y, x, y)?;
        self.write_command(Instruction::RAMWR, None)?;
        self.start_data()?;
        self.write_word(color)
    }
}
