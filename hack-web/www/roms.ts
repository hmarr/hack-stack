// Dynamically load all ROMs from the roms folder
const roms: { [k: string]: string } = {};
const requireContext = require.context('./roms', true, /\.hack$/);
requireContext.keys().forEach((key) => (roms[key.replace("./", "").replace(/\.hack$/, "")] = requireContext(key)));

export const romNames = Object.keys(roms);

export async function loadRom(name: string) {
  const rsp = await fetch(roms[name]);
  return await rsp.text();
}