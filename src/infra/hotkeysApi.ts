import { invoke } from "@tauri-apps/api/core";
import type { Action, Combo, Shortcut } from "../domain/hotkey";

export interface HotkeysApi {
  list(): Promise<Shortcut[]>;
  create(name: string, combo: Combo, action: Action): Promise<Shortcut>;
  delete(id: string): Promise<void>;
  setEnabled(id: string, enabled: boolean): Promise<void>;
  rename(id: string, name: string): Promise<void>;
}

export const tauriHotkeysApi: HotkeysApi = {
  list: () => invoke<Shortcut[]>("list_shortcuts"),
  create: (name, combo, action) => invoke<Shortcut>("create_shortcut", { name, combo, action }),
  delete: (id) => invoke("delete_shortcut", { id }),
  setEnabled: (id, enabled) => invoke("set_enable_shortcut", { id, enabled }),
  rename: (id, name) => invoke("rename_shortcut", { id, name }),
};

