import type { DictationEmojiChoice } from '../dictation/emoji';
import type { DictationMediaKind, DictationMediaResult } from '../dictation/giphy';

export type HudAction = { revision: number; action: 'cancel' | 'open' | 'copy' | 'accessibility' | 'choose' | 'favorite' | 'rotate' | 'kind'; index?: number; kind?: DictationMediaKind };
export type HudSnapshot = {
  revision: number; visible: boolean; phase: 'idle' | 'recording' | 'processing' | 'picker';
  camera: {width: number; height: number};
  target: {canPaste: boolean; appName: string; bundleId: string; appKind: string; appIconDataUrl?: string} | null;
  message: string; error: string; needsAccessibility: boolean; recovery: boolean; voiceReady: boolean;
  emoji: DictationEmojiChoice[]; media: DictationMediaResult[]; links: {name: string; url: string}[];
  leading: string; query: string; kind: DictationMediaKind; page: number; busy: boolean; mediaOpen: boolean;
};
export function initialHud(): HudSnapshot {
  return {revision: 0, visible: false, phase: 'idle', camera: {width: 210, height: 34}, target: null, message: '', error: '', needsAccessibility: false, recovery: false, voiceReady: false, emoji: [], media: [], links: [], leading: '', query: '', kind: 'gif', page: 0, busy: false, mediaOpen: false};
}
export function sanitizeHudSnapshot(snapshot: HudSnapshot): HudSnapshot {
  return snapshot.visible ? snapshot : {...initialHud(), revision: snapshot.revision};
}
export async function runOwnedHudAction<T>(operation: () => Promise<T>, isCurrent: () => boolean, apply: (value: T) => void, failed: (error: unknown) => void): Promise<void> {
  try {
    const value = await operation();
    if (isCurrent()) apply(value);
  } catch (error) {
    if (isCurrent()) failed(error);
  }
}
// A queued click from an old render cannot choose a result in a newer session/page.
export function currentHudAction(event: HudAction, snapshot: HudSnapshot): boolean {
  if (!snapshot.visible || event.revision !== snapshot.revision) return false;
  if (event.action === 'choose' || event.action === 'favorite') return !snapshot.busy && snapshot.phase === 'picker' && Number.isInteger(event.index) && event.index! >= 0 && event.index! < 3;
  if (event.action === 'kind') return snapshot.mediaOpen && !snapshot.busy && (event.kind === 'gif' || event.kind === 'sticker');
  if (event.action === 'rotate') return snapshot.mediaOpen && !snapshot.busy;
  return ['cancel', 'open', 'copy', 'accessibility'].includes(event.action);
}
