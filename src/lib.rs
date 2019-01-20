#![crate_type = "lib"]
#![crate_name = "st7735"]

//! # ST7735
//!
//! Driver for the ST7735 TFT.
//! [todo]

#[macro_use]
extern crate num_derive;

use num;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SPI_MODE_0};
use sysfs_gpio::{Direction, Pin};
use std::thread::sleep;
use std::time::Duration;
use std::mem::transmute;

pub struct ST7734 {
    /// Reset pin.
    rst: Option<Pin>,

    /// SPI clock pin.
    clk: Pin,

    /// Data/command pin.
    dc: Pin,

    /// MOSI pin.
    mosi: Pin,
}

#[derive(FromPrimitive, ToPrimitive)]
/// System instructions.
pub enum Instruction {
    NOP     = 0x00,
    SWRESET = 0x01,
    RDDID   = 0x04,
    RDDST   = 0x09,
    SLPIN   = 0x10,
    SLPOUT  = 0x11,
    PTLON   = 0x12,
    NORON   = 0x13,
    INVOFF  = 0x20,
    INVON   = 0x21,
    DISPOFF = 0x28,
    DISPON  = 0x29,
    CASET   = 0x2A,
    RASET   = 0x2B,
    RAMWR   = 0x2C,
    RAMRD   = 0x2E,
    PTLAR   = 0x30,
    COLMOD  = 0x3A,
    MADCTL  = 0x36,
    FRMCTR1 = 0xB1,
    FRMCTR2 = 0xB2,
    FRMCTR3 = 0xB3,
    INVCTR  = 0xB4,
    DISSET5 = 0xB6,
    PWCTR1  = 0xC0,
    PWCTR2  = 0xC1,
    PWCTR3  = 0xC2,
    PWCTR4  = 0xC3,
    PWCTR5  = 0xC4,
    VMCTR1  = 0xC5,
    RDID1   = 0xDA,
    RDID2   = 0xDB,
    RDID3   = 0xDC,
    RDID4   = 0xDD,
    PWCTR6  = 0xFC,
    GMCTRP1 = 0xE0,
    GMCTRN1 = 0xE1,
}

/// System function command.
pub struct Command {
    instruction: Instruction,
    arguments: Vec<u8>,
    delay: Option<u64>,
}

//https://github.com/arduino-libraries/TFT/blob/master/src/utility/Adafruit_ST7735.cpp

//const RCMD1: Vec<Command> = vec![
//    Command { instruction: Instruction::SWRESET, delay: true, arguments: [150]},
//    Command { instruction: Instruction::SLPOUT, delay: true, arguments: [255]},
//    Command { instruction: Instruction::FRMCTR1, delay: false, arguments: [0x01, 0x2C, 0x2D]},
//    Command { instruction: Instruction::FRMCTR2, delay: false, arguments: [0x01, 0x2C, 0x2D]},
//    Command { instruction: Instruction::FRMCTR3, delay: false, arguments: [0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D]},
//    Command { instruction: Instruction::INVCTR, delay: false, arguments: [0x07]},
//    Command { instruction: Instruction::PWCTR1, delay: false, arguments: [0xA2, 0x02, 0x84]},
//    Command { instruction: Instruction::PWCTR2, delay: false, arguments: [0xC5]},
//    Command { instruction: Instruction::PWCTR3, delay: false, arguments: [0x0A, 0x00]},
//    Command { instruction: Instruction::PWCTR4, delay: false, arguments: [0x8A, 0x2A]},
//    Command { instruction: Instruction::PWCTR5, delay: false, arguments: [0x8A, 0xEE]},
//    Command { instruction: Instruction::VMCTR1, delay: false, arguments: [0x0E]},
//    Command { instruction: Instruction::INVOFF, delay: false, arguments: []},
//    Command { instruction: Instruction::MADCTL, delay: false, arguments: [0xC8]},
//    Command { instruction: Instruction::COLMOD, delay: false, arguments: [0x05]},
//];
//
//const RCMD2_GREEN: Vec<Command> = vec![
//    Command { instruction: Instruction::CASET, delay: false, arguments: [0x00, 0x02, 0x00, 0x7F+0x02]},
//    Command { instruction: Instruction::RASET, delay: false, arguments: [0x00, 0x01, 0x00, 0x9F+0x01]},
//];
//
//const RCMD2_RED: Vec<Command> = vec![
//    Command { instruction: Instruction::CASET, delay: false, arguments: [0x00, 0x00, 0x00, 0x7F]},
//    Command { instruction: Instruction::RASET, delay: false, arguments: [0x00, 0x00, 0x00, 0x9F]},
//];
//
//const RCMD2_GREEN144: Vec<Command> = vec![
//    Command { instruction: Instruction::CASET, delay: false, arguments: [0x00, 0x00, 0x00, 0x7F]},
//    Command { instruction: Instruction::RASET, delay: false, arguments: [0x00, 0x00, 0x00, 0x7F]},
//];
//
//const RCMD2_GREEN160X80: Vec<Command> = vec![
//    Command { instruction: Instruction::CASET, delay: false, arguments: [0x00, 0x00, 0x00, 0x7F]},
//    Command { instruction: Instruction::RASET, delay: false, arguments: [0x00, 0x00, 0x00, 0x9F]},
//];
//
//const RCMD3: Vec<Command> = vec![
//    Command { instruction: Instruction::GMCTRP1, delay: false, arguments: [0x02, 0x1c, 0x07, 0x12, 0x37, 0x32, 0x29, 0x2d, 0x29, 0x25, 0x2B, 0x39, 0x00, 0x01, 0x03, 0x10]},
//    Command { instruction: Instruction::GMCTRN1, delay: false, arguments: [0x03, 0x1d, 0x07, 0x06, 0x2E, 0x2C, 0x29, 0x2D, 0x2E, 0x2E, 0x37, 0x3F, 0x00, 0x00, 0x02, 0x10]},
//    Command { instruction: Instruction::NORON, delay: true, arguments: [10]},
//    Command { instruction: Instruction::DISPON, delay: true, arguments: [100]},
//];


