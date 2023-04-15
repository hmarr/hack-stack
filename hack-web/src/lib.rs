#![allow(clippy::new_without_default)]

mod panic_handler;

use hack_stack::emulator;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct HackEmulator {
    emu: emulator::Emulator,
    pixel_buffer: Vec<u8>,
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
            pixel_buffer: vec![0u8; 512 * 256 * 4],
        }
    }

    #[wasm_bindgen]
    pub fn load_rom(&mut self, source: &str) -> Result<(), JsValue> {
        let mut rom = Vec::<u16>::with_capacity(0x2000);
        for line in source.lines().filter(|l| !l.is_empty()) {
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
    pub fn set_keyboard(&mut self, keycode: u16) {
        self.emu.set_keyboard(keycode);
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
        // Unsafe, but avoids copying the array, so it's faster than using js_sys::Uint16Array::from
        unsafe { js_sys::Uint16Array::view(self.emu.memory()) }
    }

    #[wasm_bindgen]
    pub fn screen_image_data(&mut self) -> js_sys::Uint8ClampedArray {
        for (i, word) in self.emu.memory()[0x4000..0x6000].iter().enumerate() {
            for bit_index in 0..16 {
                let pixel_index = (i * 16 + bit_index) * 4;
                self.pixel_buffer[pixel_index] = 0;
                self.pixel_buffer[pixel_index + 1] =
                    if (word >> bit_index) & 1 == 0 { 0 } else { 255 };
                self.pixel_buffer[pixel_index + 2] = 0;
                self.pixel_buffer[pixel_index + 3] = 255;
            }
        }

        // Unsafe, but avoids copying the array, so it's faster than using js_sys::Uint8ClampedArray::from
        unsafe { js_sys::Uint8ClampedArray::view(self.pixel_buffer.as_slice()) }
    }
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    panic_handler::set_panic_hook();
    Ok(())
}
