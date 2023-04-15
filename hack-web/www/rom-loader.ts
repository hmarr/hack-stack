import pong from './roms/pong.hack';
import snek from './roms/snek.hack';
import snek2 from './roms/snek2.hack';

const ROMS: { [k: string]: string } = {
  Snek: snek,
  Snek2: snek2,
  Pong: pong,
};

export class RomLoader {
  el: HTMLElement;

  constructor(onLoad: (rom: string) => void) {
    this.el = document.createElement('div');

    const select = document.createElement('select');

    for (const name of Object.keys(ROMS)) {
      const option = document.createElement('option');
      option.value = name;
      option.innerText = name;
      select.append(option);
    }
    this.el.append(select);

    const loadBtn = document.createElement('button');
    loadBtn.innerText = 'Load ROM';
    loadBtn.addEventListener('click', async () => {
      const rsp = await fetch(ROMS[select.value]);
      onLoad(await rsp.text());
    });
    this.el.append(loadBtn);
  }
}