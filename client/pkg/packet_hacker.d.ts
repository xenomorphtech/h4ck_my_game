/* tslint:disable */
/* eslint-disable */

export class WasmGame {
    free(): void;
    [Symbol.dispose](): void;
    challenge_state_json(completed_json: string): string;
    handle_run_request_json(request_json: string): string;
    constructor();
    reset_script_session(scenario_id: string): void;
    run_script_json(scenario_id: string, script: string, append: boolean): string;
    scenarios_json(): string;
}

export function challenge_state_json(completed_json: string): string;

export function scenarios_json(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmgame_free: (a: number, b: number) => void;
    readonly challenge_state_json: (a: number, b: number) => [number, number];
    readonly scenarios_json: () => [number, number];
    readonly wasmgame_challenge_state_json: (a: number, b: number, c: number) => [number, number];
    readonly wasmgame_handle_run_request_json: (a: number, b: number, c: number) => [number, number];
    readonly wasmgame_new: () => number;
    readonly wasmgame_reset_script_session: (a: number, b: number, c: number) => void;
    readonly wasmgame_run_script_json: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly wasmgame_scenarios_json: (a: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
