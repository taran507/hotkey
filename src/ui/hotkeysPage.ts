import type {HotkeysService} from "../app/hotkeysService";
import {type Action, comboToString} from "../domain/hotkey";
import type {HotkeysDomRefs} from "./dom";
import {go} from "./nav";

function setError(el: HTMLElement, msg: string | null) {
  el.textContent = msg ?? "";
}

function actionToText(action: Action): string {
  const launch = action.Launch;
  return `Программа: ${launch.program}${launch.args.length ? " " + launch.args.join(" ") : ""}`;
}

export function mountHotkeysPage(service: HotkeysService, dom: HotkeysDomRefs) {
  async function refreshList() {
    const shortcuts = await service.list();
    if (!shortcuts.length) {
      dom.listEl.innerHTML = `<div class="muted">Пока нет хоткеев.</div>`;
      return;
    }

    const rows = shortcuts
      .sort((a, b) => a.id.localeCompare(b.id))
      .map((s) => {
        const title = s.name.trim() || comboToString(s.combo);
        return `
          <div class="row-item" data-id="${s.id}">
            <div class="row-main">
              <div class="row-title">${title}</div>
              <div class="row-sub muted">${comboToString(s.combo)}</div>
              <div class="row-sub muted">${actionToText(s.action)}</div>
<!--              <div class="row-sub muted id">${s.id}</div>-->
            </div>
            <div class="row-controls">
              <button class="btn-edit" type="button">Изменить</button>
              <button class="btn-danger btn-delete" type="button">Удалить</button>
            </div>
          </div>
        `;
      })
      .join("");

    dom.listEl.innerHTML = rows;

    dom.listEl.querySelectorAll<HTMLElement>(".row-item").forEach((row) => {
      const id = row.dataset.id!;

      row.querySelector<HTMLButtonElement>(".btn-edit")?.addEventListener("click", () => {
        go({page: "editor", id});
      });

      row.querySelector<HTMLButtonElement>(".btn-delete")?.addEventListener("click", async () => {
        if (!confirm("Удалить хоткей?")) return;
        try {
          await service.delete(id);
          setError(dom.listErrorEl, null);
          await refreshList();
        } catch (err) {
          setError(dom.listErrorEl, String(err));
        }
      });
    });
  }

  return {
    onShown() {
      setError(dom.listErrorEl, null);
      refreshList().catch((err) => setError(dom.listErrorEl, String(err)));
    },
  };
}
