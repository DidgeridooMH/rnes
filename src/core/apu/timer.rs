#[derive(Default)]
pub struct Timer {
    current: u16,
    reload: u16,
}

impl Timer {
    pub fn tick(&mut self) -> bool {
        self.current -= 1;
        if self.current == 0 {
            self.current = self.reload;
            true
        } else {
            false
        }
    }

    pub fn set_period(&mut self, value: u16) {
        self.reload = value;
        self.current = value;
    }

    pub fn get_period(&self) -> u16 {
        self.reload
    }
}
