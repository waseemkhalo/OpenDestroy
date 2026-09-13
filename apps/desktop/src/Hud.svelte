<script lang="ts">
  import {onMount} from 'svelte';
  import {invoke} from '@tauri-apps/api/core';
  import {listen} from '@tauri-apps/api/event';
  import DictationHud from './lib/notch/DictationHud.svelte';
  import {initialHud, sanitizeHudSnapshot, type HudSnapshot, type HudAction} from './lib/notch/hud';
  let state = $state<HudSnapshot>(initialHud());
  function accept(next: HudSnapshot) { if (next.revision >= state.revision) state = sanitizeHudSnapshot(next); }
  function action(value: HudAction) { void invoke('hud_action', {action: value}).catch(() => {}); }
  onMount(() => {
    let disposed = false, off: (() => void) | undefined;
    void (async () => {
      const stop = await listen<HudSnapshot>('destroy://hud', event => accept(event.payload));
      if (disposed) { stop(); return; }
      off = stop;
      const snapshot = await invoke<HudSnapshot | null>('hud_snapshot');
      if (!disposed && snapshot) accept(snapshot);
    })();
    return () => { disposed = true; off?.(); };
  });
</script>
<DictationHud {state} onAction={action}/>
