import { HackEmulator } from '../pkg'
import { CpuView } from './cpu-view'
import { PerfView } from './perf-view';
import { GLScreenView } from './gl-screen-view'
import { RomLoader } from './rom-loader';

class App {
  emulator: HackEmulator;
  keysPressed: number[];
  debugMode: boolean;
  cpuView: CpuView;
  screenView: GLScreenView;
  perfView: PerfView;
  running: boolean;
  lastFrameTime: number;
  startBtn: HTMLButtonElement;
  stepBtn: HTMLButtonElement;
  speedSlider: HTMLInputElement;

  constructor() {
    this.emulator = new HackEmulator();
    this.keysPressed = [];
    this.debugMode = false;
    this.running = false;
    this.lastFrameTime = performance.now();

    const appEl = document.createElement('div');
    appEl.style.display = 'flex';
    appEl.style.flexDirection = 'column';
    appEl.style.padding = '10px';

    const screenEl = document.createElement('div');
    screenEl.style.flex = '1';
    this.screenView = new GLScreenView(this.emulator);
    screenEl.append(this.screenView.el);
    appEl.append(screenEl);

    const diagsEl = document.createElement('div');
    diagsEl.style.flex = '1';
    diagsEl.style.display = 'flex';
    diagsEl.style.flexDirection = 'column';
    diagsEl.style.padding = '10px';
    appEl.append(diagsEl);

    this.cpuView = new CpuView(this.emulator);
    diagsEl.append(this.cpuView.el);

    this.perfView = new PerfView();
    diagsEl.append(this.perfView.el);

    const controlsEl = document.createElement('div');
    controlsEl.style.display = 'flex';
    controlsEl.style.flexDirection = 'row';

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

    const speedLabel = document.createElement('label');
    speedLabel.innerText = 'Emulation speed:';
    speedLabel.style.color = 'white';
    speedLabel.style.marginLeft = '1.5rem';
    controlsEl.append(speedLabel);

    this.speedSlider = document.createElement('input');
    this.speedSlider.type = 'range';
    this.speedSlider.min = '1';
    this.speedSlider.max = '500000';
    this.speedSlider.value = '200000';
    this.speedSlider.style.marginRight = '1.5rem';
    controlsEl.append(this.speedSlider);

    const romLoader = new RomLoader(this.loadRom.bind(this));
    controlsEl.append(romLoader.el);

    diagsEl.append(controlsEl);

    document.body.append(appEl);
    document.body.style.background = 'rgb(40 36 39)';
    document.body.style.boxSizing = 'border-box';

    this.toggleDebugMode(false);

    this.screenView.update();
  }

  start() {
    this.running = true;
    this.startBtn.innerText = 'Stop';
    document.addEventListener('keydown', this.handleKeydown);
    document.addEventListener('keyup', this.handleKeyup);
    this.update(200000);
  }

  stop() {
    this.running = false;
    this.startBtn.innerText = 'Start';
    document.removeEventListener('keydown', this.handleKeydown);
    document.removeEventListener('keyup', this.handleKeyup);
  }

  loadRom(rom: string) {
    this.emulator.load_rom(rom);
    this.cpuView.update();
    this.screenView.update();
    this.startBtn.disabled = false;
    this.stepBtn.disabled = false;
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
      requestAnimationFrame(this.update.bind(this, parseInt(this.speedSlider.value, 10)));
    }
  }

  toggleDebugMode(status: boolean) {
    this.debugMode = status;
    this.cpuView.el.style.display = status ? '' : 'none';
    this.perfView.el.style.display = status ? '' : 'none';
    this.stepBtn.style.display = status ? '' : 'none';
  }

  handleKeydown = (ev: KeyboardEvent) => {
    const keyCode = KeyMap[ev.code];
    if (!keyCode) {
      return;
    }

    if (this.keysPressed.includes(keyCode)) {
      return;
    }

    this.keysPressed.push(keyCode);
    this.emulator.set_keyboard(keyCode);

    ev.preventDefault();
  };

  handleKeyup = (ev: KeyboardEvent) => {
    const keyCode = KeyMap[ev.code];
    this.keysPressed = this.keysPressed.filter(k => k !== keyCode);

    const newKeyCode = this.keysPressed[this.keysPressed.length - 1] || 0;
    this.emulator.set_keyboard(newKeyCode);
  };
}

const KeyMap: { [keyName: string]: number } = {
  ArrowLeft: 130,
  ArrowUp: 131,
  ArrowRight: 132,
  ArrowDown: 133,
};
for (let i = 'A'.charCodeAt(0); i <= 'Z'.charCodeAt(0); i++) {
  KeyMap[`Key${String.fromCharCode(i)}`] = i;
}
for (let i = '0'.charCodeAt(0); i <= '9'.charCodeAt(0); i++) {
  KeyMap[`Key${String.fromCharCode(i)}`] = i;
}

const app = new App();