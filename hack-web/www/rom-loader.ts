// Dynamically load all ROMs from the roms folder
const roms: { [k: string]: string } = {};
const requireContext = require.context('./roms', true, /\.hack$/);
requireContext.keys().forEach((key) => (roms[key.replace("./", "").replace(/\.hack$/, "")] = requireContext(key)));

export class RomLoader {
  el: HTMLElement;

  constructor(onLoad: (rom: string) => void) {
    this.el = document.createElement('div');

    const select = document.createElement('select');

    for (const name of Object.keys(roms)) {
      const option = document.createElement('option');
      option.value = name;
      option.innerText = name;
      select.append(option);
    }
    this.el.append(select);

    const loadBtn = document.createElement('button');
    loadBtn.innerText = 'Load ROM';
    loadBtn.addEventListener('click', async () => {
      const rsp = await fetch(roms[select.value]);
      onLoad(await rsp.text());
    });
    this.el.append(loadBtn);
  }
}