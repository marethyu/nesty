/* tslint:disable */
/* eslint-disable */
/**
*/
export class NestyWeb {
  free(): void;
/**
* @returns {NestyWeb}
*/
  static new(): NestyWeb;
/**
* @param {Uint8Array} rom_data
*/
  load_rom(rom_data: Uint8Array): void;
/**
*/
  reset(): void;
/**
*/
  save_state(): void;
/**
*/
  load_state(): void;
/**
*/
  update(): void;
/**
* @param {number} keycode
*/
  press_key(keycode: number): void;
/**
* @param {number} keycode
*/
  release_key(keycode: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_nestyweb_free: (a: number) => void;
  readonly nestyweb_new: () => number;
  readonly nestyweb_load_rom: (a: number, b: number) => void;
  readonly nestyweb_reset: (a: number) => void;
  readonly nestyweb_save_state: (a: number) => void;
  readonly nestyweb_load_state: (a: number) => void;
  readonly nestyweb_update: (a: number) => void;
  readonly nestyweb_press_key: (a: number, b: number) => void;
  readonly nestyweb_release_key: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* Synchronously compiles the given `bytes` and instantiates the WebAssembly module.
*
* @param {BufferSource} bytes
*
* @returns {InitOutput}
*/
export function initSync(bytes: BufferSource): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
