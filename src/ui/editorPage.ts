import type {HotkeysService} from "../app/hotkeysService";
import {formatArgs, parseArgs} from "../domain/args";
import {type Action, type Combo, comboToString, type Shortcut} from "../domain/hotkey";
import {openExecutableFileDialog} from "../infra/fileDialog";
import type {EditorDomRefs} from "./dom";
import {go} from "./nav";
import {createRecorder} from "./recorder";

export type ShortcutDraft = {
    name: string;
    combo: Combo;
    action: Action;
};

type EditorSession = { kind: "create" } | { kind: "edit"; id: string };

function setError(el: HTMLElement, msg: string | null) {
    el.textContent = msg ?? "";
}

function readLaunch(action: Action): { program: string; args: string[] } {
    return action.Launch;
}

export function mountEditorPage(service: HotkeysService, dom: EditorDomRefs) {
    const recorder = createRecorder();
    let session: EditorSession = {kind: "create"};

    function applyChrome() {
        const editing = session.kind === "edit";
        dom.titleEl.textContent = editing ? "Редактировать хоткей" : "Новый хоткей";
        dom.submitBtn.textContent = editing ? "Сохранить" : "Добавить";
        const desc = document.querySelector("[data-page-desc='editor']");
        if (desc) {
            desc.textContent = editing
                ? "Измени поля и сохрани хоткей."
                : "Заполни поля и добавь новый хоткей.";
        }
    }

    function syncComboInput() {
        dom.comboInput.value = recorder.state.currentCombo
            ? comboToString(recorder.state.currentCombo)
            : "";
    }

    function resetForm() {
        recorder.clear();
        dom.nameInput.value = "";
        dom.programInput.value = "";
        dom.argsInput.value = "";
        syncComboInput();
        setError(dom.errorEl, null);
    }

    function fill(shortcut: Shortcut) {
        const launch = readLaunch(shortcut.action);
        recorder.set(shortcut.combo);
        dom.nameInput.value = shortcut.name;
        dom.programInput.value = launch.program;
        dom.argsInput.value = formatArgs(launch.args);
        syncComboInput();
        setError(dom.errorEl, null);
    }

    function startRecording() {
        recorder.start();
        dom.comboInput.value = "Запись…";
        setError(dom.errorEl, null);
    }

    function stopRecording() {
        recorder.stop();
        syncComboInput();
    }

    function readDraft(): ShortcutDraft | null {
        const name = dom.nameInput.value.trim();
        const program = dom.programInput.value.trim();
        const args = parseArgs(dom.argsInput.value);
        const combo = recorder.state.currentCombo;

        if (!name) {
            setError(dom.errorEl, "Укажи название комбинации.");
            return null;
        }
        if (!combo) {
            setError(dom.errorEl, "Сначала запиши комбинацию.");
            return null;
        }
        if (!program) {
            setError(dom.errorEl, "Укажи путь к программе.");
            return null;
        }

        return {name, combo, action: {Launch: {program, args}}};
    }

    async function saveCreate(draft: ShortcutDraft) {
        await service.create(draft.name, draft.combo, draft.action);
    }

    async function saveEdit(id: string, draft: ShortcutDraft) {
        // Same page will persist combo/action here once update exists on the backend.
        void id;
        void draft;
        await service.update(id, draft.name, draft.combo, draft.action);
    }

    async function submit() {
        setError(dom.errorEl, null);
        const draft = readDraft();
        if (!draft) return;

        try {
            if (session.kind === "create") {
                await saveCreate(draft);
            } else {
                await saveEdit(session.id, draft);
            }
            go({page: "hotkeys"});
            resetForm();
        } catch (err) {
            setError(dom.errorEl, String(err));
        }
    }

    async function pickExecutableFile() {
        try {
            const selected = await openExecutableFileDialog();
            if (!selected) return;
            dom.programInput.value = selected;
            setError(dom.errorEl, null);
        } catch (err) {
            setError(dom.errorEl, String(err));
        }
    }

    dom.comboInput.addEventListener("focus", startRecording);
    dom.comboInput.addEventListener("click", startRecording);
    dom.comboClearBtn.addEventListener("click", () => {
        recorder.clear();
        syncComboInput();
        setError(dom.errorEl, null);
    });
    dom.programPickBtn.addEventListener("click", () => {
        pickExecutableFile().catch(console.error);
    });
    dom.submitBtn.addEventListener("click", () => {
        submit().catch(console.error);
    });

    window.addEventListener("keydown", (e) => {
        const wasRecording = recorder.state.recording;
        const prevCombo = recorder.state.currentCombo;

        recorder.onKeyDown(e);

        if (wasRecording && !recorder.state.recording) {
            stopRecording();
            return;
        }

        if (recorder.state.currentCombo !== prevCombo) {
            stopRecording();
        }
    });

    return {
        async open(id: string | null) {
            if (!id) {
                session = {kind: "create"};
                resetForm();
                applyChrome();
                return;
            }

            session = {kind: "edit", id};
            applyChrome();

            try {
                const found = (await service.list()).find((item) => item.id === id);
                if (!found) {
                    resetForm();
                    setError(dom.errorEl, "Хоткей не найден.");
                    return;
                }
                fill(found);
            } catch (err) {
                resetForm();
                setError(dom.errorEl, String(err));
            }
        },
        onHidden() {
            if (recorder.state.recording) {
                stopRecording();
            }
        },
    };
}
