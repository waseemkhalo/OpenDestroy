<script lang="ts">
  import { X } from "@lucide/svelte";
  import type { DictationEmojiChoice } from "./emoji";

  let {
    query,
    leadingText = "",
    choices,
    selectedIndex = 0,
    delivering = false,
    error = "",
    onSelect = (_index: number) => {},
    onCancel = () => {},
  }: {
    query: string;
    /** Message text that will be pasted ahead of the chosen emoji. */
    leadingText?: string;
    choices: DictationEmojiChoice[];
    selectedIndex?: number;
    delivering?: boolean;
    error?: string;
    onSelect?: (index: number) => void;
    onCancel?: () => void;
  } = $props();
</script>

<section class="emoji-picker" aria-label={`Emoji choices for ${query}`}>
  <header><span>Emoji · “{query}”</span><button type="button" aria-label="Close emoji choices" onclick={onCancel}><X size={13} /></button></header>
  <!-- See DictationMediaPicker: the message/command split is a heuristic, so
       the rep sees the message half before it is pasted. -->
  {#if leadingText}
    <p class="emoji-leading" title={leadingText}><span>Sends with</span> “{leadingText}”</p>
  {/if}
  <div class="emoji-grid">
    {#each choices as choice, index (choice.emoji)}
      <button
        type="button"
        class:selected={index === selectedIndex}
        disabled={delivering}
        aria-label={`${index + 1}. ${choice.label}`}
        onclick={() => onSelect(index)}
      >
        <b>{choice.emoji}</b><span>{choice.label}</span><i>{index + 1}</i>
      </button>
    {/each}
  </div>
  <footer>{error || "Click one, or hold ⌘ ` and say one, two, or three."}</footer>
</section>

<style>
  .emoji-picker { width: 332px; padding: 11px 12px 10px; color: rgba(255,255,255,.9); }
  header { display: flex; align-items: center; justify-content: space-between; min-height: 20px; }
  header span { max-width: 280px; overflow: hidden; color: rgba(255,255,255,.56); font-size: 9px; font-weight: 620; text-overflow: ellipsis; white-space: nowrap; }
  header button { display: inline-flex; align-items: center; justify-content: center; width: 20px; height: 20px; padding: 0; border: 0; border-radius: 6px; background: transparent; color: rgba(255,255,255,.38); }
  header button:hover, header button:focus-visible { background: rgba(255,255,255,.08); color: #fff; outline: none; }
  .emoji-leading { margin: 6px 0 0; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: rgba(255,255,255,.6); font-size: 9px; }
  .emoji-leading span { color: rgba(255,255,255,.38); }
  .emoji-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; margin-top: 7px; }
  .emoji-grid button { position: relative; display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 72px; border: 1px solid rgba(255,255,255,.09); border-radius: 13px; background: rgba(255,255,255,.035); color: #fff; }
  .emoji-grid button:hover, .emoji-grid button:focus-visible, .emoji-grid button.selected { border-color: rgba(76,156,255,.8); background: rgba(50,125,255,.12); outline: none; }
  .emoji-grid b { font-size: 29px; font-weight: 400; line-height: 1; }
  .emoji-grid span { margin-top: 7px; color: rgba(255,255,255,.5); font-size: 8px; }
  .emoji-grid i { position: absolute; top: 5px; right: 6px; color: rgba(255,255,255,.26); font-size: 7px; font-style: normal; }
  footer { min-height: 12px; margin-top: 7px; color: rgba(255,255,255,.32); font-size: 7px; text-align: center; }
</style>
