use std::cmp::max;

use super::timer::Timer;
use bitfield::bitfield;

bitfield! {
    #[derive(Copy, Clone, PartialEq)]
    pub struct SweepSetup(u8);
    #[inline]
    pub shift, set_shift: 2, 0;
    #[inline]
    pub negate, set_negate: 3;
    #[inline]
    pub period, set_period: 6, 4;
    #[inline]
    pub enable, set_enable: 7;
}

#[derive(Default)]
pub struct Sweep {
    shift: u16,
    negate: bool,
    enable: bool,
    divider: Timer,
    muted: bool,
}

#[derive(PartialEq)]
pub enum SweepType {
    OneComplement,
    TwoComplement,
}

impl Sweep {
    pub fn setup(&mut self, setup: SweepSetup) {
        self.enable = setup.enable();
        self.divider.set_period(setup.period() as u16);
        self.divider.set_reload();
        self.negate = setup.negate();
        self.shift = setup.shift() as u16;
    }

    pub fn step(&mut self, channel_period: u16, sweep_type: SweepType) -> u16 {
        let mut sweep_amount = (channel_period >> self.shift) as i16;

        let new_channel_period = if self.negate {
            if sweep_type == SweepType::OneComplement {
                sweep_amount -= 1;
            }
            max(channel_period as i16 - sweep_amount, 0) as u16
        } else {
            (channel_period as i16 + sweep_amount) as u16
        };

        self.muted = channel_period < 8 || new_channel_period > 0x7FF;

        if self.enable && self.divider.tick() && !self.muted {
            new_channel_period
        } else {
            channel_period
        }
    }

    pub fn mute(&self) -> bool {
        self.muted
    }
}
