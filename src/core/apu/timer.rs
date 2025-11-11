#[derive(Default)]
pub struct Timer {
    current: u16,
    reload: u16,
    reload_now: bool,
}

impl Timer {
    pub fn tick(&mut self) -> bool {
        if self.current == 0 || self.reload_now {
            self.current = self.reload;
            self.reload_now = false;
            true
        } else {
            self.current -= 1;
            false
        }
    }

    pub fn set_reload(&mut self) {
        self.reload_now = true;
    }

    pub fn set_period(&mut self, value: u16) {
        self.reload = value;
    }

    pub fn get_period(&self) -> u16 {
        self.reload
    }
}
