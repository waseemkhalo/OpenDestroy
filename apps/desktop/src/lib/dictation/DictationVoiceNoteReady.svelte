<script lang="ts">
  import { Mic2, X } from "@lucide/svelte";
  let { onCancel = () => {} }: { onCancel?: () => void } = $props();
</script>

<section class="voice-note-ready" aria-label="Voice note ready">
  <div class="voice-note-mark"><Mic2 size={18} strokeWidth={1.7} /></div>
  <div><strong>Voice note</strong><span>Hold ⌘ ` and record. Destroy will attach the audio with a transcript.</span></div>
  <button type="button" aria-label="Cancel voice note" onclick={onCancel}><X size={13} /></button>
</section>

<style>
  .voice-note-ready { display: grid; grid-template-columns: 34px 1fr 22px; gap: 9px; align-items: center; width: 332px; min-height: 70px; padding: 10px 11px; color: #fff; }
  .voice-note-mark { display: grid; place-items: center; width: 32px; height: 32px; border: 1px solid rgba(65,151,255,.5); border-radius: 11px; background: rgba(38,119,255,.12); color: #58a2ff; }
  div:nth-child(2) { display: flex; flex-direction: column; min-width: 0; }
  strong { font-size: 10px; font-weight: 650; }
  span { margin-top: 3px; color: rgba(255,255,255,.42); font-size: 8px; line-height: 1.4; }
  button { display: inline-flex; align-items: center; justify-content: center; width: 22px; height: 22px; padding: 0; border: 0; border-radius: 7px; background: transparent; color: rgba(255,255,255,.38); }
  button:hover, button:focus-visible { background: rgba(255,255,255,.08); color: #fff; outline: none; }
</style>