impl ST7734 {
    pub fn new(rst: Option<u64>, clk: u64, dc: u64, mosi: u64) -> ST7734 {
        let clk_pin = Pin::new(clk);
        clk_pin.set_direction(Direction::Out);
        clk_pin.set_value(0).expect("error while setting clock 0");
        let dc_pin = Pin::new(dc);
        dc_pin.set_direction(Direction::Out);
        let mosi_pin = Pin::new(mosi);
        mosi_pin.set_direction(Direction::Out);

        let rst_pin = match rst {
            Some(r) => {
                let pin = Pin::new(r);
                pin.set_direction(Direction::Out);
                Some(pin)
            },
            None => None
        };

        let display = ST7734 {
            rst: rst_pin,
            clk: clk_pin,
            dc: dc_pin,
            mosi: mosi_pin,
        };

        display.init();
        display
    }

    pub fn init(&self) {
        let init_commands: Vec<Command> = vec![
            Command { instruction: Instruction::SWRESET, delay: Some(200), arguments: vec![] },
            Command { instruction: Instruction::SLPOUT, delay: Some(200), arguments: vec![] },
            Command { instruction: Instruction::DISPON, delay: Some(100), arguments: vec![] },
        ];

        self.execute_commands(init_commands);
    }

    fn pulse_clock(&self) {
        self.clk.set_value(1).expect("error while pulsing clock");
        self.clk.set_value(0).expect("error while pulsing clock");
    }

    fn write_byte(&self, value: u8, data: bool) {
        let mode = match data {
            false => 0,
            true => 1
        };

        self.dc.set_value(mode).expect("error while writing byte");

        let mask = 0x80;
        for bit in 0..8 {
            self.mosi.set_value(value & (mask >> bit));
            self.pulse_clock();
        }
    }

    fn write_word(&self, value: u16) {
        let bytes: [u8; 2] = unsafe { transmute(value.to_be()) };
        self.write_byte(bytes[0], true);
        self.write_byte(bytes[1], true);
    }

    fn execute_commands(&self, commands: Vec<Command>) {
        for cmd in &commands {
            self.execute_command(cmd);
        }
    }

    fn execute_command(&self, cmd: &Command) {
        self.write_byte(num::ToPrimitive::to_u8(&cmd.instruction).unwrap(), false);

        match cmd.delay {
            Some(d) => {
                if cmd.arguments.len() > 0 {
                    sleep(Duration::from_millis(d));
                }
            },
            None => {
                for argument in &cmd.arguments {
                    self.write_byte(*argument, true);
                }
            }
        }
    }

    fn write_color(&self, color: u32) {
        let bytes: [u8; 4] = unsafe { transmute(color.to_be()) };
        self.write_byte(bytes[1], true);
        self.write_byte(bytes[2], true);
        self.write_byte(bytes[3], true);
    }

    fn set_address_window(&self, x0: u16, y0: u16, x1: u16, y1: u16) {
        self.write_byte(num::ToPrimitive::to_u8(&Instruction::CASET).unwrap(), false);
        self.write_word(x0);
        self.write_word(x1);
        self.write_byte(num::ToPrimitive::to_u8(&Instruction::RASET).unwrap(), false);
        self.write_word(y0);
        self.write_word(y1);
    }

    pub fn draw_pixel(&self, x: u16, y: u16, color: u32) {
        self.set_address_window(x, y, x, y);
        self.write_byte(num::ToPrimitive::to_u8(&Instruction::RAMWR).unwrap(), false);
        self.write_color(color);
    }

    pub fn draw_rect(&self, x0: u16, y0: u16, x1: u16, y1: u16, color: u32) {
        let width = x1 - x0 + 1;
        let height = y1 - y0 + 1;
        self.set_address_window(x0, y0, x1, y1);
        self.write_byte(num::ToPrimitive::to_u8(&Instruction::RAMWR).unwrap(), false);
        for i in 0..(width * height) {
            self.write_color(color);
        }
    }

    pub fn fill_screen(&self, color: u32) {
        self.draw_rect(0, 0, 127, 159, color);
    }

    pub fn clear_screen(&self) {
        self.draw_rect(0, 0, 127, 159, 0x0);
    }
}