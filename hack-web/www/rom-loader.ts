import pong from './roms/pong.hack';

export class RomLoader {
  el: HTMLElement;

  constructor(onLoad: (rom: string) => void) {
    this.el = document.createElement('div');

    const select = document.createElement('select');
    const option = document.createElement('option');
    option.value = 'pong';
    option.innerText = 'Pong';
    select.append(option);
    this.el.append(select);

    const loadBtn = document.createElement('button');
    loadBtn.innerText = 'Load ROM';
    loadBtn.addEventListener('click', async () => {
      const rsp = await fetch(pong);
      onLoad(await rsp.text());
    });
    this.el.append(loadBtn);
  }
}