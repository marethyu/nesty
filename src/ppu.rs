use std::cell::RefCell;
use std::sync::{Arc, Weak};

use crate::cartridge::Cartridge;
use crate::mapper::Mirroring;
use crate::io::IO;

use crate::mirror;

use crate::{test_bit, modify_bit, toggle_bit, reverse_byte};

const NAMETABLE_SIZE: usize = 0x400;
const PALETTE_RAM_SIZE: usize = 0x20;

const PT0_START: u16 = 0x0000;
const PT1_START: u16 = 0x1000;
const NT_START: u16 = 0x2000;
const AT_START: u16 = 0x23C0;
const FRAME_PAL_START: u16 = 0x3F00;

const OAM_SIZE: usize = 0x100;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

pub static SYSTEM_PALLETE: [(u8,u8,u8); 64] = [
    (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E), 
    (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00), 
    (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05), 
    (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA), 
    (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00), 
    (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21), 
    (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF), 
    (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12), 
    (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF), 
    (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF), 
    (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0), 
    (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA), 
    (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];

/*
PPU Memory Map (14bit buswidth, 0-3FFFh)
  0000h-0FFFh   Pattern Table 0 (4K) (256 Tiles)
  1000h-1FFFh   Pattern Table 1 (4K) (256 Tiles)
  2000h-23FFh   Name Table 0 and Attribute Table 0 (1K) (32x30 BG Map)
  2400h-27FFh   Name Table 1 and Attribute Table 1 (1K) (32x30 BG Map)
  2800h-2BFFh   Name Table 2 and Attribute Table 2 (1K) (32x30 BG Map)
  2C00h-2FFFh   Name Table 3 and Attribute Table 3 (1K) (32x30 BG Map)
  3000h-3EFFh   Mirror of 2000h-2EFFh
  3F00h-3F1Fh   Background and Sprite Palettes (25 entries used)
                More information: https://www.nesdev.org/wiki/PPU_palettes
  3F20h-3FFFh   Mirrors of 3F00h-3F1Fh
*/
pub struct PPU {
    cart: Weak<RefCell<Cartridge>>, /* for accessing pattern table */

    // Nametable is represented by array of 4 0x400 byte values
    // Index 0 is nametable 0 ($2000-$23FF)
    // Index 1 is nametable 1 ($2400-$27FF)
    // Index 2 is nametable 2 ($2800-$2BFF)
    // Index 3 is nametable 3 ($2C00-$2FFF)
    nametable: Vec<Vec<u8>>,
    palette_ram: Vec<u8>,

    // PPU CONTROL ($2000)
    nametable_x: bool,
    nametable_y: bool,
    vram_increment_downwards: bool,
    sprite_pattern: bool,
    bkgd_pattern: bool,
    sprite_size: bool,
    ms_select: bool,
    nmi_enable: bool,

    // PPU MASK ($2001)
    grayscale: bool,
    render_background_left: bool,
    render_sprites_left: bool,
    render_background: bool,
    render_sprites: bool,
    enhance_red: bool,
    enhance_green: bool,
    enhance_blue: bool,

    // PPU STATUS ($2002)
    unused: bool,
    sprite_overflow: bool, // TODO
    sprite_zero_hit: bool,
    vblank: bool,

    // OAM ADDRESS ($2003)
    oam_addr: u8,

    // PPU SCROLL ($2005)
    scrollx: u8,
    scrolly: u8,

    // PPU ADDRESS ($2006)
    vram_address: u16,

    // PPU DATA ($2007)
    prev_data: u8, /* AKA IO bus for open bus implementation */

    oam: [u8; OAM_SIZE],
    addr_latch: bool,

    scanline: i32,
    cycle: u32,
    odd_frame: bool,

    // For checking whether pixels in background transparent or not
    transparent: [bool; WIDTH * HEIGHT],

    pub pixels: [u8; WIDTH * HEIGHT * 3],
    pub nmi: bool
}

impl PPU {
    pub fn new(cart: Weak<RefCell<Cartridge>>) -> Self {
        PPU {
            cart: cart.clone(),

            nametable: vec![vec![0; NAMETABLE_SIZE]; 4],
            palette_ram: vec![0; PALETTE_RAM_SIZE],

            nametable_x: false,
            nametable_y: false,
            vram_increment_downwards: false,
            sprite_pattern: false,
            bkgd_pattern: false,
            sprite_size: false,
            ms_select: false,
            nmi_enable: false,

            grayscale: false,
            render_background_left: false,
            render_sprites_left: false,
            render_background: false,
            render_sprites: false,
            enhance_red: false,
            enhance_green: false,
            enhance_blue: false,

            unused: false,
            sprite_overflow: false,
            sprite_zero_hit: false,
            vblank: false,

            oam_addr: 0,

            scrollx: 0,
            scrolly: 0,

            vram_address: 0,

            prev_data: 0,

            oam: [0; OAM_SIZE],
            addr_latch: false,

            scanline: 0,
            cycle: 0,
            odd_frame: false,

            transparent: [false; WIDTH * HEIGHT],

            pixels: [0; WIDTH * HEIGHT * 3],
            nmi: false
        }
    }

    fn cart(&self) -> Arc<RefCell<Cartridge>> {
        self.cart.upgrade().expect("Cartridge lost for ppu")
    }

    pub fn reset(&mut self) {
        self.nametable_x = false;
        self.nametable_y = false;
        self.vram_increment_downwards = false;
        self.sprite_pattern = false;
        self.bkgd_pattern = false;
        self.sprite_size = false;
        self.ms_select = false;
        self.nmi_enable = false;

        self.grayscale = false;
        self.render_background_left = false;
        self.render_sprites_left = false;
        self.render_background = false;
        self.render_sprites = false;
        self.enhance_red = false;
        self.enhance_green = false;
        self.enhance_blue = false;

        self.unused = false;
        self.sprite_overflow = false;
        self.sprite_zero_hit = false;
        self.vblank = false;

        self.oam_addr = 0;

        self.scrollx = 0;
        self.scrolly = 0;

        self.vram_address = 0;

        self.prev_data = 0;

        self.addr_latch = false;

        self.scanline = 0;
        self.cycle = 0;
        self.odd_frame = false;

        self.nmi = false;
    }

    pub fn tick(&mut self) {
        match self.scanline {
            -1..=239 => { /* Pre render + visible scanline */
                if self.scanline == -1 && self.cycle == 1 {
                    self.vblank = false; // clear vblank before rendering
                    self.sprite_zero_hit = false;

                    // ntx might change after scrolling so reset
                    self.nametable_x = false;
                    // TODO is resetting nty also necessary?!?
                    // self.nametable_y = false;
                }

                if self.scanline == 0 && self.cycle == 0 && self.odd_frame {
                    self.cycle += 1; // skip if odd
                }

                if self.scanline >= 0 && self.cycle == 258 {
                    self.render_scanline();
                }
            }
            240 => {      /* Post render scanline */
            }
            241 => {      /* Start of VBlank scanline */
                if self.cycle == 1 {
                    self.vblank = true;
                    self.nmi = self.nmi_enable; // set nmi if it's enabled
                }
            }
            _ => {}
        }

        self.cycle += 1;
        if self.cycle == 341 {
            self.cycle = 0;

            self.scanline += 1;
            if self.scanline == 262 {
                self.scanline = -1;
                self.odd_frame = !self.odd_frame;
            }
        }
    }

    pub fn render_scanline(&mut self) {
        if self.render_background {
            self.render_bkgd();
        }

        if self.render_sprites {
            self.render_foreground();
        }
    }

    fn render_bkgd(&mut self) {
        let nn = ((self.nametable_y as u16) << 1) | (self.nametable_x as u16);

        let mut base_nt_addr: u16 = NT_START + nn * (NAMETABLE_SIZE as u16);
        let mut base_attr_addr: u16 = AT_START + nn * (NAMETABLE_SIZE as u16);

        let mut scx_flag = false; // it will be true after base addresses' ntx bit get toggled

        let mut actual_screen_y = (self.scanline as usize) + (self.scrolly as usize);

        // Yuh oh... we need to fix base addresses
        if actual_screen_y >= HEIGHT {
            actual_screen_y -= HEIGHT;

            // addresses follows this format: ....NN..........
            // toggling bit 11 will change nametable y
            toggle_bit!(base_nt_addr, 11);
            toggle_bit!(base_attr_addr, 11);
        }

        let ty = (actual_screen_y as u16) / 8; // which tile?
        let y = (actual_screen_y as u16) % 8; // which row?

        let pattstart = if self.bkgd_pattern { PT1_START } else { PT0_START };

        for screen_x in 0..WIDTH {
            let mut actual_screen_x = screen_x + (self.scrollx as usize);

            // Yuh oh... we need to fix base addresses
            if actual_screen_x >= WIDTH {
                actual_screen_x -= WIDTH;

                if !scx_flag {
                    // addresses follows this format: ....NN..........
                    // toggling bit 10 will change nametable x
                    toggle_bit!(base_nt_addr, 10);
                    toggle_bit!(base_attr_addr, 10);

                    scx_flag = true;
                }
            }

            let tx = (actual_screen_x / 8) as u16; // which tile?
            let x = (actual_screen_x as u16) % 8; // which column?

            let tile_addr = base_nt_addr + ty * 32 + tx;
            let tile_id = self.read_byte(tile_addr);

            // Format for attribute table byte: BR BL TR TL
            //              +----+----+
            //              | TL | TR |
            //              +----+----+
            //              | BL | BR |
            //              +----+----+
            // Remember that each byte in attribute table corresponds to a 2x2 block (each of the 4 block sections is a group of 2x2 tiles) on nametable
            let attr_addr = base_attr_addr + (ty / 4) * 8 + (tx / 4);
            let attr = self.read_byte(attr_addr);
            let tile_palno: u8;

            // Block's row and column (row,col)
            //    +----+------+
            //    | 0,0 | 0,1 |
            //    +-----+-----+
            //    | 1,0 | 1,1 |
            //    +-----+-----+
            // Remember that each block has 4x4 tiles
            let block_row = (ty % 4) / 2;
            let block_col = (tx % 4) / 2;

            /* top left */
            if block_row == 0 && block_col == 0 {
                tile_palno =  attr & 0b00000011;
            }
            /* top right */
            else if block_row == 0 && block_col == 1 {
                tile_palno = (attr & 0b00001100) >> 2;
            }
            /* bottom left */
            else if block_row == 1 && block_col == 0 {
                tile_palno = (attr & 0b00110000) >> 4;
            }
            /* bottom right */
            else {
                tile_palno = (attr & 0b11000000) >> 6;
            }

            // The first colour in frame palette is universal background colour
            // Note that addresses $3F04/$3F08/$3F0C can contain unique data
            let palette_start = FRAME_PAL_START + (tile_palno as u16) * 4;
            let sys_palette_idx = [self.read_byte(FRAME_PAL_START) as usize,
                                   self.read_byte(palette_start + 1) as usize,
                                   self.read_byte(palette_start + 2) as usize,
                                   self.read_byte(palette_start + 3) as usize];

            let lo = self.read_byte(pattstart + (tile_id as u16) * 16 + y);
            let hi = self.read_byte(pattstart + (tile_id as u16) * 16 + y + 8);

            let low = test_bit!(lo, 7 - x) as u16;
            let high = test_bit!(hi, 7 - x) as u16;
            let colour_idx = ((high << 1) | low) as usize;

            let rgb = SYSTEM_PALLETE[sys_palette_idx[colour_idx]];

            let xpos = screen_x as usize;
            let ypos = self.scanline as usize;

            self.transparent[ypos * WIDTH + xpos] = colour_idx == 0;

            let offset = ypos * WIDTH * 3 + xpos * 3;

            self.pixels[offset    ] = rgb.0;
            self.pixels[offset + 1] = rgb.1;
            self.pixels[offset + 2] = rgb.2;
        }
    }

    pub fn debug_show_nt(&mut self, nt_start: u16) {
        let pattstart = if self.bkgd_pattern { PT1_START } else { PT0_START };

        for ty in 0..30 {
            for tx in 0..32 {
                let tile_addr = nt_start + ty * 32 + tx;
                let tile_id = self.read_byte(tile_addr);

                // Format for attribute table byte: BR BL TR TL
                //              +----+----+
                //              | TL | TR |
                //              +----+----+
                //              | BL | BR |
                //              +----+----+
                // Remember that each byte in attribute table corresponds to a 2x2 block (each of the 4 block sections is a group of 2x2 tiles) on nametable
                let attr_addr = nt_start + 0x03C0 + (ty / 4) * 8 + (tx / 4);
                let attr = self.read_byte(attr_addr);
                let tile_palno: u8;

                // Block's row and column (row,col)
                //    +----+------+
                //    | 0,0 | 0,1 |
                //    +-----+-----+
                //    | 1,0 | 1,1 |
                //    +-----+-----+
                // Remember that each block has 4x4 tiles
                let block_row = (ty % 4) / 2;
                let block_col = (tx % 4) / 2;

                /* top left */
                if block_row == 0 && block_col == 0 {
                    tile_palno =  attr & 0b00000011;
                }
                /* top right */
                else if block_row == 0 && block_col == 1 {
                    tile_palno = (attr & 0b00001100) >> 2;
                }
                /* bottom left */
                else if block_row == 1 && block_col == 0 {
                    tile_palno = (attr & 0b00110000) >> 4;
                }
                /* bottom right */
                else {
                    tile_palno = (attr & 0b11000000) >> 6;
                }

                // The first colour in frame palette is universal background colour
                // Note that addresses $3F04/$3F08/$3F0C can contain unique data
                let palette_start = FRAME_PAL_START + (tile_palno as u16) * 4;
                let sys_palette_idx = [self.read_byte(FRAME_PAL_START) as usize,
                                       self.read_byte(palette_start + 1) as usize,
                                       self.read_byte(palette_start + 2) as usize,
                                       self.read_byte(palette_start + 3) as usize];

                for y in 0..8 {
                    let mut lo = self.read_byte(pattstart + (tile_id as u16) * 16 + y);
                    let mut hi = self.read_byte(pattstart + (tile_id as u16) * 16 + y + 8);

                    for x in 0..8 {
                        let low = test_bit!(lo, 7) as u16;
                        let high = test_bit!(hi, 7) as u16;
                        let colour_idx = ((high << 1) | low) as usize;

                        let rgb = SYSTEM_PALLETE[sys_palette_idx[colour_idx]];

                        let xpos = (tx * 8 + x) as usize;
                        let ypos = (ty * 8 + y) as usize;
                        let offset = ypos * WIDTH * 3 + xpos * 3;

                        self.pixels[offset    ] = rgb.0;
                        self.pixels[offset + 1] = rgb.1;
                        self.pixels[offset + 2] = rgb.2;

                        lo <<= 1;
                        hi <<= 1;
                    }
                }
            }
        }
    }

    fn render_foreground(&mut self) {
        // Reversed because sprites with lower OAM indices are drawn in front.
        // For example, sprite 0 is in front of sprite 1, which is in front of sprite 63.
        for i in (0..OAM_SIZE).step_by(4).rev() {
            let spr_x = self.oam[i + 3];
            // Sprite data is delayed by one scanline so to get actual y value, 1 must be added but it will cause superfluous sprites be drawn
            // Implementing an accurate pixel-by-pixel ppu renderer might solve this problem TODO
            let spr_y = self.oam[i];

            // For 8x8 sprites, this is the tile number of this sprite within the pattern table selected in bit 3 of PPUCTRL ($2000).
            // For 8x16 sprites, the PPU ignores the pattern table selection and selects a pattern table from bit 0 of this number.
            let id = self.oam[i + 1];
            let attr = self.oam[i + 2];

            let height = if self.sprite_size { 16 } else { 8 };

            // scanline inside sprite?
            if (self.scanline >= (spr_y as i32)) && (self.scanline < ((spr_y + height) as i32)) {
                let mut y = (self.scanline as u8) - spr_y; // which row in sprite tile?

                let patt_addr: u16;

                if !self.sprite_size {
                    patt_addr = if self.sprite_pattern { PT1_START } else { PT0_START } + (id as u16) * 16;
                } else {
                    patt_addr = if test_bit!(id, 0) { PT1_START } else { PT0_START } + ((id & 0b11111110) as u16) * 16;
                };

                let palno = attr & 0b00000011;

                if test_bit!(attr, 7) {
                    y = 8 - y; // vertical flip
                }

                let palette_start = FRAME_PAL_START + 16 + (palno as u16) * 4;
                let sys_palette_idx = [self.read_byte(FRAME_PAL_START) as usize,
                                       self.read_byte(palette_start + 1) as usize,
                                       self.read_byte(palette_start + 2) as usize,
                                       self.read_byte(palette_start + 3) as usize];

                let mut lo = self.read_byte(patt_addr + (y as u16));
                let mut hi = self.read_byte(patt_addr + (y as u16) + 8);

                if test_bit!(attr, 6) {
                    // horizontal flip
                    reverse_byte!(lo);
                    reverse_byte!(hi);
                }

                for x in 0..8 {
                    if (spr_x as u16) + (x as u16) >= (WIDTH as u16) {
                        break;
                    }

                    let low = test_bit!(lo, 7 - x) as u16;
                    let high = test_bit!(hi, 7 - x) as u16;
                    let colour_idx = ((high << 1) | low) as usize;

                    let rgb = SYSTEM_PALLETE[sys_palette_idx[colour_idx]];

                    let xpos = (spr_x + x) as usize;
                    let ypos = self.scanline as usize;

                    let offset = ypos * WIDTH * 3 + xpos * 3;
                    let mut lets_draw = false;

                    // For each pixel in the background buffer, the corresponding sprite pixel replaces it
                    // only if the sprite pixel is opaque and front priority or if the background pixel is transparent.
                    if self.transparent[ypos * WIDTH + xpos] {
                        lets_draw = true;
                    } else { // else if the background pixel is opaque
                        let opaque = colour_idx > 0;

                        if opaque && i == 0 {
                            // This flag is set as soon as an opaque pixel of the sprite at OAM index 0 intersects an opaque background pixel.
                            self.sprite_zero_hit = true;
                        }

                        lets_draw = opaque && !test_bit!(attr, 5); // only draw if sprites have front priority
                    }

                    if lets_draw {
                        self.pixels[offset    ] = rgb.0;
                        self.pixels[offset + 1] = rgb.1;
                        self.pixels[offset + 2] = rgb.2;
                    }
                }
            }
        }
    }

    pub fn dma_write_oam(&mut self, data: u8) {
        self.oam[self.oam_addr as usize] = data;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    // Learn more about registers' behaviour here https://www.nesdev.org/wiki/PPU_registers
    pub fn read_register(&mut self, register: usize) -> u8 {
        let mut data: u8 = 0;
        match register {
            0x2 => { // PPU STATUS
                modify_bit!(self.prev_data, 5, self.sprite_overflow);
                modify_bit!(self.prev_data, 6, self.sprite_zero_hit);
                modify_bit!(self.prev_data, 7, self.vblank);

                data = self.prev_data;

                self.vblank = false;
                self.addr_latch = false;
            }
            0x4 => { // OAM DATA
                data = self.oam[self.oam_addr as usize];
            }
            0x7 => { // PPU DATA
                data = self.prev_data; // reads from nametable are delayed by one cycle
                self.prev_data = self.read_byte(self.vram_address);

                // the current address was in the palette range
                if self.vram_address >= 0x3F00 {
                    data = self.prev_data;
                }

                self.vram_address += if self.vram_increment_downwards { 32 } else { 1 };
                if self.vram_address > 0x3FFF {
                    self.vram_address &= 0b11111111111111;
                }
            }
            _ => {}
        }
        data
    }

    pub fn write_register(&mut self, register: usize, data: u8) {
        match register {
            0x0 => { // PPU CONTROL
                self.nametable_x              = test_bit!(data, 0);
                self.nametable_y              = test_bit!(data, 1);
                self.vram_increment_downwards = test_bit!(data, 2);
                self.sprite_pattern           = test_bit!(data, 3);
                self.bkgd_pattern             = test_bit!(data, 4);
                self.sprite_size              = test_bit!(data, 5);
                self.ms_select                = test_bit!(data, 6);
                self.nmi_enable               = test_bit!(data, 7);
            }
            0x1 => { // PPU MASK
                self.grayscale                = test_bit!(data, 0);
                self.render_background_left   = test_bit!(data, 1);
                self.render_sprites_left      = test_bit!(data, 2);
                self.render_background        = test_bit!(data, 3);
                self.render_sprites           = test_bit!(data, 4);
                self.enhance_red              = test_bit!(data, 5);
                self.enhance_green            = test_bit!(data, 6);
                self.enhance_blue             = test_bit!(data, 7);
            }
            0x3 => { // OAM ADDRESS
                self.oam_addr = data;
            }
            0x4 => { // OAM DATA
                self.oam[self.oam_addr as usize] = data;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            0x5 => { // PPU SCROLL
                if !self.addr_latch {
                    self.scrollx = data;
                } else {
                    self.scrolly = data;
                }
                self.addr_latch = !self.addr_latch;
            }
            0x6 => { // PPU ADDRESS
                if !self.addr_latch {
                    // high byte
                    self.vram_address = ((data as u16) << 8) | (self.vram_address & 0b0000000011111111);
                } else {
                    // low byte
                    self.vram_address = (self.vram_address & 0b1111111100000000) | (data as u16);
                }

                if self.vram_address > 0x3FFF {
                    self.vram_address &= 0b11111111111111;
                }

                self.addr_latch = !self.addr_latch;
            }
            0x7 => { // PPU DATA
                self.write_byte(self.vram_address, data);

                self.vram_address += if self.vram_increment_downwards { 32 } else { 1 };
                if self.vram_address > 0x3FFF {
                    self.vram_address &= 0b11111111111111;
                }
            }
            _ => {}
        }
    }

    fn rendering_on(&self) -> bool {
        return self.render_background || self.render_sprites;
    }
}

impl IO for PPU {
    fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.cart().borrow_mut().read_byte(addr),
            0x2000..=0x3EFF => {
                let nt_idx = mirror!(0x2000, addr, NAMETABLE_SIZE * 4) / NAMETABLE_SIZE;
                let nt_addr = mirror!(0x2000, addr, NAMETABLE_SIZE);
                self.nametable[nt_idx][nt_addr]
            }
            0x3F00..=0x3FFF => {
                let addr = mirror!(0x3F00, addr, PALETTE_RAM_SIZE);

                // Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C
                if addr % 0x04 == 0 {
                    return self.palette_ram[addr & 0b00001111];
                } else {
                    return self.palette_ram[addr];
                }
            }
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn read_word(&mut self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000..=0x3EFF => {
                let a = mirror!(0x2000, addr, NAMETABLE_SIZE * 4);
                let nt_addr = mirror!(0x2000, addr, NAMETABLE_SIZE);

                match self.cart().borrow().mirroring() {
                    /* $2000 equals $2800 and $2400 equals $2C00
                            +---+---+
                            | A | B |
                            +---+---+
                            | A | B |
                            +---+---+                        */
                    Mirroring::Vertical => {
                        if (a >= 0x000 && a < 0x400) || (a >= 0x800 && a < 0xC00) {
                            self.nametable[0][nt_addr] = data;
                            self.nametable[2][nt_addr] = data;
                        } else {
                            self.nametable[1][nt_addr] = data;
                            self.nametable[3][nt_addr] = data;
                        }
                    }
                    /* $2000 equals $2400 and $2800 equals $2C00
                            +---+---+
                            | A | A |
                            +---+---+
                            | B | B |
                            +---+---+                        */
                    Mirroring::Horizontial => {
                        if a >= 0x000 && a < 0x800 {
                            self.nametable[0][nt_addr] = data;
                            self.nametable[1][nt_addr] = data;
                        } else {
                            self.nametable[2][nt_addr] = data;
                            self.nametable[3][nt_addr] = data;
                        }
                    }
                }
            }
            0x3F00..=0x3FFF => {
                let addr = mirror!(0x3F00, addr, PALETTE_RAM_SIZE);

                // Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C
                if addr % 0x04 == 0 {
                    self.palette_ram[addr & 0b00001111] = data;
                } else {
                    self.palette_ram[addr] = data;
                }
            }
            _ => panic!("Address out of bounds: {:04X}", addr)
        }
    }

    fn write_word(&mut self, addr: u16, data: u16) {
        self.write_byte(addr, (data & 0xFF) as u8);
        self.write_byte(addr + 1, (data >> 8) as u8);
    }
}
