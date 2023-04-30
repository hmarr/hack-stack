# Hack Stack for nand2tetris

Software toolchain for the Hack computer built as part of [nand2tetris](https://www.nand2tetris.org/).

The main toolchain is written in Rust, and lives in the `hack-stack` directory. It includes the following binaries:

- `hack-assemble`: Assembler for the Hack assembly language
- `hack-vm-translate`: Virtual machine translator for the Hack VM language
- `jack-compile`: Compiler for the Jack programming language
- `hack-emulate`: Emulator for the Hack computer

There's also a [web interface](https://hmarr.github.io/hack-stack) for the emulator, which lives in the `hack-web` directory. Under the hood it uses the Rust emulator from hack-stack, compiled to WebAssembly. The rest of it is written in TypeScript, and the frame buffer rendering happens on the GPU using WebGL.

https://user-images.githubusercontent.com/110275/232240322-09289f6b-8410-4f24-83bb-dfce3ef5b72c.mov

## Building the toolchain

You'll need to have the [Rust toolchain](https://www.rust-lang.org/tools/install) installed to compile hack-stack. Then you can run the following commands to build the toolchain:

```sh
cd hack-stack
cargo build --release
```

At that point you should have the following binaries available:

- `target/release/hack-assemble`
- `target/release/hack-vm-translate`
- `target/release/jack-compile`
- `target/release/hack-emulate`

You might want to temporarily add the `target/release` directory to your `PATH` environment variable so you can run these binaries from anywhere.

```sh
export PATH="$PATH:$(pwd)/target/release"
```

## Example toolchain usage

Here's how you could comple a hello world program using the toolchain. As the program uses features from the Jack standard library and OS, you'll need those files available (they're in the `my-hack-os` directory in this example). You can either use the ones provided by the course (downloaded from the [nand2tetris website](https://www.nand2tetris.org/software)), or the Jack files you write during the course.

Note: the maximum ROM size for the Hack computer is 32K words, so you can't compile a program that's larger than that. The course-provided Jack standard library and OS are larger than that, so you might to remove some of the files if you want to compile a program that uses them.

```console
$ ls HelloWorld
Main.jack

$ cat HelloWorld/Main.jack
class Main {
   function void main() {
      do Output.printString("Hello, world!");
      do Output.println();
      return;
   }
}

$ cp my-hack-os/*.jack HelloWorld/

$ jack-compile HelloWorld
Compiled HelloWorld/Math.jack successfully, wrote to HelloWorld/Math.vm
Compiled HelloWorld/Screen.jack successfully, wrote to HelloWorld/Screen.vm
Compiled HelloWorld/Sys.jack successfully, wrote to HelloWorld/Sys.vm
Compiled HelloWorld/Keyboard.jack successfully, wrote to HelloWorld/Keyboard.vm
Compiled HelloWorld/Output.jack successfully, wrote to HelloWorld/Output.vm
Compiled HelloWorld/Memory.jack successfully, wrote to HelloWorld/Memory.vm
Compiled HelloWorld/Array.jack successfully, wrote to HelloWorld/Array.vm
Compiled HelloWorld/Main.jack successfully, wrote to HelloWorld/Main.vm
Compiled HelloWorld/String.jack successfully, wrote to HelloWorld/String.vm

$ hack-vm-translate HelloWorld
Translated HelloWorld successfully, wrote to HelloWorld/HelloWorld.asm

$ hack-assemble HelloWorld/HelloWorld.asm
Assembled HelloWorld/HelloWorld.asm successfully, wrote to HelloWorld/HelloWorld.hack
```

## Web emulator for the Hack computer

You can try the emulator online out by visiting [hmarr.github.io/hack-stack](https://hmarr.github.io/hack-stack).

To run the web emulator yourself, you'll need to have [Node.js](https://nodejs.org/en/), the [Rust toolchain](https://www.rust-lang.org/tools/install), and [wasm-pack](https://rustwasm.github.io/wasm-pack/) installed. Then you can run the following commands to build and run the web emulator:

```sh
cd hack-web
npm install
npm run dev
```

At this point you can visit http://localhost:8080/ in your browser to see the web emulator. You can load a ROM file by clicking the "Load ROM" button, and then clicking the "Run" button to start the emulator.

### Loading custom ROMs

If you've compiled your own `.hack` ROMs, copy them to the "hack-web/www/roms", re-start the web server, and you should be able to load them from the web emulator.

You can also use the `compile-rom.sh` script to compile a your own Jack program and add it to the `roms` directory. First, make sure you've built the Hack toolchain. Then, add your directory of Jack source files to the `programs` directory, and run `compile-rom.sh programs/<program-dir>`. The `programs` directory includes a couple of examples you can compile right away.
