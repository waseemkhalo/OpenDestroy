import { describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import {readFileSync} from 'node:fs';
import { initialHud, sanitizeHudSnapshot, runOwnedHudAction, currentHudAction, deliveryHudFeedback, hudNotice } from './hud';
import DictationHud from './DictationHud.svelte';

describe('separate dictation presentation', () => {
  it('keeps the feedback surface as black as the recording notch', () => {
    const source=readFileSync(new URL('./DictationHud.svelte',import.meta.url),'utf8');
    expect(source).toContain('.hud{background:#000;');
    expect(source).not.toContain('background:#141516');
  });
  it('drops every prior preview field from hidden snapshots while preserving ordering', () => {
    const state = {...initialHud(), revision: 42, visible: false, phase: 'picker' as const, target: {canPaste: true, appName: 'Private app', bundleId: 'private', appKind: 'generic'}, message: 'Private delivery', error: 'Private error', leading: 'Private text', query: 'Private query', links: [{name: 'Private file', url: 'https://private.example'}]};
    expect(sanitizeHudSnapshot(state)).toEqual({...initialHud(), revision: 42});
    const visible = {...state, visible: true};
    expect(sanitizeHudSnapshot(visible)).toBe(visible);
  });
  it.each(['generation', 'account'])('does not apply an asynchronous action after its %s changes', async boundary => {
    let generation = 1, account = 'first';
    const capturedGeneration = generation, capturedAccount = account;
    let resolve!: (value: boolean) => void;
    const operation = new Promise<boolean>(done => resolve = done);
    const apply = vi.fn(), failed = vi.fn();
    const pending = runOwnedHudAction(() => operation, () => generation === capturedGeneration && account === capturedAccount, apply, failed);
    if (boundary === 'generation') generation++; else account = 'second';
    resolve(true);
    await pending;
    expect(apply).not.toHaveBeenCalled();
    expect(failed).not.toHaveBeenCalled();
  });
  it('drops late errors and applies results only for a current owner', async () => {
    let reject!: (reason: Error) => void, current = true;
    const operation = new Promise<boolean>((_, fail) => reject = fail);
    const apply = vi.fn(), failed = vi.fn();
    const pending = runOwnedHudAction(() => operation, () => current, apply, failed);
    current = false;
    reject(new Error('Previous account error'));
    await pending;
    expect(failed).not.toHaveBeenCalled();
    await runOwnedHudAction(async () => true, () => true, apply, failed);
    expect(apply).toHaveBeenCalledExactlyOnceWith(true);
  });
  it('rejects clicks from a dismissed or superseded snapshot', () => {
    const state = {...initialHud(), revision: 7, visible: true};
    expect(currentHudAction({revision: 6, action: 'cancel'}, state)).toBe(false);
    expect(currentHudAction({revision: 7, action: 'cancel'}, {...state, visible: false})).toBe(false);
    expect(currentHudAction({revision: 7, action: 'cancel'}, state)).toBe(true);
  });
  it('limits picker actions to ready bounded choices', () => {
    const state = {...initialHud(), revision: 2, visible: true, phase: 'picker' as const, mediaOpen: true};
    expect(currentHudAction({revision: 2, action: 'choose', index: 1}, state)).toBe(true);
    for (const index of [-1, 3, 0.5]) expect(currentHudAction({revision: 2, action: 'choose', index}, state)).toBe(false);
    expect(currentHudAction({revision: 2, action: 'choose', index: 0}, {...state, busy: true})).toBe(false);
    expect(currentHudAction({revision: 2, action: 'choose', index: 0}, {...state, phase: 'recording'})).toBe(false);
    expect(currentHudAction({revision: 2, action: 'rotate'}, {...state, mediaOpen: false})).toBe(false);
  });
  it('renders only the cancelable waveform and app icon during recording', () => {
    const state = {...initialHud(), visible: true, phase: 'recording' as const, target: {canPaste: true, appName: 'Notes', bundleId: 'com.apple.Notes', appKind: 'generic', appIconDataUrl: 'data:image/png;base64,dGVzdA=='}};
    const {body} = render(DictationHud, {props: {state}});
    expect(body).toContain('Cancel dictation');
    expect(body.indexOf('dictation-flank--input')).toBeLessThan(body.indexOf('dictation-camera-gutter'));
    expect(body.indexOf('dictation-camera-gutter')).toBeLessThan(body.indexOf('dictation-flank--destination'));
    expect(body).toContain('data-native-app-icon="true"');
    expect(body).toContain('dictation-app-name');
    expect(body).toContain('>Notes</span>');
    expect(body).not.toContain('Open settings');
    expect(body).not.toContain('<input');
  });
  it('presents confirmed clipboard fallback with a paste shortcut, without an editable field', () => {
    const state = {...initialHud(), ...deliveryHudFeedback({delivery:'clipboard',reason:'field_changed'},'Notes')};
    const {body} = render(DictationHud, {props: {state}});
    expect(body).toContain('Copied. Ready to paste.');
    expect(body).toContain('Couldn’t paste into Notes.');
    expect(state.message).toBe('Couldn’t paste into Notes.');
    expect(body).not.toContain('original text field changed');
    expect(body).toContain('<kbd ');
    expect(body).toContain('>V</kbd>');
    expect(body).toContain('Open settings');
    expect(body).not.toContain('<input');
    expect(render(DictationHud, {props: {state: {...state, visible: false}}}).body).not.toContain('Copied.');
  });
  it('dismisses successful delivery immediately with no success message', () => {
    const feedback=deliveryHudFeedback({delivery:'inline'},'Notes');
    expect(feedback).toEqual({visible:false,message:'',needsAccessibility:false,clipboardCopied:false});
    expect(render(DictationHud,{props:{state:{...initialHud(),...feedback}}}).body).not.toContain('<section');
    const source=readFileSync(new URL('../../App.svelte',import.meta.url),'utf8');
    expect(source).toContain('hudVisible=feedback.visible');
    expect(source).not.toContain('Text inserted.');
  });
  it('never claims clipboard success for an unknown delivery result', () => {
    expect(deliveryHudFeedback({delivery:'failed'}).clipboardCopied).toBe(false);
    const state={...initialHud(),visible:true,error:'Native pasteboard error',recovery:true};
    const {body}=render(DictationHud,{props:{state}});
    expect(body).toContain('Copy text');
    expect(body).not.toContain('Ready to paste');
    expect(body).not.toContain('Native pasteboard error');
  });
  it('gives plain-language microphone and provider errors without exposing raw internals', () => {
    expect(hudNotice({...initialHud(),error:'No speech was recognized.'}).title).toBe('We couldn’t hear you.');
    expect(hudNotice({...initialHud(),error:'Microphone permission denied'}).detail).toContain('Allow microphone access');
    expect(hudNotice({...initialHud(),error:'network fetch failed: private URL'}).title).toBe('Couldn’t reach your speech service.');
    expect(hudNotice({...initialHud(),error:'unclassified internal failure'}).detail).not.toContain('internal');
  });
  it('shows the picker’s own guidance when GIF search fails', () => {
    const missing = hudNotice({...initialHud(),mediaOpen:true,error:'Add your GIPHY key in Settings → Connections to search GIFs and stickers.'});
    expect(missing.title).toBe('GIF picker needs attention.');
    expect(missing.detail).toContain('GIPHY key');
    expect(hudNotice({...initialHud(),mediaOpen:true,error:'GIPHY rejected the API key. Check it in Settings → Connections.'}).detail).toContain('rejected');
    expect(hudNotice({...initialHud(),mediaOpen:true,error:'No speech was recognized.'}).title).toBe('We couldn’t hear you.');
    expect(hudNotice({...initialHud(),error:'unclassified internal failure'}).detail).not.toContain('internal');
  });
  it('gives live microphone failures actionable input-device recovery guidance', () => {
    const notice = hudNotice({...initialHud(),error:'No audio is arriving from the microphone. Check macOS Microphone permission and your selected input device, then try again.'});
    expect(notice.title).toBe('We couldn’t hear your microphone.');
    expect(notice.detail).toContain('input device');
  });
  it('keeps processing in the notch until delivery completes', () => {
    const {body}=render(DictationHud,{props:{state:{...initialHud(),visible:true,phase:'processing'}}});
    expect(body).toContain('Processing dictation');
    expect(body).toContain('data-processing="true"');
    expect(body).not.toContain('Open settings');
  });
  it('shows quiet and near-limit warnings in the recording notch', () => {
    for (const warning of ['No sound detected. Check your microphone.','About 15 seconds left. Release your shortcut to finish dictation.']) {
      const {body}=render(DictationHud,{props:{state:{...initialHud(),visible:true,phase:'recording',message:warning}}});
      expect(body).toContain(`aria-label="${warning}"`);
      expect(body).toContain('data-warning="true"');
      expect(body).toContain(`title="${warning}"`);
      expect(body).toContain('recording-warning-copy');
      expect(body).toContain(`>${warning}</p>`);
    }
  });
  it('uses the brand red for both waveform and spinner', () => {
    const source=readFileSync(new URL('./DictationRecordingStatus.svelte',import.meta.url),'utf8');
    const hud=readFileSync(new URL('./DictationHud.svelte',import.meta.url),'utf8');
    expect(source).toContain('background: #e25345');
    expect(source).toContain('border-top-color: #e25345');
    expect(source+hud).not.toContain('#5b9dff');
  });
});
