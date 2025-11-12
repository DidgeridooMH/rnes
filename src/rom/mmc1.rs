use bitfield::bitfield;
use std::{cell::RefCell, rc::Rc};

use crate::core::{Addressable, Bus, VRam};

use super::MirrorArrangement;

const SHIFT_REGISTER_INITIAL: u8 = 0x10;
const PRG_RAM_SIZE: usize = 8 * 1024;
const PRG_ROM_SIZE: usize = 16 * 1024;
const CHR_ROM_SIZE: usize = 4 * 1024;

bitfield! {
    #[derive(Copy, Clone, PartialEq)]
    pub struct ControlRegister(u8);
    impl Debug;
    #[inline]
    pub mirroring, set_mirroring: 1, 0;
    #[inline]
    pub prg_mode, set_prg_mode: 3, 2;
    #[inline]
    pub chr_mode, set_chr_mode: 4;
}

pub struct Mmc1 {
    prg_bank_switch: u8,
    chr_bank0_switch: u8,
    chr_bank1_switch: u8,
    sr: u8,
    control: ControlRegister,
    prg_ram: [u8; PRG_RAM_SIZE],
    prg_banks: Vec<[u8; PRG_ROM_SIZE]>,
    chr_banks: Vec<[u8; CHR_ROM_SIZE]>,
    vram: Rc<RefCell<VRam>>,
}

impl Mmc1 {
    pub fn register(
        data: &[u8],
        num_rom_banks: u8,
        num_chr_banks: u8,
        bus: &Rc<RefCell<Bus>>,
        vram_bus: &Rc<RefCell<Bus>>,
        vram: &Rc<RefCell<VRam>>,
    ) {
        let mut cursor = 0;
        let mut prg_banks: Vec<[u8; PRG_ROM_SIZE]> = Vec::with_capacity(num_rom_banks as usize);
        for _ in 0..num_rom_banks {
            prg_banks.push(data[cursor..(cursor + PRG_ROM_SIZE)].try_into().unwrap());
            cursor += PRG_ROM_SIZE;
        }
        let mut chr_banks: Vec<[u8; CHR_ROM_SIZE]> = Vec::with_capacity(num_chr_banks as usize);
        if num_chr_banks > 0 {
            for _ in 0..num_chr_banks {
                chr_banks.push(data[cursor..(cursor + CHR_ROM_SIZE)].try_into().unwrap());
                cursor += CHR_ROM_SIZE;
            }
        } else {
            chr_banks.push([0; CHR_ROM_SIZE]);
            chr_banks.push([0; CHR_ROM_SIZE]);
        }

        let rom = Rc::new(RefCell::new(Self {
            prg_bank_switch: 0,
            chr_bank0_switch: 0,
            chr_bank1_switch: 1,
            sr: SHIFT_REGISTER_INITIAL,
            control: ControlRegister(0x0C),
            prg_ram: [0; PRG_RAM_SIZE],
            prg_banks,
            chr_banks,
            vram: vram.clone(),
        }));

        bus.borrow_mut()
            .register_region(0x6000..=0xFFFF, rom.clone());
        vram_bus.borrow_mut().register_region(0..=0x1FFF, rom);
    }
}

impl Addressable for Mmc1 {
    fn read_byte(&mut self, address: u16) -> Option<u8> {
        match address {
            0..=0xFFF => Some(self.chr_banks[self.chr_bank0_switch as usize][address as usize]),
            0x1000..=0x1FFF => {
                if self.control.chr_mode() {
                    Some(self.chr_banks[self.chr_bank1_switch as usize][address as usize - 0x1000])
                } else {
                    Some(
                        self.chr_banks[self.chr_bank0_switch as usize + 1]
                            [address as usize - 0x1000],
                    )
                }
            }
            0x6000..=0x7FFF => Some(self.prg_ram[address as usize - 0x6000]),
            0x8000..=0xBFFF => match self.control.prg_mode() {
                0 | 1 => Some(
                    self.prg_banks[(self.prg_bank_switch & 0xFE) as usize]
                        [address as usize - 0x8000],
                ),
                2 => Some(self.prg_banks[0][address as usize - 0x8000]),
                3 => Some(self.prg_banks[self.prg_bank_switch as usize][address as usize - 0x8000]),
                _ => unreachable!(),
            },
            0xC000..=0xFFFF => match self.control.prg_mode() {
                0 | 1 => Some(
                    self.prg_banks[(self.prg_bank_switch & 0xFE) as usize + 1]
                        [address as usize - 0xC000],
                ),
                2 => Some(self.prg_banks[self.prg_bank_switch as usize][address as usize - 0xC000]),
                3 => Some(self.prg_banks[self.prg_banks.len() - 1][address as usize - 0xC000]),
                _ => unreachable!(),
            },
            _ => {
                println!("(warn) Unexpected read in MMC1 {address:X}");
                None
            }
        }
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            0..=0xFFF => self.chr_banks[self.chr_bank0_switch as usize][address as usize] = data,
            0x1000..=0x1FFF => {
                if self.control.chr_mode() {
                    self.chr_banks[self.chr_bank1_switch as usize][address as usize - 0x1000] =
                        data;
                } else {
                    self.chr_banks[self.chr_bank0_switch as usize + 1][address as usize - 0x1000] =
                        data;
                }
            }
            0x6000..=0x7FFF => self.prg_ram[address as usize - 0x6000] = data,
            0x8000..=0xFFFF => {
                if data & 0x80 > 0 {
                    self.control.set_prg_mode(3);
                    self.sr = SHIFT_REGISTER_INITIAL;
                } else if self.sr & 1 > 0 {
                    self.sr = (self.sr >> 1) | (0x10 * (data & 1));
                    match address {
                        0x8000..=0x9FFF => {
                            self.control.0 = self.sr;
                            self.vram
                                .borrow_mut()
                                .set_mirroring(match self.control.mirroring() {
                                    0 => MirrorArrangement::OneScreenLower,
                                    1 => MirrorArrangement::OneScreenUpper,
                                    2 => MirrorArrangement::Vertical,
                                    3 => MirrorArrangement::Horizontal,
                                    _ => unreachable!(),
                                });
                        }
                        0xA000..=0xBFFF => self.chr_bank0_switch = self.sr,
                        0xC000..=0xDFFF => self.chr_bank1_switch = self.sr,
                        0xE000..=0xFFFF => {
                            // TODO: I don't understand bit 4
                            if self.sr & 0x10 > 0 {
                                println!("(warn) I don't really know what this bit in MMC1 does.");
                            }
                            self.prg_bank_switch = self.sr & 0xF;
                        }
                        _ => unreachable!(),
                    }
                    self.sr = SHIFT_REGISTER_INITIAL;
                } else {
                    self.sr = (self.sr >> 1) | (0x10 * (data & 1));
                }
            }
            _ => {
                println!("(warn) Unexpected write to MMC1 {address:X}");
            }
        }
    }
}
