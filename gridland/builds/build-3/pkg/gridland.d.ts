/* tslint:disable */
/* eslint-disable */

export class Gridland {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * List of all bots (id, name, x, y) as JSON.
     */
    bots_summary(): string;
    /**
     * Thought bubbles to render over bots — only those with an active thought_ttl or the selected bot.
     * Capped at 6 concurrent bubbles (plus selected) so they don't overlap into soup.
     */
    bubbles(): string;
    buffer_len(): number;
    buffer_ptr(): number;
    canvas_h(): number;
    canvas_w(): number;
    clear_selection(): void;
    /**
     * Clear a tile back to grass (erase user-placed things)
     */
    clear_tile(tx: number, ty: number): void;
    /**
     * Click at pixel coords → select bot if present, else return false.
     * Returns the selected bot id, or -1 if none.
     */
    click_select(px: number, py: number): number;
    current_tick(): number;
    /**
     * Light a campfire — attracts bots, lifts mood, soothes loneliness.
     */
    drop_fire(tx: number, ty: number): void;
    /**
     * Drop a berry at tile coords
     */
    drop_food(tx: number, ty: number): void;
    /**
     * Drop a rock obstacle
     */
    drop_rock(tx: number, ty: number): void;
    event_log(): string;
    constructor(seed: number);
    render(): void;
    select_by_id(id: number): boolean;
    /**
     * Return JSON string with the selected bot's full state.
     */
    selected_info(): string;
    /**
     * Overall world stats summary.
     */
    stats(): string;
    tick(): void;
    tile_size(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_gridland_free: (a: number, b: number) => void;
    readonly gridland_bots_summary: (a: number) => [number, number];
    readonly gridland_bubbles: (a: number) => [number, number];
    readonly gridland_buffer_len: (a: number) => number;
    readonly gridland_buffer_ptr: (a: number) => number;
    readonly gridland_canvas_h: (a: number) => number;
    readonly gridland_clear_selection: (a: number) => void;
    readonly gridland_clear_tile: (a: number, b: number, c: number) => void;
    readonly gridland_click_select: (a: number, b: number, c: number) => number;
    readonly gridland_current_tick: (a: number) => number;
    readonly gridland_drop_fire: (a: number, b: number, c: number) => void;
    readonly gridland_drop_food: (a: number, b: number, c: number) => void;
    readonly gridland_drop_rock: (a: number, b: number, c: number) => void;
    readonly gridland_event_log: (a: number) => [number, number];
    readonly gridland_new: (a: number) => number;
    readonly gridland_render: (a: number) => void;
    readonly gridland_select_by_id: (a: number, b: number) => number;
    readonly gridland_selected_info: (a: number) => [number, number];
    readonly gridland_stats: (a: number) => [number, number];
    readonly gridland_tick: (a: number) => void;
    readonly gridland_tile_size: (a: number) => number;
    readonly gridland_canvas_w: (a: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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
