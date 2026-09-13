<script lang="ts">
  import RecordingStatus from './DictationRecordingStatus.svelte';
  import EmojiPicker from '../dictation/DictationEmojiPicker.svelte';
  import MediaPicker from '../dictation/DictationMediaPicker.svelte';
  import type { HudSnapshot, HudAction } from './hud';
  let {state, onAction = () => {}}: {state: HudSnapshot; onAction?: (action: HudAction) => void} = $props();
  const recording = $derived(state.phase === 'recording' || state.phase === 'processing');
  function act(action: HudAction['action'], details: Partial<HudAction> = {}) { onAction({revision: state.revision, action, ...details}); }
</script>

{#if state.visible}
<section class="hud" class:recording-only={recording} aria-label="Dictation" style={`--camera-width:${state.camera.width}px;--camera-height:${state.camera.height}px`}>
  {#if recording}
    <button class="notch" aria-label="Cancel dictation" onclick={() => act('cancel')}>
      <RecordingStatus target={state.target} needsAccessibility={state.needsAccessibility} processing={state.phase === 'processing'}/>
    </button>
  {:else}
    <div class="feedback" role="status"><p>{state.error || state.message}</p><button aria-label="Dismiss dictation" onclick={() => act('cancel')}>×</button></div>
    {#if state.recovery}<button onclick={() => act('copy')}>Copy recognized text</button>{/if}
    {#if state.needsAccessibility}<button onclick={() => act('accessibility')}>Allow Accessibility</button>{/if}
    {#if state.emoji.length}<EmojiPicker choices={state.emoji} selectedIndex={0} query={state.query} leadingText={state.leading} onSelect={index => act('choose', {index})} onCancel={() => act('cancel')}/>{/if}
    {#if state.mediaOpen}<MediaPicker kind={state.kind} query={state.query} leadingText={state.leading} page={state.page} results={state.media} selectedIndex={0} loading={state.busy} delivering={false} error={state.error} attribution="Powered by GIPHY" onKindChange={kind => act('kind', {kind})} onSelect={index => act('choose', {index})} onFavorite={index => act('favorite', {index})} onRotate={() => act('rotate')} onCancel={() => act('cancel')}/>{/if}
    {#if state.links.length}<div class="link-picker"><p>{state.leading}</p>{#each state.links as link,index}<button disabled={state.busy} onclick={() => act('choose', {index})}><kbd>{index+1}</kbd><strong>{link.name}</strong><small>{link.url}</small></button>{/each}<p>Inserts a link. Check access before sending.</p></div>{/if}
    <button class="open" onclick={() => act('open')}>Open settings</button>
  {/if}
</section>
{/if}

<style>
  .hud{background:#141516;color:#f5f4f0;border:1px solid #36383c;border-top:0;border-radius:0 0 18px 18px;padding:calc(var(--camera-height,34px) + 8px) 16px 12px;max-height:100vh;overflow:auto;font:12px/1.45 -apple-system,BlinkMacSystemFont,sans-serif}.hud.recording-only{padding:0;border:0;overflow:hidden}.notch{display:grid;grid-template-columns:1fr var(--camera-width,210px) 1fr;width:100%;height:var(--camera-height,34px);border:0;border-radius:0;padding:0;background:#141516;cursor:pointer}.notch:hover{background:#1b1d20}.feedback{display:flex;align-items:flex-start;gap:12px}.feedback p{margin:3px 0 10px;flex:1}.feedback button{border:0;background:transparent;padding:0 5px;font-size:18px;color:#a6a8aa}.hud>button{background:#222428;border:1px solid #45474c;color:#f5f4f0;margin:3px 6px 3px 0;font-size:11px}.hud>.open{display:block;color:#a6a8aa;background:transparent;border-color:transparent;padding:4px 0}.hud :global(button:focus-visible){outline:2px solid #e25345}.hud :global(.dictation-spinner){border-top-color:#e25345;border-right-color:#e25345}
</style>
