<script lang="ts">
  import { Check, Download, HardDrive, Trash2 } from "@lucide/svelte";
  import { canRemoveModel, formatBytes, freeSpaceMessage, modelProgress, type PublicLocalModel, type PublicLocalModels } from "./localModels";

  let {
    state: modelsState,
    native,
    busy = false,
    compact = false,
    onDownload,
    onSelect,
    onRemove,
    onNativeOnly,
  }: {
    state: PublicLocalModels;
    native: boolean;
    busy?: boolean;
    compact?: boolean;
    onDownload: (modelId: string) => void;
    onSelect: (modelId: string) => void;
    onRemove: (modelId: string) => void;
    onNativeOnly: () => void;
  } = $props();

  const actionDisabled = $derived(!native || busy || Boolean(modelsState.downloadingModelId));
  const hasCatalog = $derived(modelsState.models.length > 0);
  let familyFilter = $state("whisper");
  let sizeFilter = $state("all");
  let languageFilter = $state("all");
  let filterSelection = $state<string | null>(null);
  const engineKey = (model: PublicLocalModel) => model.engine.trim().toLowerCase();
  const familyLabel = (engine: string) => engine ? engine.charAt(0).toUpperCase() + engine.slice(1) : "Other";
  const languageLabel = (language: string) => language === "en" ? "English" : language;
  const whisperSize = (model: PublicLocalModel) => {
    const match = model.id.toLowerCase().match(/whisper-(tiny|base|small)(?:-|$)/);
    return match?.[1] || "other";
  };
  const families = $derived([...new Set(modelsState.models.map(engineKey))]);
  const effectiveFamily = $derived(familyFilter === "all" ? "all" : families.includes(familyFilter) ? familyFilter : families.includes("whisper") ? "whisper" : (families[0] || "all"));
  const sizeOptions = $derived([...new Set(modelsState.models.filter(model => engineKey(model) === "whisper").map(whisperSize))]);
  const languageOptions = $derived([...new Set(modelsState.models.filter(model => engineKey(model) === "whisper").map(model => model.language))]);
  const visibleModels = $derived(modelsState.models.filter(model => {
    if (effectiveFamily !== "all" && engineKey(model) !== effectiveFamily) return false;
    if (effectiveFamily === "whisper" && sizeFilter !== "all" && whisperSize(model) !== sizeFilter) return false;
    if (effectiveFamily === "whisper" && languageFilter !== "all" && model.language !== languageFilter) return false;
    return true;
  }));
  const activeModel = (model: PublicLocalModel) => modelsState.downloadingModelId === model.id;
  const selectedModel = (model: PublicLocalModel) => modelsState.selectedModelId === model.id;
  const displayName = (model: PublicLocalModel) => model.name.replaceAll(" · ", " — ");
  const progress = (model: PublicLocalModel) => modelProgress(modelsState, model);
  $effect(() => {
    const selected = modelsState.models.find(model => model.id === modelsState.selectedModelId);
    if (!selected || selected.id === filterSelection) return;
    filterSelection = selected.id;
    familyFilter = engineKey(selected) || "all";
    if (engineKey(selected) === "whisper") {
      sizeFilter = whisperSize(selected);
      languageFilter = selected.language;
    } else {
      sizeFilter = "all";
      languageFilter = "all";
    }
  });
  function trigger(action: () => void) {
    if (!native) onNativeOnly();
    else action();
  }
</script>

