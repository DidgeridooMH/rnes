const RATE_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 85, 72, 54,
];

#[derive(Default)]
pub struct Dmc {
    pub enabled: bool,

    pub interrupt_flag: bool,
    pub loop_flag: bool,

    pub rate: usize,

    memory_reader: DmcMemoryReader,
    output_unit: DmcOutputUnit,
}

#[derive(Default)]
struct DmcMemoryReader {
    pub sample_address: u16,
    pub sample_length: u16,

    pub sample_buffer: u8,
    pub current_address: u16,
    pub bytes_remaining: u16,
}

#[derive(Default)]
struct DmcOutputUnit {
    pub shift_register: u8,
    pub bits_remaining: u8,
    pub silence_flag: bool,
    pub output_level: u8,
}

impl Dmc {
    pub fn tick(&mut self) {
        if self.output_unit.bits_remaining == 0 {
            self.reset_output_unit();
        }
    }

    fn reset_output_unit(&mut self) {
        self.output_unit.bits_remaining = 8;
        if self.memory_reader.bytes_remaining > 0 {
            self.output_unit.silence_flag = false;
            self.refill_output_unit();
        } else {
            self.output_unit.silence_flag = true;
            self.reset_memory_reader();
        }
    }

    fn refill_output_unit(&mut self) {
        self.output_unit.shift_register = self.memory_reader.sample_buffer;
        self.memory_reader.sample_buffer = 0;
        self.memory_reader.bytes_remaining -= 1;
    }

    fn reset_memory_reader(&mut self) {
        if self.loop_flag {
            self.memory_reader.current_address = self.memory_reader.sample_address;
            self.memory_reader.bytes_remaining = self.memory_reader.sample_length;
        } else {
            self.interrupt_flag = true;
        }
    }

    pub fn get_sample(&self) -> f32 {
        0.0
    }
}
