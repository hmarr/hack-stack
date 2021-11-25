import { HackEmulator } from "../pkg";

export class ScreenView {
  el: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  emulator: HackEmulator;

  constructor(emulator: HackEmulator) {
    this.emulator = emulator;
    this.el = document.createElement('canvas');
    this.el.width = 512;
    this.el.height = 256;
    this.el.style.width = '512px';
    this.el.style.height = '256px';
    this.ctx = this.el.getContext('2d')!;

    this.update();
  }

  update() {
    const screenData = this.emulator.screen_image_data();
    this.ctx.putImageData(new ImageData(screenData, 512, 256), 0, 0);
  }
}