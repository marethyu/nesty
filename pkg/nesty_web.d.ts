/* tslint:disable */
/* eslint-disable */
/**
*/
export class Nesty {
  free(): void;
/**
* @returns {Nesty}
*/
  static new(): Nesty;
/**
* @param {Uint8Array} rom_data
*/
  load_rom(rom_data: Uint8Array): void;
/**
*/
  reset(): void;
/**
*/
  update(): void;
/**
* @returns {Uint8Array}
*/
  pixel_buffer(): Uint8Array;
/**
* @param {number} key
*/
  press_key(key: number): void;
/**
* @param {number} key
*/
  release_key(key: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_nesty_free: (a: number) => void;
  readonly nesty_new: () => number;
  readonly nesty_load_rom: (a: number, b: number) => void;
  readonly nesty_reset: (a: number) => void;
  readonly nesty_update: (a: number) => void;
  readonly nesty_pixel_buffer: (a: number) => number;
  readonly nesty_press_key: (a: number, b: number) => void;
  readonly nesty_release_key: (a: number, b: number) => void;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
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
