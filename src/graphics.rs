use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::ImageDimensions;
use embedded_graphics::prelude::Pixel;
use embedded_graphics::prelude::PixelColor;
use embedded_graphics::prelude::{DrawTarget, IntoStorage, Point, Size};
use embedded_graphics::{
    pixelcolor::raw::{RawData, RawU16},
    primitives::Rectangle,
};

use embedded_hal::digital::v2::OutputPin;

use crate::{Error, Orientation, ST7789};
use display_interface::WriteOnlyDataCommand;

impl<DI, RST, BL, PinE> ST7789<DI, RST, BL>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin<Error = PinE>,
    BL: OutputPin<Error = PinE>,
{
    /// Returns the bounding box for the entire framebuffer.
    fn framebuffer_bounding_box(&self) -> Rectangle {
        let size = match self.orientation {
            Orientation::Portrait | Orientation::PortraitSwapped => Size::new(240, 320),
            Orientation::Landscape | Orientation::LandscapeSwapped => Size::new(320, 240),
        };

        Rectangle::new(
            Point::zero(),
            Point::new(size.width as i32, size.height as i32),
        )
    }
}

impl<DI, RST, BL, PinE> DrawTarget<Rgb565> for ST7789<DI, RST, BL>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin<Error = PinE>,
    BL: OutputPin<Error = PinE>,
{
    type Error = Error<PinE>;

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

    // fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    // where
    //     I: IntoIterator<Item = Self::Color>,
    // {
    //     if let Some(bottom_right) = area.bottom_right() {
    //         let mut count = 0u32;
    //         let max = area.size.width * area.size.height;

    //         let mut colors = colors
    //             .into_iter()
    //             .take_while(|_| {
    //                 count += 1;
    //                 count <= max
    //             })
    //             .map(|color| RawU16::from(color).into_inner());

    //         let sx = area.top_left.x as u16;
    //         let sy = area.top_left.y as u16;
    //         let ex = bottom_right.x as u16;
    //         let ey = bottom_right.y as u16;
    //         self.set_pixels(sx, sy, ex, ey, &mut colors)
    //     } else {
    //         // nothing to draw
    //         Ok(())
    //     }
    // }

    // fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
    //     let area = area.intersection(&self.framebuffer_bounding_box());

    //     if let Some(bottom_right) = area.bottom_right() {
    //         let mut count = 0u32;
    //         let max = area.size.width * area.size.height;

    //         let mut colors = core::iter::repeat(color.into_storage()).take_while(|_| {
    //             count += 1;
    //             count <= max
    //         });

    //         let sx = area.top_left.x as u16;
    //         let sy = area.top_left.y as u16;
    //         let ex = bottom_right.x as u16;
    //         let ey = bottom_right.y as u16;
    //         self.set_pixels(sx, sy, ex, ey, &mut colors)
    //     } else {
    //         // nothing to draw
    //         Ok(())
    //     }
    // }

    fn clear(&mut self, color: Rgb565) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        let color16 = RawU16::from(color).into_inner();
        const DIM1: u16 = 240;
        const DIM2: u16 = 320;
        let colors = (0..(DIM1 as u32 * DIM2 as u32)).map(|_| color16); // blank entire HW RAM contents

        match self.orientation {
            Orientation::Portrait | Orientation::PortraitSwapped => {
                self.set_pixels(0, 0, DIM1 - 1, DIM2 - 1, colors)
            }
            Orientation::Landscape | Orientation::LandscapeSwapped => {
                self.set_pixels(0, 0, DIM2 - 1, DIM1 - 1, colors)
            }
        }
    }

    fn draw_pixel(
        &mut self,
        item: embedded_graphics::drawable::Pixel<Rgb565>,
    ) -> Result<(), Self::Error> {
        self.set_pixel(item.0.x as u16, item.0.y as u16, item.1.into_storage())
    }

    fn size(&self) -> Size {
        Size::new(self.size_x.into(), self.size_y.into()) // visible area, not RAM-pixel size
    }
}

impl<DI, RST, BL, PinE> ImageDimensions for ST7789<DI, RST, BL>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin<Error = PinE>,
    BL: OutputPin<Error = PinE>,
{
    fn width(&self) -> u32 {
        self.size_x as u32
    }

    fn height(&self) -> u32 {
        self.size_y as u32
    }
    // fn size(&self) -> Size {
    //     Size::new(self.size_x.into(), self.size_y.into()) // visible area, not RAM-pixel size
    // }
}
