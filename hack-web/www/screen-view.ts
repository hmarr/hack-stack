import { HackEmulator } from "../pkg";

export class ScreenView {
  el: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  emulator: HackEmulator;

  constructor(emulator: HackEmulator) {
    this.emulator = emulator;
    this.el = document.createElement('canvas');
    this.el.width = 512;
    this.el.height = 770;
    this.el.style.width = '512px';
    this.el.style.height = '770px';
    this.ctx = this.el.getContext('2d')!;
    // this.ctx.scale(2, 2);

    this.update();
  }

  update() {
    this.drawMemory(this.emulator.memory.slice(0x0000, 0x4000), 0);
    this.drawMemory(this.emulator.memory.slice(0x4000, 0x6000), 514);
    // this.ctx.fillStyle = 'rgb(200, 200, 200)';
    // this.ctx.clearRect(0, 0, 512, 768);

    // const arrayBuffer = new ArrayBuffer(512 * 768 * 4);
    // const pixels = new Uint8ClampedArray(arrayBuffer);

    // this.emulator.memory.slice(0x0000, 0x6000).forEach((n, wordIndex) => {
    //   for (let bitIndex = 0; bitIndex < 16; bitIndex++) {
    //     const pixelIndex = (wordIndex * 16 + bitIndex) * 4;
    //     pixels[pixelIndex] = 0;
    //     if ((n & (1 << bitIndex)) >> bitIndex !== 0) {
    //       pixels[pixelIndex + 1] = 255;
    //     } else {
    //       pixels[pixelIndex + 1] = 0;
    //     }
    //     pixels[pixelIndex + 2] = 0;
    //     pixels[pixelIndex + 3] = 255;
    //   }
    // });
  }

  drawMemory(memory: Uint16Array, yOffset: number) {
    const height = memory.length / 32;
    const arrayBuffer = new ArrayBuffer(512 * height * 4);
    const pixels = new Uint8ClampedArray(arrayBuffer);

    memory.forEach((n, wordIndex) => {
      for (let bitIndex = 0; bitIndex < 16; bitIndex++) {
        const pixelIndex = (wordIndex * 16 + bitIndex) * 4;
        pixels[pixelIndex] = 0;
        if ((n & (1 << bitIndex)) >> bitIndex !== 0) {
          pixels[pixelIndex + 1] = 255;
        } else {
          pixels[pixelIndex + 1] = 0;
        }
        pixels[pixelIndex + 2] = 0;
        pixels[pixelIndex + 3] = 255;
      }
    });

    const imageData = new ImageData(pixels, 512, height);
    this.ctx.putImageData(imageData, 0, yOffset)
  }
}