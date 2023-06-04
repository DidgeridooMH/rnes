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
    target: u16,
    shift: u16,
    negate: bool,
    enable: bool,
    period: u8,
    divider: u8,
    reload: bool,
    channel: u8,
}

impl Sweep {
    pub fn new(channel: u8) -> Self {
        Self {
            channel,
            ..Default::default()
        }
    }

    pub fn setup(&mut self, setup: SweepSetup) {
        self.enable = setup.enable();
        self.period = setup.period();
        self.divider = setup.period() + 1;
        self.negate = setup.negate();
        self.shift = setup.shift() as u16;
        self.reload = true;
    }

    pub fn step(&mut self) -> Option<u16> {
        if self.divider == 0 {
            self.divider = self.period;
            self.reload = false;

            if self.enable && !self.mute() && self.target < 0x7FF {
                let target = self.target;
                self.reset_target(target);
                return Some(target);
            }
        } else {
            self.divider -= 1;
        }

        if self.reload {
            self.divider = self.period;
            self.reload = false;
        }

        None
    }

    pub fn reset_target(&mut self, current: u16) {
        let change_amount = current >> self.shift;
        if self.negate {
            if change_amount < current {
                if self.channel == 0 {
                    self.target = current - change_amount - 1;
                } else {
                    self.target = current - change_amount;
                }
            } else {
                self.target = 0;
            }
        } else {
            self.target = current + change_amount;
        }
    }

    pub fn mute(&self) -> bool {
        self.target >= 0x7FF
    }
}
