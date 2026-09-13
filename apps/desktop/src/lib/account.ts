import {invoke} from '@tauri-apps/api/core';

type Account = Readonly<{expectedUserId?: string; expectedBackendUrl?: string}>;
let current: Account = {};
/** A fresh identity on every connection boundary also fences same-user reconnects. */
export function setAccount(user: string | null, backend: string = ''): void {
  current = user ? {expectedUserId: user, expectedBackendUrl: backend} : {};
}
export function captureAccount(): Account { return current; }
export async function invokeAccount<T>(command: string, args: Record<string, unknown> = {}, owner = captureAccount()): Promise<T> {
  if (owner !== current) throw new Error('Connection changed; request cancelled.');
  const result = await invoke<T>(command, {...args, ...owner});
  if (owner !== current) throw new Error('Connection changed; response discarded.');
  return result;
}
