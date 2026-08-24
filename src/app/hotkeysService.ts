import type { Action, Combo, Shortcut } from "../domain/hotkey";
import type { HotkeysApi } from "../infra/hotkeysApi";

export class HotkeysService {
  constructor(private readonly api: HotkeysApi) {}

  list(): Promise<Shortcut[]> {
    return this.api.list();
  }

  create(combo: Combo, action: Action): Promise<Shortcut> {
    return this.api.create(combo, action);
  }

  delete(id: string): Promise<void> {
    return this.api.delete(id);
  }

  setEnabled(id: string, enabled: boolean): Promise<void> {
    return this.api.setEnabled(id, enabled);
  }
}

