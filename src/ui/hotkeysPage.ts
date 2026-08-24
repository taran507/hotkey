import type { HotkeysService } from "../app/hotkeysService";
import { parseArgs } from "../domain/args";
import { comboToString, type Action } from "../domain/hotkey";
import { openExecutableFileDialog } from "../infra/fileDialog";
import type { DomRefs } from "./dom";
import { createRecorder } from "./recorder";

function setError(el: HTMLElement, msg: string | null) {
  el.textContent = msg ?? "";
}

function actionToText(action: Action): string {
  const launch = action.Launch;
  return `${launch.program}${launch.args.length ? " " + launch.args.join(" ") : ""}`;
}

export function mountHotkeysPage(service: HotkeysService, dom: DomRefs) {
  const recorder = createRecorder();

  function startRecording() {
    recorder.start();
    dom.comboInput.value = "Запись…";
    setError(dom.addErrorEl, null);
  }

  function syncComboInput() {
    dom.comboInput.value = recorder.state.currentCombo ? comboToString(recorder.state.currentCombo) : "";
  }

  function stopRecording() {
    recorder.stop();
    syncComboInput();
  }

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
              <div class="row-edit">
                <input class="rename-input" value="${s.name}" />
                <button class="btn-save-name" type="button">Сохранить</button>
              </div>
              <div class="row-sub muted">${comboToString(s.combo)}</div>
              <div class="row-sub muted">${actionToText(s.action)}</div>
              <div class="row-sub muted id">${s.id}</div>
            </div>
            <div class="row-controls">
              <label class="toggle">
                <input class="toggle-enabled" type="checkbox" ${s.enabled ? "checked" : ""} />
                <span>enabled</span>
              </label>
              <button class="btn-danger btn-delete" type="button">Удалить</button>
            </div>
          </div>
        `;
      })
      .join("");

    dom.listEl.innerHTML = rows;

    dom.listEl.querySelectorAll<HTMLElement>(".row-item").forEach((row) => {
      const id = row.dataset.id!;

      row.querySelector<HTMLButtonElement>(".btn-save-name")?.addEventListener("click", async () => {
        const nextName = row.querySelector<HTMLInputElement>(".rename-input")?.value.trim() ?? "";
        if (!nextName) {
          setError(dom.addErrorEl, "Название не должно быть пустым.");
          return;
        }

        try {
          await service.rename(id, nextName);
          setError(dom.addErrorEl, null);
          await refreshList();
        } catch (err) {
          setError(dom.addErrorEl, String(err));
        }
      });

      row.querySelector<HTMLInputElement>(".toggle-enabled")?.addEventListener("change", async (e) => {
        const checked = (e.target as HTMLInputElement).checked;
        try {
          await service.setEnabled(id, checked);
          await refreshList();
        } catch (err) {
          console.error(err);
          await refreshList();
        }
      });

      row.querySelector<HTMLButtonElement>(".btn-delete")?.addEventListener("click", async () => {
        if (!confirm("Удалить хоткей?")) return;
        try {
          await service.delete(id);
          await refreshList();
        } catch (err) {
          console.error(err);
        }
      });
    });
  }

  async function addShortcut() {
    setError(dom.addErrorEl, null);
    const name = dom.nameInput.value.trim();
    const prog = dom.programInput.value.trim();
    const args = parseArgs(dom.argsInput.value);

    if (!name) {
      setError(dom.addErrorEl, "Укажи название комбинации.");
      return;
    }
    if (!recorder.state.currentCombo) {
      setError(dom.addErrorEl, "Сначала запиши комбинацию.");
      return;
    }
    if (!prog) {
      setError(dom.addErrorEl, "Укажи путь к программе.");
      return;
    }

    const action: Action = { Launch: { program: prog, args } };

    try {
      await service.create(name, recorder.state.currentCombo, action);
      dom.nameInput.value = "";
      dom.programInput.value = "";
      dom.argsInput.value = "";
      recorder.clear();
      syncComboInput();
      await refreshList();
    } catch (err) {
      setError(dom.addErrorEl, String(err));
    }
  }

  async function pickExecutableFile() {
    try {
      const selected = await openExecutableFileDialog();
      if (!selected) return;

      dom.programInput.value = selected;
      setError(dom.addErrorEl, null);
    } catch (err) {
      setError(dom.addErrorEl, String(err));
    }
  }

  // bind UI
  dom.comboInput.addEventListener("focus", startRecording);
  dom.comboInput.addEventListener("click", startRecording);
  dom.comboClearBtn.addEventListener("click", () => {
    recorder.clear();
    syncComboInput();
    setError(dom.addErrorEl, null);
  });
  dom.programPickBtn.addEventListener("click", () => {
    pickExecutableFile().catch(console.error);
  });
  dom.addBtn.addEventListener("click", addShortcut);

  window.addEventListener("keydown", (e) => {
    const wasRecording = recorder.state.recording;
    const prevCombo = recorder.state.currentCombo;

    recorder.onKeyDown(e);

    if (wasRecording && !recorder.state.recording) {
      // captured or stopped
      stopRecording();
      return;
    }

    if (recorder.state.currentCombo !== prevCombo) {
      stopRecording();
    }
  });

  // initial
  refreshList().catch(console.error);
}

