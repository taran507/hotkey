import type {Combo} from "../domain/hotkey";

export type RecorderState = {
  recording: boolean;
  currentCombo: Combo | null;
};

export function createRecorder() {
  const state: RecorderState = { recording: false, currentCombo: null };

  function isModifierCode(code: string) {
    return (
      code.startsWith("Shift") ||
      code.startsWith("Control") ||
      code.startsWith("Alt") ||
      code.startsWith("Meta")
    );
  }

  function start() {
    state.recording = true;
  }

  function stop() {
    state.recording = false;
  }

  function clear() {
    set(null);
  }

  function set(combo: Combo | null) {
    state.currentCombo = combo;
    state.recording = false;
  }

  function onKeyDown(e: KeyboardEvent) {
    if (!state.recording) return;
    e.preventDefault();
    e.stopPropagation();

    if (isModifierCode(e.code)) return;

    state.currentCombo = {
      key: e.code,
      mods: { ctrl: e.ctrlKey, alt: e.altKey, shift: e.shiftKey, logo: e.metaKey },
    };
    stop();
  }

  return {state, start, stop, clear, set, onKeyDown};
}

