export type HotkeysDomRefs = {
  listEl: HTMLElement;
  listErrorEl: HTMLElement;
};

export type EditorDomRefs = {
  titleEl: HTMLElement;
  nameInput: HTMLInputElement;
  comboInput: HTMLInputElement;
  comboClearBtn: HTMLButtonElement;
  programInput: HTMLInputElement;
  programPickBtn: HTMLButtonElement;
  argsInput: HTMLInputElement;
  submitBtn: HTMLButtonElement;
  errorEl: HTMLElement;
};

export type SettingsDomRefs = {
  autostartToggle: HTMLInputElement;
  autostartErrorEl: HTMLElement;
};

function req<T extends Element>(selector: string): T {
  const el = document.querySelector(selector);
  if (!el) throw new Error(`Missing element: ${selector}`);
  return el as T;
}

export function getHotkeysDomRefs(): HotkeysDomRefs {
  return {
    listEl: req<HTMLElement>("#list"),
    listErrorEl: req<HTMLElement>("#list-error"),
  };
}

export function getEditorDomRefs(): EditorDomRefs {
  return {
    titleEl: req<HTMLElement>("#editor-title"),
    nameInput: req<HTMLInputElement>("#name-input"),
    comboInput: req<HTMLInputElement>("#combo-input"),
    comboClearBtn: req<HTMLButtonElement>("#combo-clear"),
    programInput: req<HTMLInputElement>("#program-input"),
    programPickBtn: req<HTMLButtonElement>("#program-pick-btn"),
    argsInput: req<HTMLInputElement>("#args-input"),
    submitBtn: req<HTMLButtonElement>("#editor-submit"),
    errorEl: req<HTMLElement>("#editor-error"),
  };
}

export function getSettingsDomRefs(): SettingsDomRefs {
  return {
    autostartToggle: req<HTMLInputElement>("#autostart-toggle"),
    autostartErrorEl: req<HTMLElement>("#autostart-error"),
  };
}
