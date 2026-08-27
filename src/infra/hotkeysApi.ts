import {invoke} from "@tauri-apps/api/core";
import type {Action, Combo, Shortcut} from "../domain/hotkey";

export interface HotkeysApi {
    list(): Promise<Shortcut[]>;

    create(name: string, combo: Combo, action: Action): Promise<Shortcut>;

    delete(id: string): Promise<void>;

    update(id: String, name: string, combo: Combo, action: Action): Promise<Shortcut>;
}

export const tauriHotkeysApi: HotkeysApi = {
    list: () => invoke<Shortcut[]>("list_shortcuts"),
    create: (name, combo, action) => invoke<Shortcut>("create_shortcut", {name, combo, action}),
    delete: (id) => invoke("delete_shortcut", {id}),
    update: (id: String, name: string, combo: Combo, action: Action) => invoke("update_shortcut", {
        id,
        name,
        combo,
        action
    }),
};

