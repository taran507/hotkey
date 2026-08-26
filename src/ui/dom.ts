export type DomRefs = {
  autostartToggle: HTMLInputElement;
  autostartErrorEl: HTMLElement;
  nameInput: HTMLInputElement;
  comboInput: HTMLInputElement;
  comboClearBtn: HTMLButtonElement;
  programInput: HTMLInputElement;
  programPickBtn: HTMLButtonElement;
  argsInput: HTMLInputElement;
  addBtn: HTMLButtonElement;
  addErrorEl: HTMLElement;
  listEl: HTMLElement;
};

function req<T extends Element>(selector: string): T {
  const el = document.querySelector(selector);
  if (!el) throw new Error(`Missing element: ${selector}`);
  return el as T;
}

export function getDomRefs(): DomRefs {
  return {
    autostartToggle: req<HTMLInputElement>("#autostart-toggle"),
    autostartErrorEl: req<HTMLElement>("#autostart-error"),
    nameInput: req<HTMLInputElement>("#name-input"),
    comboInput: req<HTMLInputElement>("#combo-input"),
    comboClearBtn: req<HTMLButtonElement>("#combo-clear"),
    programInput: req<HTMLInputElement>("#program-input"),
    programPickBtn: req<HTMLButtonElement>("#program-pick-btn"),
    argsInput: req<HTMLInputElement>("#args-input"),
    addBtn: req<HTMLButtonElement>("#add-btn"),
    addErrorEl: req<HTMLElement>("#add-error"),
    listEl: req<HTMLElement>("#list"),
  };
}

