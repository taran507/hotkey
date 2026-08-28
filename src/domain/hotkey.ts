export type Mods = { ctrl: boolean; alt: boolean; shift: boolean; logo: boolean };
export type Combo = { key: string; mods: Mods };
export type Action = { Launch: { program: string; args: string[] } };
export type Shortcut = { id: string; name: string; combo: Combo; action: Action; enabled: boolean, create_at: number };

export function comboToString(combo: Combo): string {
  const parts: string[] = [];
  if (combo.mods.ctrl) parts.push("Ctrl");
  if (combo.mods.alt) parts.push("Alt");
  if (combo.mods.shift) parts.push("Shift");
  if (combo.mods.logo) parts.push("Win");
  parts.push(combo.key);
  return parts.join("+");
}

