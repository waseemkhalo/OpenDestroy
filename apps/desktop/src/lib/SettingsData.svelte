<script lang="ts">
  import { Download, Plus, RotateCcw, Trash2, X } from "@lucide/svelte";
  import {
    parseLinksJson,
    parseVariablesJson,
    serializeLinks,
    serializeVariables,
    validateLinkRows,
    validateVariableRows,
    type SettingsLinkRow,
    type SettingsVariableRow,
  } from "./settingsForms";

  let {
    vars,
    linksJson,
    user,
    remoteConnected,
    deleteConfirm,
    onVarsChange,
    onLinksChange,
    onSaveExtras,
    onExport,
    onDeletePrompt,
    onDelete,
    onKeepData,
    onResetPrompt,
    onRestartOnboarding,
    searchQuery = "",
  }: {
    vars: string;
    linksJson: string;
    user: string | null;
    remoteConnected: boolean;
    deleteConfirm: boolean;
    onVarsChange: (value: string) => void;
    onLinksChange: (value: string) => void;
    onSaveExtras: () => void;
    onExport: () => void;
    onDeletePrompt: () => void;
    onDelete: () => void;
    onKeepData: () => void;
    onResetPrompt: () => void;
    onRestartOnboarding: () => void;
    searchQuery?: string;
  } = $props();

  let variableRows = $state<SettingsVariableRow[]>([]);
  let linkRows = $state<SettingsLinkRow[]>([]);
  let rawVars = $state("");
  let rawLinks = $state("");
  let loadedVars = $state<string | null>(null);
  let loadedLinks = $state<string | null>(null);
  let varsError = $state("");
  let linksError = $state("");
  let varsRawError = $state("");
  let linksRawError = $state("");
  let advancedVarsOpen = $state(false);
  let advancedLinksOpen = $state(false);
  let rowId = 0;

  $effect(() => {
    const next = vars;
    if (loadedVars !== null && (next === rawVars || next === loadedVars)) return;
    const parsed = parseVariablesJson(next);
    rawVars = next;
    loadedVars = next;
    varsError = parsed.error;
    varsRawError = parsed.error;
    if (!parsed.error) variableRows = parsed.rows;
    if (parsed.error) advancedVarsOpen = true;
  });

  $effect(() => {
    const next = linksJson;
    if (loadedLinks !== null && (next === rawLinks || next === loadedLinks)) return;
    const parsed = parseLinksJson(next);
    rawLinks = next;
    loadedLinks = next;
    linksError = parsed.error;
    linksRawError = parsed.error;
    if (!parsed.error) linkRows = parsed.rows;
    if (parsed.error) advancedLinksOpen = true;
  });

  function nextRowId(prefix: string): string {
    rowId += 1;
    return `${prefix}-new-${rowId}`;
  }

  function syncVariables() {
    const validation = validateVariableRows(variableRows);
    varsError = validation;
    varsRawError = "";
    if (validation) return;
    rawVars = serializeVariables(variableRows);
    onVarsChange(rawVars);
  }

  function syncLinks() {
    const validation = validateLinkRows(linkRows);
    linksError = validation;
    linksRawError = "";
    if (validation) return;
    rawLinks = serializeLinks(linkRows);
    onLinksChange(rawLinks);
  }

  function updateVariable(id: string, field: "name" | "value", value: string) {
    variableRows = variableRows.map((row) => (row.id === id ? { ...row, [field]: value } : row));
    syncVariables();
  }

  function updateLink(id: string, field: "name" | "url", value: string) {
    linkRows = linkRows.map((row) => (row.id === id ? { ...row, [field]: value } : row));
    syncLinks();
  }

  function addVariable() {
    variableRows = [...variableRows, { id: nextRowId("variable"), name: "", value: "" }];
    varsError = "Give the new variable a name and value.";
  }

  function removeVariable(id: string) {
    variableRows = variableRows.filter((row) => row.id !== id);
    syncVariables();
  }

  function addLink() {
    linkRows = [...linkRows, { id: nextRowId("link"), name: "", url: "" }];
    linksError = "Give the new link a name and HTTPS URL.";
  }

  function removeLink(id: string) {
    linkRows = linkRows.filter((row) => row.id !== id);
    syncLinks();
  }

  function updateRawVariables(value: string) {
    rawVars = value;
    onVarsChange(value);
    const parsed = parseVariablesJson(value);
    varsError = parsed.error;
    varsRawError = parsed.error;
    if (!parsed.error) variableRows = parsed.rows;
    if (parsed.error) advancedVarsOpen = true;
  }

  function updateRawLinks(value: string) {
    rawLinks = value;
    onLinksChange(value);
    const parsed = parseLinksJson(value);
    linksError = parsed.error;
    linksRawError = parsed.error;
    if (!parsed.error) linkRows = parsed.rows;
    if (parsed.error) advancedLinksOpen = true;
  }

  function saveExtras() {
    varsRawError = parseVariablesJson(rawVars).error;
    linksRawError = parseLinksJson(rawLinks).error;
    varsError = validateVariableRows(variableRows) || varsRawError;
    linksError = validateLinkRows(linkRows) || linksRawError;
    if (varsError || linksError) {
      if (varsError) advancedVarsOpen = true;
      if (linksError) advancedLinksOpen = true;
      return;
    }
    rawVars = serializeVariables(variableRows);
    rawLinks = serializeLinks(linkRows);
    onVarsChange(rawVars);
    onLinksChange(rawLinks);
    onSaveExtras();
  }

  function inputValue(event: Event): string {
    return (event.currentTarget as HTMLInputElement | HTMLTextAreaElement).value;
  }

  function matchesSearch(...terms: string[]): boolean {
    const query = searchQuery.trim().toLowerCase();
    return !query || terms.some((term) => term.toLowerCase().includes(query));
  }
