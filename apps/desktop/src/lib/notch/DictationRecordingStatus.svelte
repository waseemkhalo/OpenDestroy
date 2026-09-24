<script lang="ts">
  import { AppWindow, Globe2, Mail, ShieldAlert } from "@lucide/svelte";

  type DictationTarget = {
    canPaste: boolean;
    appName: string;
    bundleId: string;
    appKind: string;
    appIconDataUrl?: string | null;
  };

  let {
    target = null,
    needsAccessibility = false,
    processing = false,
  }: {
    target?: DictationTarget | null;
    needsAccessibility?: boolean;
    processing?: boolean;
  } = $props();
</script>

<!-- The center cell is intentionally empty: it maps to the physical camera. -->
<div
  class="dictation-flank dictation-flank--input"
  role="status"
  aria-label={processing ? "Processing dictation" : "Listening"}
>
  <div class="dictation-indicator" data-processing={processing}>
    <div class="waveform" aria-hidden="true">
      {#each [0, 1, 2, 3, 4] as bar}
        <span class="wave-bar" style={`--i:${bar}`}></span>
      {/each}
    </div>
    <span class="dictation-spinner" aria-hidden="true"></span>
  </div>
</div>
<div class="dictation-camera-gutter" aria-hidden="true"></div>
<div
  class="dictation-flank dictation-flank--destination"
  aria-label={target?.appName ? `Dictating into ${target.appName}` : "Detecting focused app"}
  title={target?.appName || "Detecting focused app"}
>
  <span class="dictation-app-icon" aria-hidden="true">
    {#if target?.appIconDataUrl}
      <img src={target.appIconDataUrl} alt="" data-native-app-icon="true" />
    {:else if target?.appKind === "gmail"}
      <Mail size={14} strokeWidth={1.8} />
    {:else if target?.appKind === "slack"}
      <span class="dictation-brand dictation-brand--slack">#</span>
    {:else if target?.appKind === "linkedin"}
      <span class="dictation-brand dictation-brand--linkedin">in</span>
    {:else if target?.appKind === "notion"}
      <span class="dictation-brand dictation-brand--notion">N</span>
    {:else if target?.appKind === "mail" || target?.appKind === "outlook"}
      <Mail size={14} strokeWidth={1.8} />
    {:else if target?.appKind === "browser"}
      <Globe2 size={14} strokeWidth={1.8} />
    {:else}
      <AppWindow size={14} strokeWidth={1.8} />
    {/if}
  </span>
  <span class="dictation-app-name">{target?.appName || "Detecting…"}</span>
  {#if needsAccessibility}
    <span class="dictation-permission-warning" aria-label="Accessibility required for direct paste">
      <ShieldAlert size={13} strokeWidth={1.8} aria-hidden="true" />
    </span>
  {/if}
</div>

<style>
  .dictation-flank {
    display: flex;
    align-items: center;
    min-width: 0;
    height: 100%;
  }
  .dictation-flank--input {
    justify-content: center;
  }
  .dictation-camera-gutter {
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
  .dictation-flank--destination {
    justify-content: flex-start;
    gap: 6px;
    padding: 0 9px;
    color: var(--destroy-text-2);
    overflow: hidden;
  }
  .dictation-indicator {
    position: relative;
    width: 29px;
    height: 20px;
    flex-shrink: 0;
  }
  .waveform {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    gap: 3px;
    transition: opacity 0.16s ease, transform 0.2s var(--destroy-ease);
  }
  .wave-bar {
    width: 3px;
    height: 100%;
    border-radius: 999px;
    background: #e25345;
    transform-origin: center;
    animation: wave 0.9s ease-in-out infinite;
    animation-delay: calc(var(--i) * -0.18s);
  }
  .dictation-spinner {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 14px;
    height: 14px;
    margin: -7px 0 0 -7px;
    box-sizing: border-box;
    border: 2px solid color-mix(in srgb, #e25345 24%, transparent);
    border-top-color: #e25345;
    border-right-color: #e25345;
    border-radius: 50%;
    opacity: 0;
    transform: scale(0.72) rotate(0deg);
    animation: dictation-spin 0.72s linear infinite;
    animation-play-state: paused;
    transition: opacity 0.16s ease, transform 0.2s var(--destroy-ease);
  }
  .dictation-indicator[data-processing="true"] .waveform {
    opacity: 0;
    transform: scale(0.72);
  }
  .dictation-indicator[data-processing="true"] .dictation-spinner {
    opacity: 1;
    transform: scale(1) rotate(0deg);
    animation-play-state: running;
  }
  .dictation-app-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    flex: 0 0 22px;
    border-radius: 6px;
    color: var(--destroy-text-2);
    overflow: hidden;
  }
  .dictation-app-icon img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
  .dictation-app-name {
    min-width: 0;
    overflow: hidden;
    color: var(--destroy-text);
    font-size: 11px;
    font-weight: 580;
    line-height: 1;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dictation-permission-warning {
    display: inline-flex;
    flex: 0 0 auto;
    color: var(--destroy-amber);
  }
  .dictation-brand {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 15px;
    height: 15px;
    color: currentColor;
    font-weight: 700;
    line-height: 1;
  }
  .dictation-brand--slack { font-size: 16px; font-weight: 500; }
  .dictation-brand--linkedin {
    border: 1px solid currentColor;
    border-radius: 2px;
    font-size: 8px;
    letter-spacing: -.08em;
  }
  .dictation-brand--notion {
    border: 1px solid currentColor;
    border-radius: 2px;
    font-family: Georgia, serif;
    font-size: 10px;
  }
  @keyframes wave {
    0%, 100% { transform: scaleY(0.35); opacity: 0.7; }
    50% { transform: scaleY(1); opacity: 1; }
  }
  @keyframes dictation-spin {
    to { transform: scale(1) rotate(360deg); }
  }
  @media (prefers-reduced-motion: reduce) {
    .wave-bar { animation: none; transform: scaleY(0.7); }
    .dictation-spinner { animation: none; }
  }
</style>
