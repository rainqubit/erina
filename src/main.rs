use std::ops::Add;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddrMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_x,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub status: u8,
    pub program_counter: u16,
    mem: [u8; 0xffff],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
            mem: [0x00; 0xffff],
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn mem_read_u16(&self, addr: u16) -> u16 {
        let lo = self.mem_read(addr) as u16;
        let hi = self.mem_read(addr + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.mem[addr as usize] = data
    }

    fn mem_write_u16(&self, addr: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(addr, lo);
        self.mem_write(addr + 1, hi)
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.mem[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000)
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    fn get_operand_address(&self, mode: &AddrMode) -> u16 {
        match mode {
            AddrMode::Immediate => self.program_counter,

            AddrMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            AddrMode::Absolute => self.mem_read_u16(self.program_counter),

            AddrMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }

            AddrMode::ZeroPage_y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }

            AddrMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddrMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }

            AddrMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode)
            }
        }
    }

    fn lda(&mut self, value: u8) {
        self.register_a = value;
        self.update_z_n_flag(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_z_n_flag(self.register_x);
    }

    fn inx(&mut self) {
        if self.register_x == 0xff {
            self.register_x = 0;
        } else {
            self.register_x = self.register_x + 1;
        }
        self.update_z_n_flag(self.register_x);
    }

    fn update_z_n_flag(&mut self, result: u8) {
        // z flag
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        // n flag
        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    pub fn run(&mut self) {
        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match opscode {
                //BRK
                0x00 => {
                    return;
                }

                //LDA
                0xA9 => {
                    let param = self.mem_read(self.program_counter);
                    self.program_counter += 1;
                    self.register_a = param;

                    self.lda(param);
                }

                //TAX
                0xAA => {
                    self.tax();
                }

                //INX
                0xE8 => {
                    self.inx();
                }

                _ => todo!(""),
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_0xa9_lda_immidiate_load_data() {
        let mut cpu = CPU::new();

        //LDA 0x05
        //BRK
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);

        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();

        // LDA 0x00
        // BRK
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);

        assert!(cpu.status & 0b0000_0010 == 0b10)
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();

        cpu.register_a = 10;

        //TAX
        //BRK
        cpu.load_and_run(vec![0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }
}
