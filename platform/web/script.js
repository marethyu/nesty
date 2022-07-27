import { NestyWeb, default as init } from './pkg/nesty_web.js';

// Initialize wasm first
await init('./pkg/nesty_web_bg.wasm');

const display = document.getElementById("display");
const selector = document.getElementById("samples-select");

const nesty = NestyWeb.new();

function renderLoop() {
    nesty.update();
    requestAnimationFrame(renderLoop);
}

// Main logic happens here
nesty.reset();
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
            nesty.load_rom(rom);
        }
    }

    xhr.onerror = function(e) {
        alert("XHR error: " + e.target.status);
    };

    xhr.send();
}

document.getElementById('rom-input').addEventListener('change', openROM, false);

display.addEventListener('keydown', (event) => {
    if (event.code == "F10")      nesty.save_state();
    else if (event.code == "F11") nesty.load_state();
    else                          nesty.press_key(event.keyCode);
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
    if(["Space","ArrowUp","ArrowDown","ArrowLeft","ArrowRight","F11"].indexOf(e.code) > -1) {
        e.preventDefault();
    }
}, false);
