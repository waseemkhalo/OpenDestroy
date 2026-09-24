import type { DictationEmojiChoice } from '../dictation/emoji';
import type { DictationMediaKind, DictationMediaResult } from '../dictation/giphy';

export type HudAction = { revision: number; action: 'cancel' | 'open' | 'copy' | 'accessibility' | 'choose' | 'favorite' | 'rotate' | 'kind'; index?: number; kind?: DictationMediaKind };
export type HudSnapshot = {
  revision: number; visible: boolean; phase: 'idle' | 'recording' | 'processing' | 'picker';
  camera: {width: number; height: number};
  target: {canPaste: boolean; appName: string; bundleId: string; appKind: string; appIconDataUrl?: string} | null;
  message: string; error: string; needsAccessibility: boolean; recovery: boolean; voiceReady: boolean;
  clipboardCopied?: boolean;
  emoji: DictationEmojiChoice[]; media: DictationMediaResult[]; links: {name: string; url: string}[];
  leading: string; query: string; kind: DictationMediaKind; page: number; busy: boolean; mediaOpen: boolean;
  mediaFavoritesEnabled?: boolean;
};
export function initialHud(): HudSnapshot {
  return {revision: 0, visible: false, phase: 'idle', camera: {width: 200, height: 34}, target: null, message: '', error: '', needsAccessibility: false, recovery: false, voiceReady: false, emoji: [], media: [], links: [], leading: '', query: '', kind: 'gif', page: 0, busy: false, mediaOpen: false, mediaFavoritesEnabled: true};
}
export function sanitizeHudSnapshot(snapshot: HudSnapshot): HudSnapshot {
  return snapshot.visible ? snapshot : {...initialHud(), revision: snapshot.revision};
}

/** Delivery is authoritative: never claim a copy after a rejected native call. */
export function deliveryHudFeedback(delivery: {delivery: string; reason?: string | null}, appName = '') {
  if (delivery.delivery === 'inline') return {visible: false, message: '', needsAccessibility: false, clipboardCopied: false};
  const destination = appName.trim() ? ` into ${appName.trim().slice(0, 80)}` : '';
  return {
    visible: true,
    message: `Couldn’t paste${destination}.`,
    needsAccessibility: delivery.reason === 'needs_accessibility',
    clipboardCopied: delivery.delivery === 'clipboard',
  };
}

export function hudNotice(state: HudSnapshot): {title: string; detail: string; icon: 'microphone' | 'clipboard' | 'warning'} {
  if (state.clipboardCopied) return {title: 'Copied. Ready to paste.', detail: state.message, icon: 'clipboard'};
  const error = state.error.toLowerCase();
  if (/no speech|no audio|blank_audio|empty recording/.test(error)) {
    return {title: 'We couldn’t hear you.', detail: 'Check your microphone, then hold your shortcut and try again.', icon: 'microphone'};
  }
  if (/microphone|audio input|recording channels|audioqueue/.test(error)) {
    return {title: 'Your microphone needs attention.', detail: /permission|access|denied/.test(error) ? 'Allow microphone access in macOS Settings, then try again.' : 'Check your input device in Settings, then try again.', icon: 'microphone'};
  }
  if (state.recovery) return {title: 'Your words aren’t lost.', detail: 'We couldn’t finish inserting them. Copy your text below, then paste it where you need it.', icon: 'clipboard'};
  if (/network|connection|timeout|timed out|fetch/.test(error)) return {title: 'Couldn’t reach your speech service.', detail: 'Check your connection and try again. Nothing was pasted.', icon: 'warning'};
  if (state.error) return {title: 'Dictation didn’t finish.', detail: 'Try your shortcut again. If this keeps happening, check your speech setup in Settings.', icon: 'warning'};
  return {title: state.message, detail: '', icon: 'warning'};
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
