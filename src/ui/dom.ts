export type DomRefs = {
  comboInput: HTMLInputElement;
  comboClearBtn: HTMLButtonElement;
  programInput: HTMLInputElement;
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
    comboInput: req<HTMLInputElement>("#combo-input"),
    comboClearBtn: req<HTMLButtonElement>("#combo-clear"),
    programInput: req<HTMLInputElement>("#program-input"),
    argsInput: req<HTMLInputElement>("#args-input"),
    addBtn: req<HTMLButtonElement>("#add-btn"),
    addErrorEl: req<HTMLElement>("#add-error"),
    listEl: req<HTMLElement>("#list"),
  };
}

