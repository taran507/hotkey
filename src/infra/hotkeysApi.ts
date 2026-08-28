import {invoke} from "@tauri-apps/api/core";
import type {Action, Combo, Shortcut} from "../domain/hotkey";

export interface HotkeysApi {
    list(): Promise<Shortcut[]>;

    create(name: string, combo: Combo, action: Action, enabled: boolean): Promise<Shortcut>;

    delete(id: string): Promise<void>;

    update(
        id: string,
        name: string,
        combo: Combo,
        action: Action,
        enabled: boolean,
    ): Promise<Shortcut>;
}

export const tauriHotkeysApi: HotkeysApi = {
    list: () => invoke<Shortcut[]>("list_shortcuts"),
    create: (name, combo, action, enabled) =>
        invoke<Shortcut>("create_shortcut", {name, combo, action, enabled}),
    delete: (id) => invoke("delete_shortcut", {id}),
    update: (id, name, combo, action, enabled) =>
        invoke<Shortcut>("update_shortcut", {id, name, combo, action, enabled}),
};

