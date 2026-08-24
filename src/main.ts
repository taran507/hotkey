import { HotkeysService } from "./app/hotkeysService";
import { tauriHotkeysApi } from "./infra/hotkeysApi";
import { getDomRefs } from "./ui/dom";
import { mountHotkeysPage } from "./ui/hotkeysPage";

window.addEventListener("DOMContentLoaded", () => {
  const service = new HotkeysService(tauriHotkeysApi);
  const dom = getDomRefs();
  mountHotkeysPage(service, dom);
});
