import * as wasm from "./generator_bg.wasm";
import { __wbg_set_wasm } from "./generator_bg.js";
__wbg_set_wasm(wasm);
export * from "./generator_bg.js";
