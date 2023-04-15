use self::cpu::Cpu;

mod cpu;

pub struct Emulator {
    pub cpu: Cpu,
    rom: Vec<u16>,
    memory: Vec<u16>,
}

impl Emulator {
    pub fn new(rom: Vec<u16>) -> Self {
        Self {
            cpu: Cpu::new(),
            rom,
            memory: vec![0; 0x6001],
        }
    }

    pub fn memory(&self) -> &[u16] {
        &self.memory
    }

    pub fn step(&mut self) -> Result<(), String> {
        let instruction = self.fetch_instruction()?;
        let addr = self.cpu.a;
        self.load_memory(addr as usize);
        self.cpu.execute(instruction)?;
        if self.cpu.write_m {
            self.set_memory(addr, self.cpu.m)?;
        }

        Ok(())
    }

    pub fn load_rom(&mut self, rom: Vec<u16>) {
        self.rom = rom;
        self.cpu.reset();
        self.memory.fill(0);
    }

    pub fn set_memory(&mut self, addr: u16, val: u16) -> Result<(), String> {
        match addr {
            0..=0x6000 => {
                self.memory[addr as usize] = val;
                Ok(())
            }
            _ => Err(format!("Out of bounds memory access ({:#x})", addr)),
        }
    }

    pub fn set_keyboard(&mut self, value: u16) {
        self.memory[0x6000] = value;
    }

    fn fetch_instruction(&self) -> Result<u16, String> {
        match self.rom.get(self.cpu.pc as usize) {
            Some(&i) => Ok(i),
            None => Err(format!("Out of bounds ROM access ({:#x})", self.cpu.pc)),
        }
    }

    fn load_memory(&mut self, addr: usize) {
        if let Some(&m) = self.memory.get(addr) {
            self.cpu.m = m;
        }
    }
}
