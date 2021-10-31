export class PerfView {
  el: HTMLElement;
  stepTimes: number[];
  frameTimes: number[];

  constructor() {
    this.el = document.createElement('div');
    this.el.style.fontFamily = 'monospace';
    this.el.style.whiteSpace = 'pre';
    this.stepTimes = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    this.frameTimes = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    this.update(0, 0);
  }

  update(newStepTime: number, newFrameTime: number) {
    this.stepTimes.push(newStepTime);
    this.stepTimes.shift();
    this.frameTimes.push(newFrameTime);
    this.frameTimes.shift();

    const stepTime = Math.round(this.stepTimes.reduce((acc, v) => acc + v) / this.stepTimes.length);
    const frameTime = Math.round(this.frameTimes.reduce((acc, v) => acc + v) / this.frameTimes.length);
    this.el.innerText = [
      ` step time: ${stepTime} ms`,
      `frame time: ${frameTime} ms`,
      `       fps: ${Math.round(1000 / frameTime)}`,
    ].join('\n');
  }
}