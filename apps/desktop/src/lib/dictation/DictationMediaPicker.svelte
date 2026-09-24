<script lang="ts">
  import { Heart, Shuffle } from "@lucide/svelte";
  import type { DictationMediaKind, DictationMediaResult } from "./giphy";

  let {
    kind,
    query,
    leadingText = "",
    page = 0,
    results,
    selectedIndex,
    loading,
    delivering,
    error,
    attribution,
    onKindChange,
    onSelect,
    onFavorite,
    allowFavorites = true,
    onRotate,
    onCancel,
  }: {
    kind: DictationMediaKind;
    query: string;
    /** Message text that will be pasted ahead of the chosen media. */
    leadingText?: string;
    results: DictationMediaResult[];
    selectedIndex: number;
    loading: boolean;
    delivering: boolean;
    error: string;
    attribution: string;
    /** Zero-based rotation window; page 0 leads with the rep's favorites. */
    page?: number;
    onKindChange: (kind: DictationMediaKind) => void;
    onSelect: (index: number) => void;
    onFavorite: (index: number) => void;
    allowFavorites?: boolean;
    onRotate: () => void;
    onCancel: () => void;
  } = $props();
</script>

<section class="media-picker" aria-label={`${kind === "gif" ? "GIF" : "Sticker"} results for ${query}`}>
  <header>
    <strong><span>{kind === "gif" ? "GIF" : "Sticker"}</span> · “{query}”</strong>
    <div class="media-tabs" role="tablist" aria-label="Media type">
      <button type="button" role="tab" aria-selected={kind === "gif"} onclick={() => onKindChange("gif")}>GIF</button>
      <button type="button" role="tab" aria-selected={kind === "sticker"} onclick={() => onKindChange("sticker")}>Sticker</button>
    </div>
    <button
      class="media-rotate"
      type="button"
      aria-label="Show different results"
      title="Show different results (R)"
      disabled={loading || delivering}
      onclick={onRotate}
    >
      <Shuffle size={11} strokeWidth={1.9} aria-hidden="true" />
      <span>More</span>
    </button>
    <button class="media-close" type="button" aria-label="Cancel media picker" onclick={onCancel}>×</button>
  </header>

  <!-- The split between message and command is a heuristic over an unpunctuated
       transcript. Showing the message half here is what makes a wrong split
       obvious before it reaches the customer's field, rather than after. -->
  {#if leadingText}
    <p class="media-leading" title={leadingText}>
      <span>Sends with</span> “{leadingText}”
    </p>
  {/if}

  {#if delivering}
    <div class="media-loading" role="status">Inserting your choice…</div>
  {:else if loading}
    <div class="media-loading" role="status">
      {page > 0 ? "Finding three different choices…" : "Finding three choices…"}
    </div>
  {:else if error}
    <div class="media-error" role="alert">{error}</div>
  {:else if results.length === 0}
    <div class="media-empty" role="status">No safe results found.</div>
  {:else}
    <div class="media-results" role="listbox" aria-label="GIPHY results">
      {#each results as result, index (result.id)}
        <div class="media-result-wrap">
          <button
            type="button"
            role="option"
            aria-selected={index === selectedIndex}
            aria-label={`${index + 1}. ${result.alt_text || result.title}`}
            class:media-result--selected={index === selectedIndex}
            onclick={() => onSelect(index)}
          >
            <img src={result.preview_url} alt="" draggable="false" />
            <span class="media-number">{index + 1}</span>
          </button>
          {#if allowFavorites}
            <button class:favorited={Boolean(result.favorite_id)} class="favorite-button" type="button" aria-label={result.favorite_id ? "Remove from favorites" : "Add to favorites"} onclick={() => onFavorite(index)}>
              <Heart size={11} strokeWidth={1.8} />
            </button>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
  <footer>
    <span>Say 1–3 · ← → · R more · Enter</span>
    <small>{attribution || "Powered by GIPHY"}</small>
  </footer>
</section>

<style>
  .media-picker {
    width: 100%;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: calc(var(--notch-safe) + 5px);
    color: var(--destroy-text);
    white-space: normal;
  }
  header { display: flex; align-items: center; gap: 7px; min-width: 0; height: 22px; }
  header strong {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--destroy-text-2);
    font-size: 10px;
    font-weight: 500;
  }
  header strong span { color: var(--destroy-text); font-weight: 650; }
  .media-tabs { display: flex; align-items: center; gap: 2px; padding: 2px; border-radius: 8px; background: var(--destroy-surface); }
  .media-tabs button, .media-close, .media-rotate {
    border: 0;
    background: transparent;
    color: var(--destroy-text-2);
    border-radius: 6px;
    font: 600 9px/1 inherit;
    cursor: pointer;
  }
  .media-tabs button { padding: 4px 7px; }
  .media-tabs button[aria-selected="true"] { background: var(--destroy-surface-strong); color: var(--destroy-text); }
  .media-close { width: 20px; height: 20px; padding: 0; font-size: 14px; }
  .media-rotate {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 20px;
    padding: 0 7px;
    background: var(--destroy-surface);
  }
  .media-rotate:hover:not(:disabled), .media-rotate:focus-visible { color: var(--destroy-text); }
  .media-rotate:disabled { opacity: .45; cursor: default; }
  .media-results { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 5px; }
  .media-result-wrap { position: relative; min-width: 0; }
  .media-result-wrap > button:first-child {
    position: relative;
    width: 100%;
    min-width: 0;
    padding: 0;
    overflow: hidden;
    border: 1px solid var(--destroy-line-strong);
    border-radius: 9px;
    background: #0e0e0e;
    cursor: pointer;
  }
  .media-results button.media-result--selected { border-color: var(--destroy-blue); box-shadow: 0 0 0 1px var(--destroy-blue); }
  .media-results img { display: block; width: 100%; height: 62px; object-fit: cover; background: #0e0e0e; }
  .favorite-button { position: absolute; top: 5px; right: 5px; display: grid; place-items: center; width: 20px; height: 20px; padding: 0; border: 0; border-radius: 999px; background: rgba(0,0,0,.72); color: rgba(255,255,255,.72); cursor: pointer; }
  .favorite-button:hover, .favorite-button:focus-visible { color: #fff; outline: 1px solid rgba(255,255,255,.5); }
  .favorite-button.favorited { color: #ff7890; }
  .favorite-button.favorited :global(svg) { fill: currentColor; }
  .media-number {
    position: absolute;
    left: 5px;
    bottom: 5px;
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    border-radius: 999px;
    background: rgba(0, 0, 0, .78);
    color: #fff;
    font-size: 8px;
    font-weight: 700;
  }
  footer { display: flex; justify-content: space-between; gap: 10px; color: var(--destroy-text-3); font-size: 8px; }
  small { color: var(--destroy-text-3); font: inherit; }
  .media-leading {
    margin: 0;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--destroy-text-2);
    font-size: 9px;
  }
  .media-leading span { color: var(--destroy-text-3); }
  .media-loading, .media-empty, .media-error { min-height: 62px; display: grid; place-items: center; color: var(--destroy-text-2); font-size: 10px; text-align: center; }
  .media-error { color: var(--destroy-red); }
  @media (prefers-reduced-motion: reduce) { * { scroll-behavior: auto; } }
</style>
