import { HackEmulator } from "../pkg";

export class ScreenView {
  el: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  emulator: HackEmulator;
  pixelBuffer: Uint8ClampedArray;
  imageData: ImageData;

  constructor(emulator: HackEmulator) {
    this.emulator = emulator;
    this.el = document.createElement('canvas');
    this.el.width = 512;
    this.el.height = 770;
    this.el.style.width = '512px';
    this.el.style.height = '770px';
    this.ctx = this.el.getContext('2d')!;

    const memorySize = 0x6000;
    const screenHeight = memorySize / 32;
    const arrayBuffer = new ArrayBuffer(512 * screenHeight * 4);
    this.pixelBuffer = new Uint8ClampedArray(arrayBuffer);
    this.imageData = new ImageData(this.pixelBuffer, 512, screenHeight);


    this.update();
  }

  update() {
    this.drawMemory(this.emulator.memory.slice(0x0000, 0x4000), 0);
    this.drawMemory(this.emulator.memory.slice(0x4000, 0x6000), 514);
  }

  drawMemory(memory: Uint16Array, yOffset: number) {
    for (let wordIndex = 0; wordIndex < memory.length; wordIndex++) {
      const n = memory[wordIndex];
      for (let bitIndex = 0; bitIndex < 16; bitIndex++) {
        const pixelIndex = (wordIndex * 16 + bitIndex) * 4;
        this.pixelBuffer[pixelIndex] = 0;
        if ((n & (1 << bitIndex)) >> bitIndex !== 0) {
          this.pixelBuffer[pixelIndex + 1] = 255;
        } else {
          this.pixelBuffer[pixelIndex + 1] = 0;
        }
        this.pixelBuffer[pixelIndex + 2] = 0;
        this.pixelBuffer[pixelIndex + 3] = 255;
      }
    }

    this.ctx.putImageData(this.imageData, 0, yOffset)
  }
}