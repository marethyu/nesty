import { Nesty, default as init } from './nesty_web.js';

/* Initialize wasm first */
await init('./nesty_web_bg.wasm');

const SCREEN_WIDTH = 256;
const SCREEN_HEIGHT = 240;

const display = document.getElementById("display");

const nesty = Nesty.new();

let romLoaded = false;

function render() {
    const canvas = document.createElement("canvas");
    canvas.width = 256;
    canvas.height = 240;

    const ctx = canvas.getContext("2d");
    const imageData = ctx.getImageData(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
    const pixels = nesty.pixel_buffer();

    for (let j = 0; j < SCREEN_HEIGHT; j++) {
        for (let i = 0; i < SCREEN_WIDTH; i++) {
            let data_offset = j * SCREEN_WIDTH * 4 + i * 4;
            let pix_offset = j * SCREEN_WIDTH * 3 + i * 3;

            imageData.data[data_offset + 0] = pixels[pix_offset + 0];
            imageData.data[data_offset + 1] = pixels[pix_offset + 1];
            imageData.data[data_offset + 2] = pixels[pix_offset + 2];
            imageData.data[data_offset + 3] = 255;
        }
    }

    ctx.putImageData(imageData, 0, 0);
    display.getContext("2d").drawImage(canvas, 0, 0, 512, 480);
}

function renderLoop() {
    if (romLoaded) {
        nesty.update();
        render();
    }

    requestAnimationFrame(renderLoop);
}

/* Main logic happens here */
console.log("Test");
requestAnimationFrame(renderLoop);

function openROM(e) {
    const romFile = e.target.files[0];
    if (!romFile) {
        return;
    }

    const reader = new FileReader();
    reader.onload = function(e) {
        const rom = new Uint8Array(e.target.result);

        nesty.load_rom(rom);
        nesty.reset();

        romLoaded = true;
    };

    reader.readAsArrayBuffer(romFile);
}

document.getElementById('input').addEventListener('change', openROM, false);

display.addEventListener('keydown', (event) => {
    if (event.code == "KeyA")       nesty.press_key(0);
    if (event.code == "KeyS")       nesty.press_key(1);
    if (event.code == "Space")      nesty.press_key(2);
    if (event.code == "Enter")      nesty.press_key(3);
    if (event.code == "ArrowUp")    nesty.press_key(4);
    if (event.code == "ArrowDown")  nesty.press_key(5);
    if (event.code == "ArrowLeft")  nesty.press_key(6);
    if (event.code == "ArrowRight") nesty.press_key(7);
}, false);

display.addEventListener('keyup', (event) => {
    if (event.code == "KeyA")       nesty.release_key(0);
    if (event.code == "KeyS")       nesty.release_key(1);
    if (event.code == "Space")      nesty.release_key(2);
    if (event.code == "Enter")      nesty.release_key(3);
    if (event.code == "ArrowUp")    nesty.release_key(4);
    if (event.code == "ArrowDown")  nesty.release_key(5);
    if (event.code == "ArrowLeft")  nesty.release_key(6);
    if (event.code == "ArrowRight") nesty.release_key(7);
}, false);
