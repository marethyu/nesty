use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::savable::Savable;
use crate::{test_bit, modify_bit};

pub const BUTTON_A: u8      = 0;
pub const BUTTON_B: u8      = 1;
pub const BUTTON_SELECT: u8 = 2;
pub const BUTTON_START: u8  = 3;
pub const BUTTON_UP: u8     = 4;
pub const BUTTON_DOWN: u8   = 5;
pub const BUTTON_LEFT: u8   = 6;
pub const BUTTON_RIGHT: u8  = 7;

pub struct Joypad {
    state: u8,
    button_index: u8,
    strobe: bool
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            state: 0,
            button_index: 0,
            strobe: false
        }
    }

    pub fn reset(&mut self) {
        self.state = 0;
        self.button_index = 0;
        self.strobe = false;
    }

    pub fn read(&mut self) -> u8 {
        /* strobe bit on - controller reports only status of the button A on every read */
        if self.strobe {
            return test_bit!(self.state, BUTTON_A) as u8;
        }

        /* strobe bit off - controller cycles through all buttons */

        let button_state: u8;

        // return 1 if all bits read otherwise return next bit
        if self.button_index > BUTTON_RIGHT {
            button_state = 1;
        } else {
            button_state = test_bit!(self.state, self.button_index) as u8;
            self.button_index += 1;
        }

        button_state
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;

        if self.strobe {
            self.button_index = 0;
        }
    }

    pub fn press(&mut self, button: u8) {
        modify_bit!(self.state, button, true);
    }

    pub fn release(&mut self, button: u8) {
        modify_bit!(self.state, button, false);
    }
}

impl Savable for Joypad {
    fn save_state(&self, state: &mut Vec<u8>) {
        state.write_u8(self.state).expect("Unable to save u8");
        state.write_u8(self.button_index).expect("Unable to save u8");
        state.write_u8(self.strobe as u8).expect("Unable to save u8");
    }

    fn load_state(&mut self, state: &mut Cursor<Vec<u8>>) {
        self.state = state.read_u8().expect("Unable to load u8");
        self.button_index = state.read_u8().expect("Unable to load u8");
        self.strobe = state.read_u8().expect("Unable to load u8") != 0;
    }
}
