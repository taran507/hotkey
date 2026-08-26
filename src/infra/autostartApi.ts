import {disable, enable, isEnabled} from "@tauri-apps/plugin-autostart";

export type AutostartApi = {
    isEnabled(): Promise<boolean>;
    setEnabled(enabled: boolean): Promise<void>;
};

export const tauriAutostartApi: AutostartApi = {
    isEnabled: () => isEnabled(),
    setEnabled: async (enabled) => {
        if (enabled) {
            await enable();
        } else {
            await disable();
        }
    },
};
