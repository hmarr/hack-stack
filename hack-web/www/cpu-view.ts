import { HackEmulator } from '../pkg'

export class CpuView {
  el: HTMLElement;
  emulator: HackEmulator;

  constructor(emulator: HackEmulator) {
    this.emulator = emulator;
    this.el = document.createElement('div');
    this.el.style.fontFamily = 'monospace';

    this.update();
  }

  update() {
    const state = this.emulator.cpu_state();
    this.el.innerText = [
      ` d: ${state.d.toString(16)}`,
      ` a: ${state.a.toString(16)}`,
      ` m: ${state.m.toString(16)}`,
      `pc: ${state.pc.toString(16)}`,
    ].join('\n');
  }
}