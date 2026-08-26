import type {AutostartApi} from "../infra/autostartApi";
import type {SettingsDomRefs} from "./dom";

function setError(el: HTMLElement, msg: string | null) {
    el.textContent = msg ?? "";
}

export async function mountSettingsPage(api: AutostartApi, dom: SettingsDomRefs) {
    try {
        dom.autostartToggle.checked = await api.isEnabled();
        setError(dom.autostartErrorEl, null);
    } catch (err) {
        setError(dom.autostartErrorEl, String(err));
    }

    dom.autostartToggle.addEventListener("change", async () => {
        const wanted = dom.autostartToggle.checked;
        dom.autostartToggle.disabled = true;
        setError(dom.autostartErrorEl, null);

        try {
            await api.setEnabled(wanted);
            dom.autostartToggle.checked = await api.isEnabled();
        } catch (err) {
            dom.autostartToggle.checked = !wanted;
            setError(dom.autostartErrorEl, String(err));
        } finally {
            dom.autostartToggle.disabled = false;
        }
    });
}