<section class:compact aria-label="Local speech models" class="local-models">
  <div class="local-models-head">
    <div>
      <strong>Local speech model</strong>
      <small>Audio stays on this Mac. Choose one model to use for dictation.</small>
    </div>
    {#if native && modelsState.availableBytes !== null}
      <span class="storage"><HardDrive size={13} aria-hidden="true" /> {formatBytes(modelsState.availableBytes)} free</span>
    {/if}
  </div>

  {#if modelsState.error}
    <p class="model-error" role="alert">{modelsState.error}</p>
  {/if}

  {#if !native}
    <p class="model-empty">The native app provides the available model catalog and download sizes.</p>
  {:else if !hasCatalog}
    <p class="model-empty">The native model catalog is unavailable. No model choices are shown.</p>
  {:else}
    <div class="model-filters" aria-label="Filter local speech models">
      <div class="family-tabs" role="tablist" aria-label="Model engine">
        <button type="button" class:chosen={effectiveFamily === "all"} role="tab" aria-selected={effectiveFamily === "all"} onclick={() => familyFilter = "all"}>All</button>
        {#each families as family}
          <button type="button" class:chosen={effectiveFamily === family} role="tab" aria-selected={effectiveFamily === family} onclick={() => { familyFilter = family; if (family !== "whisper") { sizeFilter = "all"; languageFilter = "all"; } }}>{familyLabel(family)}</button>
        {/each}
      </div>
      {#if effectiveFamily === "whisper"}
        <div class="whisper-filters">
          <label>Size<select aria-label="Whisper size" bind:value={sizeFilter}><option value="all">All sizes</option>{#each sizeOptions as size}<option value={size}>{size.charAt(0).toUpperCase() + size.slice(1)}</option>{/each}</select></label>
          <label>Language<select aria-label="Whisper language" bind:value={languageFilter}><option value="all">All languages</option>{#each languageOptions as language}<option value={language}>{languageLabel(language)}</option>{/each}</select></label>
        </div>
      {/if}
    </div>
    <div class="model-grid">
      {#each visibleModels as model (model.id)}
        {@const percentage = progress(model)}
        <article class:supported={model.supported} class:selected={selectedModel(model)} class:active={activeModel(model)} class="model-card">
          <div class="model-card-copy">
            <div class="model-name-line">
              <strong>{displayName(model)}</strong>
              {#if selectedModel(model)}<span class="selected-badge"><Check size={11} aria-hidden="true" /> Selected</span>{/if}
            </div>
            <div class="model-meta"><span>{model.engine}</span><span>{languageLabel(model.language)}</span><span>{formatBytes(model.downloadBytes)}</span></div>
          </div>

          {#if !model.supported}
            <p class="model-unavailable">{model.unavailableReason || "Unavailable on this Mac."}</p>
          {:else if activeModel(model)}
            <div class="model-progress" aria-label={`Downloading ${displayName(model)}`}>
              <div class="progress-track" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={percentage ?? undefined} aria-valuetext={`${formatBytes(modelsState.downloadedBytes)} of ${formatBytes(modelsState.totalBytes)}`}><span style={`width:${percentage ?? 0}%`}></span></div>
              <small class="progress-label"><span>{formatBytes(modelsState.downloadedBytes)}</span><span>of {formatBytes(modelsState.totalBytes)}</span>{#if percentage !== null}<span>{percentage}%</span>{/if}</small>
            </div>
          {:else if model.installed}
            <div class="model-actions">
              {#if !selectedModel(model)}
                <button type="button" disabled={actionDisabled} onclick={() => trigger(() => onSelect(model.id))}>Use</button>
              {:else}
                <span class="installed-label">Installed</span>
              {/if}
              <button class="remove" type="button" disabled={!canRemoveModel(modelsState, model, busy)} onclick={() => onRemove(model.id)}><Trash2 size={12} aria-hidden="true" /> Remove</button>
            </div>
          {:else}
            <div class="model-actions model-download">
              <span class="model-space">{freeSpaceMessage(modelsState, model)}</span>
              <button type="button" disabled={actionDisabled || modelsState.availableBytes === null || modelsState.availableBytes < model.downloadBytes} onclick={() => trigger(() => onDownload(model.id))}><Download size={12} aria-hidden="true" /> Download</button>
            </div>
          {/if}
        </article>
      {/each}
    </div>
  {/if}
</section>

<style>
  .local-models{margin:14px 0 4px;padding:13px 0 0;border-top:1px solid #363735}.local-models-head{display:flex;justify-content:space-between;align-items:flex-start;gap:12px;margin-bottom:9px}.local-models-head strong{display:block;font-size:13px;font-weight:500;margin-bottom:4px}.local-models-head small,.model-card small{display:block;color:#95958f;font-size:10px;line-height:1.45}.storage{display:inline-flex;align-items:center;gap:4px;white-space:nowrap;color:#95958f;font-size:10px}.model-filters{display:grid;gap:7px;margin:6px 0 8px}.family-tabs{display:flex;gap:5px}.family-tabs button{padding:5px 8px;font-size:10px;background:transparent;color:#aaa9a1;border-color:#3d403e}.family-tabs button.chosen{background:#302523;color:#ffcbc5;border-color:#8b4b43}.whisper-filters{display:grid;grid-template-columns:1fr 1fr;gap:7px}.whisper-filters label{margin:0;gap:4px;font-size:9px;color:#95958f}.whisper-filters select{padding:6px 8px;font-size:10px}.model-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:7px;max-height:248px;overflow:auto;padding:1px 2px 2px 0;scrollbar-width:thin}.model-card{min-width:0;min-height:88px;padding:10px 11px;background:#202223;border:1px solid #383b3a;border-radius:7px;display:flex;flex-direction:column;justify-content:space-between;gap:8px}.model-card.selected{border-color:#b8332b;background:#2a2423;box-shadow:inset 2px 0 #d83a30}.model-card.active{border-color:#d83a30}.model-card-copy{min-width:0}.model-name-line{display:flex;align-items:flex-start;gap:6px;min-width:0}.model-name-line strong{font-size:12px;line-height:1.25;font-weight:550;overflow-wrap:anywhere}.selected-badge{display:inline-flex;align-items:center;gap:3px;flex-shrink:0;border:1px solid #75423d;border-radius:999px;padding:2px 5px;color:#ffcbc5;font-size:9px;line-height:1}.model-meta{display:flex;flex-wrap:wrap;gap:7px;margin-top:4px}.model-meta span{color:#95958f;font-size:10px;line-height:1.35}.model-actions{display:flex;align-items:center;justify-content:space-between;gap:6px;min-height:22px}.model-actions button{display:inline-flex;align-items:center;gap:4px;padding:5px 7px;font-size:10px}.model-actions .remove{background:transparent;color:#b9b5ad;border-color:transparent}.model-actions .remove:hover{background:#2b2e2e}.installed-label{font-size:10px;color:#aaa9a1}.model-space{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:#95958f;font-size:9px}.model-download{align-items:center}.model-progress{display:grid;gap:4px}.model-progress small{font-size:9px}.progress-label{display:flex!important;gap:7px}.progress-track{height:4px;background:#3c3d3b;border-radius:99px;overflow:hidden}.progress-track span{display:block;height:100%;min-width:2px;background:#d83a30;border-radius:inherit;transition:width .2s ease}.model-unavailable,.model-error,.model-empty{margin:6px 0 0;font-size:10px;line-height:1.45;color:#c8a39f}.model-error{border-left:2px solid #d83a30;padding-left:8px}.model-empty{color:#95958f}.compact .model-grid{max-height:224px}.compact .model-card{min-height:80px;padding:9px 10px}.compact .local-models-head small{max-width:360px}@media(max-width:620px){.model-grid{grid-template-columns:1fr;max-height:300px}.local-models-head{flex-direction:column;gap:4px}}@media(prefers-reduced-motion:reduce){.progress-track span{transition:none}}
</style>
