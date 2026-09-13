import { describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import { initialHud, sanitizeHudSnapshot, runOwnedHudAction, currentHudAction } from './hud';
import DictationHud from './DictationHud.svelte';

describe('separate dictation presentation', () => {
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
    expect(body).not.toContain('>Notes<');
    expect(body).not.toContain('Open settings');
    expect(body).not.toContain('<input');
  });
  it('presents delivery feedback without an editable field and clears hidden content', () => {
    const state = {...initialHud(), visible: true, message: 'Copied. Press Command-V.'};
    const {body} = render(DictationHud, {props: {state}});
    expect(body).toContain('Copied. Press Command-V.');
    expect(body).toContain('Open settings');
    expect(body).not.toContain('<input');
    expect(render(DictationHud, {props: {state: {...state, visible: false}}}).body).not.toContain('Copied.');
  });
});
