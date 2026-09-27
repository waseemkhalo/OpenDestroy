export type UpdatePayload = {
  version: string;
  downloadAndInstall: (progress?: (event: { event: string; data?: { contentLength?: number; chunkLength?: number } }) => void, options?: { timeout: number }) => Promise<void>;
  close: () => Promise<void>;
};
export type UpdateState = { phase: 'idle' | 'checking' | 'available' | 'installing' | 'installed' | 'error'; message: string; version?: string; percent?: number };
type Dependencies = {
  check: () => Promise<UpdatePayload | null>;
  restart: () => Promise<void>;
  busy: () => boolean;
  lock: (value: boolean) => void;
  changed: (state: UpdateState) => void;
};
/** One owner for an explicit update, even when the settings drawer is closed. */
export function createUpdater(deps: Dependencies) {
  let state: UpdateState = { phase: 'idle', message: 'Check for a signed OpenDestroy update.' };
  let payload: UpdatePayload | null = null;
  const set = (next: UpdateState) => { state = next; deps.changed(next); };
  const release = async () => { const previous = payload; payload = null; await previous?.close().catch(() => {}); };
  async function restart() {
    if (state.phase !== 'installed') return;
    try { await deps.restart(); }
    catch { set({ phase: 'installed', message: 'Update installed. Quit and reopen OpenDestroy, or try Restart again.' }); }
  }
  return {
    async check() {
      if (['checking', 'installing', 'installed'].includes(state.phase)) return;
      set({ phase: 'checking', message: 'Checking for updates…' });
      await release();
      try {
        payload = await deps.check();
        set(payload ? { phase: 'available', version: payload.version, message: `OpenDestroy ${payload.version} is available.` } : { phase: 'idle', message: 'You’re up to date.' });
      } catch { set({ phase: 'error', message: 'Could not check for updates. Try again when you’re online.' }); }
    },
    async install() {
      if (state.phase !== 'available' || !payload || deps.busy()) return;
      deps.lock(true);
      const version = payload.version;
      set({ phase: 'installing', version, message: 'Downloading and verifying the update…' });
      let total = 0, received = 0;
      try {
        await payload.downloadAndInstall(event => {
          if (event.event === 'Started') total = event.data?.contentLength || 0;
          if (event.event === 'Progress') received += event.data?.chunkLength || 0;
          set({ phase: 'installing', version, message: event.event === 'Finished' ? 'Verifying and installing…' : 'Downloading and verifying the update…', percent: total ? Math.min(100, Math.floor(received / total * 100)) : undefined });
        }, { timeout: 300000 });
        set({ phase: 'installed', version, message: 'Update installed. Restarting OpenDestroy…' });
        await restart();
      } catch {
        deps.lock(false);
        set({ phase: 'error', message: 'The update could not be installed or verified. Your settings have not been reset. Check again to retry.' });
      } finally { await release(); }
    },
    restart,
  };
}
