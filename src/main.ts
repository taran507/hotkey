import {HotkeysService} from "./app/hotkeysService";
import {tauriAutostartApi} from "./infra/autostartApi";
import {tauriHotkeysApi} from "./infra/hotkeysApi";
import {getEditorDomRefs, getHotkeysDomRefs, getSettingsDomRefs} from "./ui/dom";
import {mountEditorPage} from "./ui/editorPage";
import {mountHotkeysPage} from "./ui/hotkeysPage";
import {mountNav} from "./ui/nav";
import {mountSettingsPage} from "./ui/settingsPage";

window.addEventListener("DOMContentLoaded", () => {
  const service = new HotkeysService(tauriHotkeysApi);
  const hotkeys = mountHotkeysPage(service, getHotkeysDomRefs());
  const editor = mountEditorPage(service, getEditorDomRefs());
  mountSettingsPage(tauriAutostartApi, getSettingsDomRefs()).catch(console.error);

  mountNav((route) => {
    editor.onHidden();
    if (route.page === "hotkeys") hotkeys.onShown();
    if (route.page === "editor") editor.open(route.id).catch(console.error);
  });
});
