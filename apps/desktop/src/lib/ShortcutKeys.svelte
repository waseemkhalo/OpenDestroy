<script lang="ts">
  import {shortcutFeedback, type HeldShortcutKeys} from './shortcutFeedback';
  let { value = "CommandOrControl+Backquote", compact = false, recording = false, interactive = false }: { value?: string; compact?: boolean; recording?: boolean; interactive?: boolean } = $props();
  const empty = (): HeldShortcutKeys => ({meta:false,ctrl:false,alt:false,shift:false,codes:[]});
  let held = $state<HeldShortcutKeys>(empty());
  const mac = typeof navigator === 'undefined' || /Mac|iPhone|iPad/.test(navigator.platform);
  const parts = $derived(value.split("+").map((part) => part.replace("CommandOrControl", "⌘").replace("Backquote", "`")));
  const pressed = $derived(shortcutFeedback(value,held,mac));
  function key(event: KeyboardEvent, down: boolean) {
    if (!interactive || event.isComposing) return;
    const codes = new Set(held.codes);
    if (down) codes.add(event.code); else codes.delete(event.code);
    // macOS can swallow keyup while Command is held; releasing it clears the chord.
    if (!down && (event.code === 'MetaLeft' || event.code === 'MetaRight')) codes.clear();
    held = {meta:event.metaKey,ctrl:event.ctrlKey,alt:event.altKey,shift:event.shiftKey,codes:[...codes]};
  }
  $effect(()=>{value; held=empty();});
</script>

<svelte:window onkeydown={event=>key(event,true)} onkeyup={event=>key(event,false)} onblur={()=>held=empty()}/>
<span class:compact class:recording class="shortcut-keys" aria-label={`Shortcut ${parts.join(" ")}`}>
  {#each parts as part,index}<kbd class:pressed={recording || pressed[index]}>{part}<i aria-hidden="true" class="ink-ring"></i></kbd>{/each}
</span>

<style>
  .shortcut-keys{display:inline-flex;align-items:center;gap:9px}.shortcut-keys kbd{position:relative;z-index:0;display:inline-flex;align-items:center;justify-content:center;min-width:80px;height:76px;margin-bottom:8px;padding:0 16px;border:1px solid #5a605b;border-radius:12px;background:linear-gradient(145deg,#343936 0%,#252927 38%,#1b1e1d 100%);box-shadow:inset 0 1px 0 #f1eee72b,inset 0 -4px 0 #0e1110,0 8px 0 #252b27,0 16px 22px #000b;color:#f1eee7;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;font-size:29px;font-weight:520;line-height:1;transition:transform 120ms ease,box-shadow 120ms ease}
  .shortcut-keys kbd::before{content:"";position:absolute;inset:4px;border:1px solid #ffffff14;border-radius:8px;pointer-events:none}.shortcut-keys kbd::after{content:"";position:absolute;z-index:-1;left:5px;right:5px;bottom:-8px;height:9px;border:1px solid #080a09;border-top:0;border-radius:0 0 8px 8px;background:linear-gradient(180deg,#2b302c,#151817);box-shadow:0 7px 0 #090b0a}
  .shortcut-keys.recording kbd{transform:translateY(4px);box-shadow:inset 0 1px 0 #f1eee71e,inset 0 -2px 0 #0e1110,0 3px 0 #242a26,0 10px 16px #000b}
  .shortcut-keys.compact{gap:4px}.shortcut-keys.compact kbd{min-width:31px;height:31px;padding:0 8px;border-radius:7px;font-size:14px;box-shadow:inset 0 1px 0 #f1eee71c,inset 0 -2px 0 #0e1110,0 4px 0 #0b0d0c,0 6px 9px #0006}.shortcut-keys.compact kbd::before{inset:2px;border-radius:4px}.shortcut-keys.compact kbd::after{left:3px;right:3px;bottom:-5px;height:6px;border-radius:0 0 5px 5px}.shortcut-keys.compact.recording kbd{transform:translateY(2px);box-shadow:inset 0 1px 0 #f1eee71c,inset 0 -1px 0 #0e1110,0 2px 0 #0b0d0c,0 4px 7px #0007}
  @media(prefers-reduced-motion:reduce){.shortcut-keys kbd{transition:none}}
  .shortcut-keys kbd.pressed{transform:translateY(6px) scale(.97);color:#fff5e9;border-color:#d96b58;background:linear-gradient(145deg,#593a31,#302822 65%,#211e1b);box-shadow:inset 0 2px 6px #0008,0 2px 0 #191b18,0 0 22px #d94c4038;transition:transform 110ms ease-out,box-shadow 150ms ease-out,color 150ms ease-out,background 150ms ease-out}
  .shortcut-keys kbd.pressed::after{bottom:-3px;height:4px;box-shadow:0 3px 0 #090b0a}
  .ink-ring{position:absolute;inset:-7px;border:1px solid transparent;border-radius:17px;pointer-events:none}
  .pressed .ink-ring{animation:ink-press 480ms ease-out both}
  @keyframes ink-press{0%{border-color:#e9785f99;transform:scale(.94);opacity:1}100%{border-color:#e9785f00;transform:scale(1.35);opacity:0}}
  .compact kbd.pressed{transform:translateY(2px);box-shadow:inset 0 1px 4px #0008,0 2px 0 #0b0d0c}
  @media(prefers-reduced-motion:reduce){.shortcut-keys kbd.pressed{transform:none;transition:none}.pressed .ink-ring{animation:none}}
</style>