</script>

<div class="settings-data" aria-label="Data">
  {#if searchQuery.trim() && !matchesSearch("variables", "snippet variables", "links", "saved links", "advanced json", "save", "changes", "export", "personal data", "delete", "remote", "installation", "onboarding", "reset", "local data")}
    <p class="empty-guidance" role="status">No data settings match “{searchQuery}”.</p>
  {/if}
  {#if matchesSearch("variables", "snippet variables", "advanced json")}
  <section class="data-section" aria-labelledby="variables-heading">
    <div class="data-section-head">
      <div><h3 id="variables-heading">Snippet variables</h3><p>Use <code>{'{{company}}'}</code> in a voice shortcut.</p></div>
      <button class="small-action" type="button" disabled={Boolean(varsRawError)} onclick={addVariable}><Plus size={14} aria-hidden="true" />Add variable</button>
    </div>
    {#if variableRows.length}
      <div class="variable-list" aria-label="Snippet variables">
        {#each variableRows as row (row.id)}
          <div class="editor-row variable-row">
            <div class="field-wrap"><label for={`variable-name-${row.id}`}>Name</label><input id={`variable-name-${row.id}`} class="settings-input" disabled={Boolean(varsRawError)} value={row.name} placeholder="company" aria-invalid={Boolean(varsError)} oninput={(event) => updateVariable(row.id, "name", inputValue(event))} /></div>
            <div class="field-wrap"><label for={`variable-value-${row.id}`}>Value</label><input id={`variable-value-${row.id}`} class="settings-input" disabled={Boolean(varsRawError)} value={row.value} placeholder="Destroy" aria-invalid={Boolean(varsError)} oninput={(event) => updateVariable(row.id, "value", inputValue(event))} /></div>
            <button class="remove-action" type="button" disabled={Boolean(varsRawError)} aria-label={`Remove variable ${row.name || "without a name"}`} onclick={() => removeVariable(row.id)}><X size={14} aria-hidden="true" /></button>
          </div>
        {/each}
      </div>
    {:else}
      <p class="empty-guidance">{varsRawError ? "Structured editing is paused until Advanced JSON is valid." : "No variables yet. Add one for a reusable voice shortcut."}</p>
    {/if}
    {#if varsError}<p class="inline-error" role="alert">{varsError}</p>{/if}
    <details class="advanced-disclosure" bind:open={advancedVarsOpen}>
      <summary>Advanced JSON <span>Keep the existing object format</span></summary>
      <label for="variables-json">Variables JSON</label>
      <textarea id="variables-json" class="settings-textarea" rows="4" value={rawVars} aria-invalid={Boolean(varsError)} oninput={(event) => updateRawVariables(inputValue(event))}></textarea>
    </details>
  </section>
  {/if}

  {#if matchesSearch("links", "saved links", "advanced json")}
  <section class="data-section" aria-labelledby="links-heading">
    <div class="data-section-head">
      <div><h3 id="links-heading">Saved links</h3><p>Speak a saved name to insert its existing HTTPS link.</p></div>
      <button class="small-action" type="button" disabled={Boolean(linksRawError)} onclick={addLink}><Plus size={14} aria-hidden="true" />Add link</button>
    </div>
    {#if linkRows.length}
      <div class="link-list" aria-label="Saved links">
        {#each linkRows as row (row.id)}
          <div class="editor-row link-row">
            <div class="field-wrap"><label for={`link-name-${row.id}`}>Name</label><input id={`link-name-${row.id}`} class="settings-input" disabled={Boolean(linksRawError)} value={row.name} placeholder="Product deck" aria-invalid={Boolean(linksError)} oninput={(event) => updateLink(row.id, "name", inputValue(event))} /></div>
            <div class="field-wrap"><label for={`link-url-${row.id}`}>HTTPS URL</label><input id={`link-url-${row.id}`} class="settings-input" disabled={Boolean(linksRawError)} value={row.url} type="url" inputmode="url" placeholder="https://example.com/deck" aria-invalid={Boolean(linksError)} oninput={(event) => updateLink(row.id, "url", inputValue(event))} /></div>
            <button class="remove-action" type="button" disabled={Boolean(linksRawError)} aria-label={`Remove link ${row.name || "without a name"}`} onclick={() => removeLink(row.id)}><X size={14} aria-hidden="true" /></button>
          </div>
        {/each}
      </div>
    {:else}
      <p class="empty-guidance">{linksRawError ? "Structured editing is paused until Advanced JSON is valid." : "No saved links yet. Add an HTTPS link for quick, reliable retrieval."}</p>
    {/if}
    {#if linksError}<p class="inline-error" role="alert">{linksError}</p>{/if}
    <details class="advanced-disclosure" bind:open={advancedLinksOpen}>
      <summary>Advanced JSON <span>Keep optional keywords and existing fields visible</span></summary>
      <label for="links-json">Saved links JSON</label>
      <textarea id="links-json" class="settings-textarea" rows="5" value={rawLinks} aria-invalid={Boolean(linksError)} oninput={(event) => updateRawLinks(inputValue(event))}></textarea>
    </details>
  </section>
  {/if}

  {#if matchesSearch("save", "changes", "variables", "links")}
  <div class="save-line">
    <button class="primary-action" type="button" disabled={!user || Boolean(varsError || linksError)} onclick={saveExtras}>Save changes</button>
    <span>{user ? "Changes are ready to save." : "Connect a backend to save variables and links."}</span>
  </div>
  {/if}

  {#if matchesSearch("export", "personal data", "delete", "remote")}
  <section class="data-section action-section" aria-labelledby="export-heading">
    <div class="data-action-row">
      <span class="action-icon"><Download size={16} aria-hidden="true" /></span>
      <div><h3 id="export-heading">Export personal data</h3><p>Save a local copy of your Destroy data.</p></div>
      <button type="button" onclick={onExport}>Export</button>
    </div>
    <div class="data-action-row">
      <span class="action-icon danger-icon"><Trash2 size={16} aria-hidden="true" /></span>
      <div><h3>Delete remote data</h3><p>Deletes data from this backend and disconnects this app.</p></div>
      <button class="danger-action" type="button" disabled={!remoteConnected} onclick={() => remoteConnected && onDeletePrompt()}>Delete…</button>
    </div>
    {#if deleteConfirm && remoteConnected}
      <div class="confirmation-row" role="alert">
        <p>This clears saved preferences, snippets, vocabulary, favorites, and links from the connected backend.</p>
        <div><button class="danger-action" type="button" onclick={onDelete}>Confirm deletion</button><button type="button" onclick={onKeepData}>Keep data</button></div>
      </div>
    {/if}
  </section>
  {/if}

  {#if matchesSearch("installation", "onboarding", "reset", "local data")}
  <section class="data-section action-section local-actions" aria-labelledby="local-heading">
    <div class="section-label" id="local-heading">This installation</div>
    <div class="data-action-row">
      <span class="action-icon"><RotateCcw size={16} aria-hidden="true" /></span>
      <div><h3>Restart onboarding</h3><p>Walk through setup again while keeping your saved data.</p></div>
      <button type="button" onclick={onRestartOnboarding}>Restart</button>
    </div>
    <div class="data-action-row">
      <span class="action-icon danger-icon"><Trash2 size={16} aria-hidden="true" /></span>
      <div><h3>Delete local data &amp; reset</h3><p>Remove this installation’s setup and local cache. Remote data stays untouched.</p></div>
      <button class="danger-action" type="button" onclick={onResetPrompt}>Delete &amp; reset…</button>
    </div>
  </section>
  {/if}
</div>

<style>
  .settings-data { padding: 0 0 30px; color: #f1eee7; }
  .data-section { margin: 0 0 14px; padding: 20px; border: 1px solid #ffffff14; border-radius: 20px; background: #111; box-shadow: 0 14px 40px #0005; }
  .data-section-head { display: flex; align-items: start; justify-content: space-between; gap: 16px; }
  .data-section h3, .data-action-row h3 { margin: 0; color: #f1eee7; font-size: 14px; font-weight: 520; }
  .data-section-head p, .data-action-row p { margin: 5px 0 0; color: #aaa9a2; font-size: 11px; line-height: 1.5; }
  code { color: #e4d6c4; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 10px; }
  .small-action { display: inline-flex; align-items: center; gap: 6px; min-height: 34px; border: 1px solid #ffffff16; border-radius: 999px; background: #1b1b1b; color: #f1eee7; padding: 7px 13px; font-size: 11px; white-space: nowrap; }
  .small-action:hover { border-color: #ffffff2b; background: #242424; }
  .variable-list, .link-list { margin-top: 16px; }
  .editor-row { display: grid; align-items: end; gap: 10px; padding: 14px 0 0; border-top: 1px solid #ffffff0e; }
  .variable-row { grid-template-columns: minmax(120px, .7fr) minmax(180px, 1.3fr) 28px; }
  .link-row { grid-template-columns: minmax(120px, .7fr) minmax(220px, 1.3fr) 28px; }
  .field-wrap { min-width: 0; }
  .field-wrap label, .advanced-disclosure label { display: block; margin: 0 0 6px; color: #aaa9a2; font-size: 10px; }
  .settings-input, .settings-textarea { width: 100%; border: 1px solid #ffffff18; border-radius: 10px; background: #1b1b1b; color: #f1eee7; padding: 10px 11px; font-size: 12px; }
  .settings-input { min-height: 40px; }
  .settings-textarea { min-height: 96px; resize: vertical; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 11px; line-height: 1.5; }
  .settings-input::placeholder, .settings-textarea::placeholder { color: #777a72; }
  .settings-input:focus-visible, .settings-textarea:focus-visible { border-color: #d94c40; outline: 2px solid #d94c40; outline-offset: 2px; }
  .settings-input[aria-invalid="true"], .settings-textarea[aria-invalid="true"] { border-color: #a95249; }
  .remove-action { width: 30px; height: 30px; display: grid; place-items: center; padding: 0; border: 1px solid #ffffff16; border-radius: 50%; color: #aaa9a2; background: #1b1b1b; }
  .remove-action:hover { color: #f1eee7; border-color: #ffffff2b; background: #242424; }
  .empty-guidance { margin: 16px 0 0; padding: 13px 14px; border: 1px solid #ffffff12; border-radius: 12px; background: #151515; color: #aaa9a2; font-size: 11px; line-height: 1.5; }
  .inline-error { margin: 10px 0 0; color: #ff9c90; font-size: 11px; line-height: 1.45; }
  .advanced-disclosure { margin-top: 18px; border-top: 1px solid #ffffff0e; padding-top: 2px; }
  .advanced-disclosure summary { display: flex; align-items: center; gap: 5px; min-height: 40px; cursor: pointer; color: #d6d1c7; font-size: 11px; list-style-position: inside; }
  .advanced-disclosure summary span { margin-left: auto; color: #777a72; text-align: right; }
  .advanced-disclosure[open] summary { margin-bottom: 12px; color: #f1eee7; }
  .advanced-disclosure[open] .settings-textarea { animation: data-panel-in 180ms ease both; }
  .advanced-disclosure label { margin-top: 0; }
  .save-line { display: flex; align-items: center; gap: 12px; padding: 17px 0 3px; }
  .save-line span { color: #aaa9a2; font-size: 10px; }
  .primary-action { min-height: 38px; border: 1px solid #ebe7df; border-radius: 999px; background: #ebe7df; color: #111; padding: 8px 17px; font-size: 12px; }
  .primary-action:hover { border-color: #fffaf0; background: #fffaf0; }
  .data-action-row { display: grid; grid-template-columns: 30px minmax(0, 1fr) auto; align-items: center; gap: 12px; min-height: 62px; padding: 8px 0; }
  .action-icon { width: 30px; height: 30px; display: grid; place-items: center; color: #d6d1c7; }
  .danger-icon { color: #e95d52; }
  .data-action-row button { min-height: 34px; border: 1px solid #ffffff16; border-radius: 999px; background: #1b1b1b; color: #f1eee7; padding: 7px 13px; font-size: 11px; white-space: nowrap; }
  .data-action-row button:hover { border-color: #ffffff2b; background: #242424; }
  .danger-action { border-color: #71372f !important; background: #2b1715 !important; color: #ff9c90 !important; }
  .data-action-row button.danger-action:hover { border-color: #a95249 !important; background: #3a1d1a !important; }
  .confirmation-row { margin: 8px 0 0 42px; padding: 13px 14px; border: 1px solid #71372f; border-radius: 12px; background: #241414; }
  .confirmation-row p { margin: 0 0 10px; color: #c9c3b9; font-size: 11px; line-height: 1.5; }
  .confirmation-row button { margin-right: 8px; font-size: 11px; }
  .local-actions { padding-bottom: 20px; }
  .section-label { margin-bottom: 4px; color: #777a72; font-size: 10px; font-weight: 650; letter-spacing: .12em; text-transform: uppercase; }
  @keyframes data-panel-in { from { opacity: 0; transform: translateY(-5px); } to { opacity: 1; transform: translateY(0); } }
  @media (max-width: 620px) {
    .data-section-head { align-items: start; }
    .variable-row, .link-row { grid-template-columns: minmax(0, 1fr) 30px; }
    .variable-row .field-wrap:nth-child(2), .link-row .field-wrap:nth-child(2) { grid-column: 1 / -1; grid-row: 2; }
    .variable-row .remove-action, .link-row .remove-action { grid-column: 2; grid-row: 1; }
    .save-line { align-items: start; flex-direction: column; gap: 8px; }
    .data-action-row { grid-template-columns: 30px minmax(0, 1fr); }
    .data-action-row button { grid-column: 2; justify-self: start; }
    .confirmation-row { margin-left: 0; }
  }
  @media (prefers-reduced-motion: reduce) {
    .advanced-disclosure[open] .settings-textarea { animation: none; }
  }
</style>
