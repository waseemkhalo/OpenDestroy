export type HeldShortcutKeys = {
  meta: boolean; ctrl: boolean; alt: boolean; shift: boolean; codes: readonly string[];
};

export function shortcutFeedback(value: string, held: HeldShortcutKeys, mac: boolean): boolean[] {
  return value.split('+').map(part => {
    const key = part.trim().toLowerCase();
    if (key === 'commandorcontrol' || key === 'cmdorctrl') return mac ? held.meta : held.ctrl;
    if (['command', 'cmd', 'meta', 'super'].includes(key)) return held.meta;
    if (['control', 'ctrl'].includes(key)) return held.ctrl;
    if (['alt', 'option'].includes(key)) return held.alt;
    if (key === 'shift') return held.shift;
    const aliases: Record<string, string> = {'`':'Backquote', space:'Space', spacebar:'Space', esc:'Escape', return:'Enter'};
    const code = aliases[key] ?? (/^[a-z]$/.test(key) ? `Key${key.toUpperCase()}` : /^\d$/.test(key) ? `Digit${key}` : part.trim());
    return held.codes.some(heldCode => heldCode.toLowerCase() === code.toLowerCase());
  });
}
