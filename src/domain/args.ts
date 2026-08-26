export function parseArgs(raw: string): string[] {
  const s = raw.trim();
  if (!s) return [];

  const out: string[] = [];
  let cur = "";
  let quote: '"' | "'" | null = null;

  for (let i = 0; i < s.length; i++) {
    const ch = s[i];

    if (quote) {
      if (ch === quote) {
        quote = null;
        continue;
      }
      if (ch === "\\" && i + 1 < s.length) {
        cur += s[i + 1];
        i++;
        continue;
      }
      cur += ch;
      continue;
    }

    if (ch === '"' || ch === "'") {
      quote = ch;
      continue;
    }

    if (/\s/.test(ch)) {
      if (cur.length) out.push(cur), (cur = "");
      continue;
    }

    cur += ch;
  }

  if (cur.length) out.push(cur);
  return out;
}

export function formatArgs(args: string[]): string {
  return args
      .map((arg) => (/\s|"/.test(arg) ? `"${arg.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"` : arg))
      .join(" ");
}

