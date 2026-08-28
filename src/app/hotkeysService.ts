import type {Action, Combo, Shortcut} from "../domain/hotkey";
import type {HotkeysApi} from "../infra/hotkeysApi";

export class HotkeysService {
    constructor(private readonly api: HotkeysApi) {
    }

    list(): Promise<Shortcut[]> {
        return this.api.list();
    }

    create(name: string, combo: Combo, action: Action, enabled: boolean): Promise<Shortcut> {
        return this.api.create(name, combo, action, enabled);
    }

    delete(id: string): Promise<void> {
        return this.api.delete(id);
    }

    update(
        id: string,
        name: string,
        combo: Combo,
        action: Action,
        enabled: boolean,
    ): Promise<Shortcut> {
        return this.api.update(id, name, combo, action, enabled);
    }
}

