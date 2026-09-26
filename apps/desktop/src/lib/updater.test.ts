import { describe, it, expect, vi } from 'vitest';
import { createUpdater, type UpdateState } from './updater';
function setup() {
  let busy = false; let state: UpdateState | undefined;
  const payload = { version: '0.1.1', close: vi.fn(async () => {}), downloadAndInstall: vi.fn(async () => {}) };
  const deps = { check: vi.fn(async () => payload), restart: vi.fn(async () => {}), busy: () => busy, lock: vi.fn(), changed: (next: UpdateState) => { state = next; } };
  const updater = createUpdater(deps);
  return { updater, deps, payload, state: () => state, busy: (value: boolean) => { busy = value; } };
}
describe('explicit signed updates', () => {
  it('does not install or restart while dictation or model work is active', async () => {
    const t = setup(); await t.updater.check(); t.busy(true); await t.updater.install();
    expect(t.payload.downloadAndInstall).not.toHaveBeenCalled(); expect(t.deps.lock).not.toHaveBeenCalled(); expect(t.deps.restart).not.toHaveBeenCalled();
  });
  it('locks before installation and does not restart after verification failure', async () => {
    const t = setup(); await t.updater.check();
    t.payload.downloadAndInstall.mockImplementation(async () => { expect(t.deps.lock).toHaveBeenCalledWith(true); throw new Error('invalid signature'); });
    await t.updater.install(); expect(t.deps.restart).not.toHaveBeenCalled(); expect(t.deps.lock).toHaveBeenLastCalledWith(false); expect(t.state()?.phase).toBe('error'); expect(t.payload.close).toHaveBeenCalledOnce();
  });
  it('ignores duplicate installs and offers a retry if relaunch fails', async () => {
    const t = setup(); await t.updater.check();
    let finish!: () => void;
    t.payload.downloadAndInstall.mockImplementation(() => new Promise<void>(resolve => { finish = resolve; }));
    t.deps.restart.mockRejectedValueOnce(new Error('restart failed'));
    const pending = t.updater.install(); await t.updater.install(); expect(t.payload.downloadAndInstall).toHaveBeenCalledOnce();
    finish(); await pending; expect(t.state()?.phase).toBe('installed'); await t.updater.restart(); expect(t.deps.restart).toHaveBeenCalledTimes(2);
  });
  it('keeps a missing or failing feed honest', async () => {
    const t = setup(); t.deps.check.mockRejectedValueOnce(new Error('404')); await t.updater.check();
    expect(t.state()?.phase).toBe('error'); await t.updater.install(); expect(t.payload.downloadAndInstall).not.toHaveBeenCalled();
  });
});
