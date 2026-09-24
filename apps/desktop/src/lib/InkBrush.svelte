<script lang="ts">
  import { onMount } from "svelte";

  const SAMPLE_COUNT = 34;
  const clamp = (value: number) => Math.max(0, Math.min(1, Number.isFinite(value) ? value : 0));
  const perceptualLevel = (value: number) => Math.sqrt(clamp(value * 4));

  let { microphoneLevel = 0, recording = false }: { microphoneLevel?: number; recording?: boolean } = $props();
  let history = $state<number[]>(Array(SAMPLE_COUNT).fill(0));
  let paintLevel = $state(0);
  let reducedMotion = $state(false);

  $effect(() => {
    if (!reducedMotion) return;
    const level = recording ? perceptualLevel(microphoneLevel) : 0;
    paintLevel = level;
    history = Array(SAMPLE_COUNT).fill(level);
  });

  $effect(() => {
    if (reducedMotion) return;
    if (!recording) {
      paintLevel = 0;
      history = Array(SAMPLE_COUNT).fill(0);
      return;
    }
    let frame = 0;
    let lastSample = 0;
    let smoothLevel = 0;
    const sample = (now: number) => {
      if (now - lastSample >= 48) {
        lastSample = now;
        const target = perceptualLevel(microphoneLevel);
        smoothLevel += (target - smoothLevel) * 0.32;
        paintLevel = smoothLevel;
        history = [...history.slice(1), smoothLevel];
      }
      frame = requestAnimationFrame(sample);
    };
    frame = requestAnimationFrame(sample);
    return () => cancelAnimationFrame(frame);
  });

  onMount(() => {
    const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    reducedMotion = mediaQuery.matches;
    const onMotionPreferenceChange = () => {
      reducedMotion = mediaQuery.matches;
    };
    mediaQuery.addEventListener("change", onMotionPreferenceChange);

    return () => {
      mediaQuery.removeEventListener("change", onMotionPreferenceChange);
    };
  });
</script>

<div class:recording class="ink-brush" style={`--paint-level:${paintLevel}`} aria-hidden="true">
  <svg class="art-filter-defs" aria-hidden="true" focusable="false">
    <defs>
      <filter id="ink-charcoal-key" color-interpolation-filters="sRGB">
        <feColorMatrix type="matrix" values=".8 0 0 0 0  0 .8 0 0 0  0 0 .8 0 0  .8 .8 .8 0 -.19" />
      </filter>
    </defs>
  </svg>
  <span class="brush-base" aria-hidden="true"></span>
  <div class="brush-source" aria-hidden="true">
    {#each [0, 1, 2, 3, 4] as segment}
      {@const sampleIndex = Math.min(SAMPLE_COUNT - 1, Math.round((segment + 0.5) * SAMPLE_COUNT / 5))}
      <span style={`--segment:${segment};--segment-level:${history[sampleIndex] ?? 0}`}></span>
    {/each}
  </div>
</div>

<style>
  .ink-brush{position:relative;width:min(100%,560px);height:78px;overflow:hidden;mask-image:radial-gradient(ellipse at center,#000 0%,#000 44%,#000c 63%,transparent 88%);-webkit-mask-image:radial-gradient(ellipse at center,#000 0%,#000 44%,#000c 63%,transparent 88%)}
  .brush-base{position:absolute;inset:0;background-image:url('/art/voice-ink.png');background-size:100% auto;background-position:center 87%;background-repeat:no-repeat;filter:url(#ink-charcoal-key);mix-blend-mode:normal;opacity:.95;mask-image:linear-gradient(180deg,transparent 4%,#000 23%,#000 82%,transparent 100%);-webkit-mask-image:linear-gradient(180deg,transparent 4%,#000 23%,#000 82%,transparent 100%)}
  .brush-source{position:absolute;z-index:1;inset:0;display:flex;opacity:.78;filter:saturate(1.18) contrast(1.1)}
  .brush-source span{mix-blend-mode:normal;filter:url(#ink-charcoal-key)}
  .brush-source span{display:block;flex:1;height:100%;background-image:url('/art/selection-brush.png');background-size:500% 256%;background-position:calc(var(--segment) * 25%) 50%;background-repeat:no-repeat;transform:translateY(calc((var(--paint-level) + var(--segment-level)) * -2px)) scaleY(calc(.86 + var(--paint-level) * .2 + var(--segment-level) * .28));transform-origin:center;transition:transform 100ms ease}
  @media(prefers-reduced-motion:reduce){.brush-source span{transition:none}}
</style>
