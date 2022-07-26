import { NestyWeb, default as init } from './pkg/nesty_web.js';

// Initialize wasm first
await init('./pkg/nesty_web_bg.wasm');

const SCREEN_WIDTH = 256;
const SCREEN_HEIGHT = 240;

const display = document.getElementById("display");
const selector = document.getElementById("samples-select");

const nesty = NestyWeb.new();

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
    nesty.update();
    render();

    requestAnimationFrame(renderLoop);
}

// Main logic happens here
nesty.reset();
requestAnimationFrame(renderLoop);

function loadROM(rom) {
    if (nesty.load_rom(rom)) {
        nesty.reset();
    } else {
        alert("This cartridge is not supported yet bro");
    }
}

function openROM(e) {
    const romFile = e.target.files[0];
    if (!romFile) {
        return;
    }

    const reader = new FileReader();
    reader.onload = function(e) {
        const rom = new Uint8Array(e.target.result);
        loadROM(rom);
    };

    reader.readAsArrayBuffer(romFile);
}

function openROM2(romPath) {
    var xhr = new XMLHttpRequest();

    xhr.open("GET", romPath, true);
    xhr.overrideMimeType("text/plain; charset=x-user-defined");
    xhr.responseType = "arraybuffer";

    xhr.onload = function(e) {
        if (this.status == 200) {
            var rom = new Uint8Array(this.response);
            loadROM(rom);
        }
    }

    xhr.onerror = function(e) {
        alert("XHR error: " + e.target.status);
    };

    xhr.send();
}

document.getElementById('rom-input').addEventListener('change', openROM, false);

display.addEventListener('keydown', (event) => {
    if (event.code == "KeyO")       nesty.save_state();
    else if (event.code == "KeyP")  nesty.load_state();
    else                            nesty.press_key(event.keyCode);
}, false);

display.addEventListener('keyup', (event) => {
    nesty.release_key(event.keyCode);
}, false);

selector.addEventListener("change", () => {
    switch (selector.value) {
        case "nestest": openROM2("./roms/nestest.nes"); break;
        case "sm-forever": openROM2("./roms/Super_Mario_Forever_Clean_Patch.nes"); break;
    }
});

window.addEventListener("keydown", function(e) {
    if(["Space","ArrowUp","ArrowDown","ArrowLeft","ArrowRight"].indexOf(e.code) > -1) {
        e.preventDefault();
    }
}, false);
