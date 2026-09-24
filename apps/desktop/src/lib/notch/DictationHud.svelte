<script lang="ts">
  import RecordingStatus from './DictationRecordingStatus.svelte';
  import EmojiPicker from '../dictation/DictationEmojiPicker.svelte';
  import MediaPicker from '../dictation/DictationMediaPicker.svelte';
  import type { HudSnapshot, HudAction } from './hud';
  import {hudNotice} from './hud';
  import {MicOff, ClipboardCheck, CircleAlert, X, ArrowUpRight, Copy} from '@lucide/svelte';
  let {state, onAction = () => {}}: {state: HudSnapshot; onAction?: (action: HudAction) => void} = $props();
  const recording = $derived(state.phase === 'recording' || state.phase === 'processing');
  const notice = $derived(hudNotice(state));
  const failure = $derived(Boolean(state.error || state.clipboardCopied || state.recovery || state.needsAccessibility));
  const mac = typeof navigator === 'undefined' || /Mac|iPhone|iPad/.test(navigator.platform);
  function act(action: HudAction['action'], details: Partial<HudAction> = {}) { onAction({revision: state.revision, action, ...details}); }
</script>

{#if state.visible}
<section class="hud" class:recording-only={recording} aria-label="Dictation" style={`--camera-width:${state.camera.width}px;--camera-height:${state.camera.height}px`}>
  {#if recording}
    <button class="notch" aria-label="Cancel dictation" onclick={() => act('cancel')}>
      <RecordingStatus target={state.target} needsAccessibility={state.needsAccessibility} processing={state.phase === 'processing'}/>
    </button>
  {:else}
    <div class="feedback" role={failure?'alert':'status'}>
      {#if failure}<span class="notice-icon" aria-hidden="true">{#if notice.icon==='microphone'}<MicOff size={21}/>{:else if notice.icon==='clipboard'}<ClipboardCheck size={21}/>{:else}<CircleAlert size={21}/>{/if}</span>{/if}
      <div class="notice-copy"><h2>{notice.title}</h2>{#if notice.detail}<p>{notice.detail}</p>{/if}
        {#if state.clipboardCopied}<p class="paste-hint">Click your text field, then press <kbd>{mac?'⌘':'Ctrl'}</kbd> <kbd>V</kbd></p>{/if}
      </div>
      <button class="dismiss" aria-label="Dismiss dictation" onclick={() => act('cancel')}><X size={16}/></button>
    </div>
    {#if failure}<div class="notice-actions">
      {#if state.recovery}<button class="copy-action" onclick={() => act('copy')}><Copy size={14}/>Copy text</button>{/if}
      {#if state.needsAccessibility}<button onclick={() => act('accessibility')}>Allow Accessibility<ArrowUpRight size={13}/></button>{/if}
      <button class="open" onclick={() => act('open')}>Open settings<ArrowUpRight size={13}/></button>
    </div>{/if}
    {#if state.emoji.length}<EmojiPicker choices={state.emoji} selectedIndex={0} query={state.query} leadingText={state.leading} onSelect={index => act('choose', {index})} onCancel={() => act('cancel')}/>{/if}
    {#if state.mediaOpen}<MediaPicker kind={state.kind} query={state.query} leadingText={state.leading} page={state.page} results={state.media} selectedIndex={0} loading={state.busy} delivering={false} error={state.error} attribution="Powered by GIPHY" allowFavorites={state.mediaFavoritesEnabled !== false} onKindChange={kind => act('kind', {kind})} onSelect={index => act('choose', {index})} onFavorite={index => act('favorite', {index})} onRotate={() => act('rotate')} onCancel={() => act('cancel')}/>{/if}
    {#if state.links.length}<div class="link-picker"><p>{state.leading}</p>{#each state.links as link,index}<button disabled={state.busy} onclick={() => act('choose', {index})}><kbd>{index+1}</kbd><strong>{link.name}</strong><small>{link.url}</small></button>{/each}<p>Inserts a link. Check access before sending.</p></div>{/if}
  {/if}
</section>
{/if}

<style>
  :global(html.dictation-hud-window),:global(body.dictation-hud-window),:global(body.dictation-hud-window #app){background:transparent!important}
  .hud{background:#000;color:#f5f4f0;border:1px solid #000;border-top:0;border-radius:0 0 20px 20px;padding:calc(var(--camera-height,34px) + 12px) 20px 16px;max-height:100vh;overflow:auto;font:13px/1.45 -apple-system,BlinkMacSystemFont,sans-serif}
  .hud.recording-only{background:transparent;padding:0;border:0;overflow:hidden}
  .notch{display:grid;grid-template-columns:128px var(--camera-width,200px) 128px;width:calc(var(--camera-width,200px) + 256px);min-width:calc(var(--camera-width,200px) + 256px);height:max(46px,var(--camera-height,34px));min-height:max(46px,var(--camera-height,34px));border:0;border-radius:0 0 18px 18px;padding:0;background:#000;cursor:pointer}
  .notch:hover{background:#000}
  .feedback{display:flex;align-items:flex-start;gap:12px}
  .notice-icon{display:grid;place-items:center;width:34px;height:34px;flex:0 0 34px;border-radius:11px;background:#e253451a;color:#e25345}
  .notice-copy{flex:1;min-width:0}
  .notice-copy h2{font:600 14px/1.4 -apple-system,BlinkMacSystemFont,sans-serif;margin:0 0 5px;color:#f1eee7}
  .notice-copy p{margin:0;color:#aaa9a2;font-size:12px;line-height:1.5;overflow-wrap:anywhere}
  .notice-copy .paste-hint{color:#f1eee7;margin-top:10px}
  .paste-hint kbd{display:inline-block;padding:1px 5px;min-width:20px;border:1px solid #ffffff25;border-radius:5px;background:#191919;color:#f1eee7;font:11px/1.6 -apple-system,BlinkMacSystemFont,sans-serif;text-align:center}
  .dismiss{display:grid;place-items:center;flex:none;width:26px;height:26px;margin:-4px -8px 0 0;padding:0;border:0;background:transparent;color:#aaa9a2;border-radius:7px}
  .dismiss:hover{background:#ffffff12;color:#fff}
  .notice-actions{display:flex;align-items:center;flex-wrap:wrap;gap:8px;margin:14px 0 0 46px}
  .notice-actions button{display:inline-flex;align-items:center;gap:6px;margin:0;padding:6px 9px;border:0;border-radius:7px;background:#ffffff0c;color:#e6e2da;font-size:11px;cursor:pointer}
  .notice-actions .copy-action{background:#e25345;color:#fff}
  .notice-actions .open{background:transparent;color:#aaa9a2}
  .notice-actions button:hover{filter:brightness(1.2)}
  .hud :global(button:focus-visible){outline:2px solid #e25345;outline-offset:2px}
</style>
