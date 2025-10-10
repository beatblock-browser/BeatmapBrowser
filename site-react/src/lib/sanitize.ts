export function sanitizeText(input: any, maxLen: number): string {
  const s = typeof input === 'string' ? input : String(input ?? '');
  const out: string[] = [];
  let count = 0;
  for (const ch of s) {
    if (count >= maxLen) break;
    const code = ch.codePointAt(0) ?? 0;
    // Skip control chars
    if (code < 0x20 || code === 0x7f) continue;
    // Drop angle brackets to avoid accidental HTML parsing by future code
    if (ch === '<' || ch === '>') continue;
    out.push(ch);
    count++;
  }
  const joined = out.join('').trim();
  return joined.replace(/\s+/g, ' ');
}
