import { Vm } from "chip-eight";
import { memory } from "chip-eight/chipeight_bg";

import { roms } from "./roms";

/*********************************************************
 *  Constants
 *********************************************************/

const PIXEL_SIZE = 12;
const WIDTH = 64;
const HEIGHT = 32;
const ON_COLOR = "#c5c5c5";
const OFF_COLOR = "#000";
const KEYMAP = {
  49: 0x1, // 1
  50: 0x2, // 2
  51: 0x3, // 3
  52: 0xc, // 4
  81: 0x4, // Q
  87: 0x5, // W
  69: 0x6, // E
  82: 0xd, // R
  65: 0x7, // A
  83: 0x8, // S
  68: 0x9, // D
  70: 0xe, // F
  90: 0xa, // Z
  88: 0x0, // X
  67: 0xb, // C
  86: 0xf // V
};

/*********************************************************
 *  Canvas
 *********************************************************/

const canvas = document.getElementById("chip-eight-canvas");
canvas.width = PIXEL_SIZE * WIDTH;
canvas.height = PIXEL_SIZE * HEIGHT;
const ctx = canvas.getContext("2d");

/*********************************************************
 *  Graphics
 *********************************************************/

let req = null;
let keydownHandler = null;
let keyupHandler = null;

function emulateRom(rom) {
  const vm = Vm.new();
  vm.load(rom);

  // Key event handlers
  keydownHandler = window.addEventListener("keydown", event => {
    let idx = KEYMAP[event.keyCode];
    if (idx !== undefined) {
      vm.keyDown(idx);
    }
  });

  keyupHandler = window.addEventListener("keyup", event => {
    let idx = KEYMAP[event.keyCode];
    if (idx !== undefined) {
      vm.keyUp(idx);
    }
  });

  function renderLoop() {
    for (let i = 0; i < 9; i++) {
      if (vm.drawFlag()) {
        drawScreen(vm);
      }
      vm.tick();
    }
    vm.updateTimers();
    req = requestAnimationFrame(renderLoop);
  }

  renderLoop();
}

const getIndex = (row, column) => {
  return row * WIDTH + column;
};

function drawScreen(vm) {
  const screenPtr = vm.screen();
  const screen = new Uint8Array(memory.buffer, screenPtr, WIDTH * HEIGHT);

  for (let row = 0; row < HEIGHT; row++) {
    for (let col = 0; col < WIDTH; col++) {
      const idx = getIndex(row, col);

      ctx.fillStyle = screen[idx] === 1 ? ON_COLOR : OFF_COLOR;

      ctx.fillRect(
        col * PIXEL_SIZE + 1,
        row * PIXEL_SIZE + 1,
        PIXEL_SIZE,
        PIXEL_SIZE
      );
    }
  }

  ctx.stroke();
}

/*********************************************************
 *  DOM
 *********************************************************/

function selectRomHandler(romName) {
  // Cancel handlers
  if (req !== null) {
    cancelAnimationFrame(req);
  }
  if (keydownHandler !== null) {
    window.cancelAnimationFrame(keydownHandler);
  }

  if (keyupHandler !== null) {
    window.cancelAnimationFrame(keyupHandler);
  }

  let rom = roms[romName];
  emulateRom(rom);
}

const header = document.getElementById("header");
Object.keys(roms).map(key => {
  let div = document.createElement("div");
  div.className = "rom";
  div.innerText = key.replace(/([a-z])([A-Z])/g, "$1 $2");
  div.onclick = () => selectRomHandler(key);
  header.appendChild(div);
});
