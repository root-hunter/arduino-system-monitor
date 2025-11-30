use arduino_hal::{
    clock::MHz16,
    hal::{
        port::{PC4, PC5},
        Atmega,
    },
    pac::TWI,
    port::{mode::Input, Pin},
    prelude::*,
    I2c,
};
use heapless::Vec;
use panic_halt as _;
use sh1106::{
    interface::DisplayInterface,
    mode::GraphicsMode,
    prelude::{DisplaySize, I2cInterface},
    Builder,
};
use ufmt::derive;

const FRAME_INTERVAL_MS: u32 = 100;
const PIPELINE_CAPACITY: usize = 32;

#[derive(Debug)]
pub enum FuncType {
    DrawSquare,
    DrawRectangle,
    DrawCircle,
}

#[derive(Debug)]
pub struct Func {
    pub func_type: FuncType,
    pub params: Vec<u32, 4>,
}

pub struct Display {
    display: GraphicsMode<I2cInterface<I2c>>,
    last_frame: u32,
    pub pipeline: Vec<Func, PIPELINE_CAPACITY>,
}

impl Display {
    pub fn build(i2c: arduino_hal::i2c::I2c) -> Self {
        let display: sh1106::mode::GraphicsMode<_> = Builder::new()
            .with_size(DisplaySize::Display128x64)
            .with_rotation(sh1106::displayrotation::DisplayRotation::Rotate0)
            .connect_i2c(i2c)
            .into();

        Display {
            display,
            last_frame: 0,
            pipeline: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        self._display_init();
        self._draw();
    }

    fn _display_init(&mut self) {
        self.display.init().unwrap();
    }

    pub fn clear(&mut self) {
        self.display.clear();
    }

    fn _should_update(&mut self, interval_ms: u32) -> bool {
        let current_ticks = crate::system::System::get_ticks();
        if current_ticks.wrapping_sub(self.last_frame) >= interval_ms {
            self.last_frame = current_ticks;
            true
        } else {
            false
        }
    }

    fn _draw(&mut self) {
        self.clear();

        while let Some(func) = self.pipeline.pop() {
            match func.func_type {
                FuncType::DrawSquare => {
                    if func.params.len() >= 3 {
                        self._draw_square(func.params[0], func.params[1], func.params[2]);
                    }
                }
                FuncType::DrawRectangle => {
                    if func.params.len() >= 4 {
                        self._draw_rectangle(
                            func.params[0],
                            func.params[1],
                            func.params[2],
                            func.params[3],
                        );
                    }
                }
                FuncType::DrawCircle => {
                    if func.params.len() >= 3 {
                        self._draw_circle(func.params[0], func.params[1], func.params[2]);
                    }
                }
            }
        }

        self._flush();
    }

    pub fn update(&mut self) {
        if self._should_update(FRAME_INTERVAL_MS) {
            self._draw();
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, value: u8) {
        if x < 128 && y < 64 {
            self.display.set_pixel(x, y, value);
        }
    }

    fn _flush(&mut self) {
        self.display.flush().unwrap();
    }

    fn _enqueue_func(&mut self, func: Func) {
        if self.pipeline.len() < self.pipeline.capacity() {
            self.pipeline.push(func).unwrap();
        }
    }

    pub fn draw_rectangle(&mut self, x: u32, y: u32, w: u32, h: u32) {
        self._enqueue_func(Func {
            func_type: FuncType::DrawRectangle,
            params: {
                let mut params = Vec::new();
                params.push(x).unwrap();
                params.push(y).unwrap();
                params.push(w).unwrap();
                params.push(h).unwrap();
                params
            },
        });
    }

    fn _draw_rectangle(&mut self, x: u32, y: u32, w: u32, h: u32) {
        for i in 0..w {
            for j in 0..h {
                self.set_pixel(x + i, y + j, 1);
            }
        }
    }

    pub fn draw_square(&mut self, x: u32, y: u32, size: u32) {
        self._enqueue_func(Func {
            func_type: FuncType::DrawSquare,
            params: {
                let mut params = Vec::new();
                params.push(x).unwrap();
                params.push(y).unwrap();
                params.push(size).unwrap();
                params
            },
        });
    }

    fn _draw_square(&mut self, x: u32, y: u32, size: u32) {
        self._draw_rectangle(x, y, size, size);
    }

    pub fn draw_circle(&mut self, x0: u32, y0: u32, radius: u32) {
        self._enqueue_func(Func {
            func_type: FuncType::DrawCircle,
            params: {
                let mut params = Vec::new();
                params.push(x0).unwrap();
                params.push(y0).unwrap();
                params.push(radius).unwrap();
                params
            },
        });
    }

    fn _draw_circle(&mut self, x0: u32, y0: u32, radius: u32) {
        let mut x = radius as i32;
        let mut y = 0i32;
        let mut err = 0i32;

        while x >= y {
            self.set_pixel((x0 as i32 + x) as u32, (y0 as i32 + y) as u32, 1);
            self.set_pixel((x0 as i32 + y) as u32, (y0 as i32 + x) as u32, 1);
            self.set_pixel((x0 as i32 - y) as u32, (y0 as i32 + x) as u32, 1);
            self.set_pixel((x0 as i32 - x) as u32, (y0 as i32 + y) as u32, 1);
            self.set_pixel((x0 as i32 - x) as u32, (y0 as i32 - y) as u32, 1);
            self.set_pixel((x0 as i32 - y) as u32, (y0 as i32 - x) as u32, 1);
            self.set_pixel((x0 as i32 + y) as u32, (y0 as i32 - x) as u32, 1);
            self.set_pixel((x0 as i32 + x) as u32, (y0 as i32 - y) as u32, 1);

            y += 1;
            if err <= 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }
}
