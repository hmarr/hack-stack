import { loadRom, romNames } from "./roms";

export class RomLoader {
  el: HTMLElement;

  constructor(onLoad: (rom: string) => void) {
    this.el = document.createElement('div');

    const select = document.createElement('select');

    for (const name of romNames) {
      const option = document.createElement('option');
      option.value = name;
      option.innerText = name;
      select.append(option);
    }
    this.el.append(select);

    const loadBtn = document.createElement('button');
    loadBtn.innerText = 'Load ROM';
    loadBtn.addEventListener('click', async () => {
      onLoad(await loadRom(select.value));
    });
    this.el.append(loadBtn);
  }
}