import { HackEmulator } from '../pkg'
import { CpuView } from './cpu-view'
import { PerfView } from './perf-view';
import { ScreenView } from './screen-view'
import { RomLoader } from './rom-loader';

class App {
  emulator: HackEmulator;
  debugMode: boolean;
  cpuView: CpuView;
  screenView: ScreenView;
  perfView: PerfView;
  running: boolean;
  lastFrameTime: number;
  startBtn: HTMLButtonElement;
  stepBtn: HTMLButtonElement;

  constructor() {
    this.emulator = new HackEmulator();
    this.debugMode = false;
    this.running = false;
    this.lastFrameTime = performance.now();

    const appEl = document.createElement('div');
    appEl.style.display = 'flex';
    appEl.style.flexDirection = 'row';

    const diagsEl = document.createElement('div');
    diagsEl.style.flex = '1';
    diagsEl.style.display = 'flex';
    diagsEl.style.flexDirection = 'column';
    diagsEl.style.padding = '10px';
    appEl.append(diagsEl);

    this.cpuView = new CpuView(this.emulator);
    diagsEl.append(this.cpuView.el);

    this.perfView = new PerfView();
    this.perfView.el.style.marginTop = '20px';
    diagsEl.append(this.perfView.el);

    const screenEl = document.createElement('div');
    screenEl.style.flex = '1';
    this.screenView = new ScreenView(this.emulator);
    screenEl.append(this.screenView.el);
    appEl.append(screenEl);

    const controlsEl = document.createElement('div');
    controlsEl.style.marginTop = '20px';

    this.startBtn = document.createElement('button');
    this.startBtn.innerText = 'Start';
    this.startBtn.disabled = true;
    this.startBtn.addEventListener('click', () => {
      this.running ? this.stop() : this.start();
    });
    controlsEl.append(this.startBtn);

    this.stepBtn = document.createElement('button');
    this.stepBtn.innerText = 'Step';
    this.stepBtn.disabled = true;
    this.stepBtn.addEventListener('click', () => this.update(1));
    controlsEl.append(this.stepBtn);

    const romLoader = new RomLoader(rom => {
      this.emulator.load_rom(rom);
      this.cpuView.update();
      this.screenView.update();
      this.startBtn.disabled = false;
      this.stepBtn.disabled = false;
    });
    controlsEl.append(romLoader.el);

    diagsEl.append(controlsEl);

    document.body.append(appEl);

    this.toggleDebugMode(false);
  }

  start() {
    this.running = true;
    this.startBtn.innerText = 'Stop';
    this.update(50000);
  }

  stop() {
    this.running = false;
    this.startBtn.innerText = 'Start';
  }

  update(steps: number) {
    const t1 = performance.now();
    try {
      this.emulator.step(steps);
    } catch (e) {
      console.log(e)
    }
    const stepTime = performance.now() - t1;

    if (this.debugMode) {
      this.cpuView.update();
    }

    this.screenView.update();

    if (this.debugMode) {
      const frameTime = performance.now() - this.lastFrameTime;
      this.lastFrameTime = performance.now();
      this.perfView.update(stepTime, frameTime);
    }

    if (this.running) {
      requestAnimationFrame(this.update.bind(this, steps));
    }
  }

  toggleDebugMode(status: boolean) {
    this.debugMode = status;
    this.cpuView.el.style.display = status ? '' : 'none';
    this.perfView.el.style.display = status ? '' : 'none';
    this.stepBtn.style.display = status ? '' : 'none';
  }
}

const app = new App();