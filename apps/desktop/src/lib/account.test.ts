import {beforeEach, describe, expect, it, vi} from 'vitest';
import {invoke} from '@tauri-apps/api/core';
import {captureAccount, invokeAccount, setAccount} from './account';
import {currentDictationPreferences, loadDictationPreferences, resetDictationComposerCache} from './dictation/dictationComposer';
vi.mock('@tauri-apps/api/core', () => ({invoke: vi.fn()}));
beforeEach(() => {vi.mocked(invoke).mockReset();setAccount(null);resetDictationComposerCache();});
describe('community connection ownership', () => {
  it('sends the captured user and canonical backend to native commands', async () => {
    setAccount('owner', 'http://127.0.0.1:8787');
    vi.mocked(invoke).mockResolvedValue({ok: true});
    await invokeAccount('backend_api', {method: 'GET', path: '/v1/account'});
    expect(invoke).toHaveBeenCalledWith('backend_api', {method:'GET',path:'/v1/account',expectedUserId:'owner',expectedBackendUrl:'http://127.0.0.1:8787'});
  });
  it('does not send a queued fallback to a replacement account', async () => {
    setAccount('first', 'http://127.0.0.1:8787');const old=captureAccount();setAccount('second', 'http://127.0.0.1:8787');
    await expect(invokeAccount('destroy_transcribe_dictation', {audioBase64:'audio'}, old)).rejects.toThrow('request cancelled');
    expect(invoke).not.toHaveBeenCalled();
  });
  it('discards a response even when the same user reconnects to the same server', async () => {
    setAccount('owner', 'http://127.0.0.1:8787');
    let resolve!:(value: unknown)=>void;
    vi.mocked(invoke).mockImplementation(()=>new Promise<unknown>(r=>{resolve=r;}) as Promise<never>);
    const pending=invokeAccount('backend_api');
    const rejected=expect(pending).rejects.toThrow('response discarded');
    setAccount(null);setAccount('owner', 'http://127.0.0.1:8787');resolve({private:'old'});await rejected;
  });
  it('cannot repopulate preferences after an account cache was cleared', async () => {
    setAccount('owner', 'http://127.0.0.1:8787');
    let resolve!:(value: unknown)=>void;
    vi.mocked(invoke).mockImplementation(()=>new Promise<unknown>(r=>{resolve=r;}) as Promise<never>);
    const pending=loadDictationPreferences('server|owner');
    const rejected=expect(pending).rejects.toThrow('response discarded');
    setAccount(null);resetDictationComposerCache();resolve({style_note:'old private style'});await rejected;
    expect(currentDictationPreferences('server|owner').style_note).toBe('');
  });
});
