mod panic_handler;

use hack_stack::emulator;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct HackEmulator {
    emu: emulator::Emulator,
}

#[wasm_bindgen]
pub struct CpuState {
    pub d: u16,
    pub a: u16,
    pub m: u16,
    pub pc: u16,
}

#[wasm_bindgen]
impl HackEmulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let rom = vec![];
        Self {
            emu: emulator::Emulator::new(rom),
        }
    }

    #[wasm_bindgen]
    pub fn load_rom(&mut self, source: &str) -> Result<(), JsValue> {
        let mut rom = Vec::<u16>::with_capacity(0x2000);
        for line in source.lines().filter(|l| l.len() > 0) {
            rom.push(
                u16::from_str_radix(line.trim_end(), 2)
                    .map_err(|_| format!("error parsing instruction {}", line))?,
            );
        }
        self.emu.load_rom(rom);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn step(&mut self, n: usize) -> Result<(), JsValue> {
        // self.emu.set_keyboard(1);
        for _ in 0..n {
            self.emu.step().map_err(|e| JsValue::from_str(&e))?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn cpu_state(&self) -> CpuState {
        CpuState {
            d: self.emu.cpu.d,
            a: self.emu.cpu.a,
            m: self.emu.cpu.m,
            pc: self.emu.cpu.pc,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn memory(&self) -> js_sys::Uint16Array {
        // Unsafe but appears to be a bit faster than using js_sys::Uint16Array::from
        unsafe { js_sys::Uint16Array::view(self.emu.memory()) }
    }

    // The JS implementation of this runs much faster
    // #[wasm_bindgen]
    // pub fn screen_image_data(&self) -> js_sys::Uint8ClampedArray {
    //     let mut data = Vec::<u8>::with_capacity(512 * 768 * 4);
    //     for (i, word) in self.emu.memory()[0..0x6000].iter().enumerate() {
    //         for bit_index in 0..16 {
    //             let pixel_index = ((i * 16 + bit_index) * 4) as usize;
    //             data.push(if (word >> bit_index) & 1 == 0 { 255 } else { 0 });
    //             data.push(50);
    //             data.push(50);
    //             data.push(255);
    //         }
    //     }
    //     js_sys::Uint8ClampedArray::from(data.as_slice());
    // }
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    panic_handler::set_panic_hook();
    Ok(())
}
