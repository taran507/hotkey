import {HotkeysService} from "./app/hotkeysService";
import {tauriAutostartApi} from "./infra/autostartApi";
import {tauriHotkeysApi} from "./infra/hotkeysApi";
import {mountAutostartSetting} from "./ui/autostart";
import {getDomRefs} from "./ui/dom";
import {mountHotkeysPage} from "./ui/hotkeysPage";

window.addEventListener("DOMContentLoaded", () => {
  const service = new HotkeysService(tauriHotkeysApi);
  const dom = getDomRefs();
  mountHotkeysPage(service, dom);
    mountAutostartSetting(tauriAutostartApi, dom).catch(console.error);
});
