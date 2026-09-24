<script lang="ts">
  import { Accessibility, Check, Mic, Monitor, Search, ShieldCheck } from "@lucide/svelte";
  import { fly, slide } from "svelte/transition";
  import SettingsIcon from "./SettingsIcon.svelte";
  import LocalModelPicker from "./LocalModelPicker.svelte";
  import WritingSettings from "./WritingSettings.svelte";
  import SettingsConnections from "./SettingsConnections.svelte";
  import SettingsData from "./SettingsData.svelte";
  import ShortcutKeys from "./ShortcutKeys.svelte";

  type SettingsTab = "speech" | "system" | "data" | "writing" | "connections";

  let {
    initialTab = "speech",
    speech,
    permissions,
    shortcut,
    devices,
    mic,
    localModels,
    integrations,
    native,
    localBusy,
    user,
    scope,
    vars,
    linksJson,
    deleteConfirm,
    backendUrl,
    backendToken,
    connecting,
    micTesting = false,
    micTestLevel = 0,
    onDownload,
    onSelectModel,
    onRemoveModel,
    onMic,
    onShortcutSave,
    onShortcutChange,
    onMicChange,
    onSaveExtras,
    onVarsChange,
    onLinksChange,
    onExport,
    onDeletePrompt,
    onDelete,
    onKeepData,
    onBackendUrl,
    onBackendToken,
    onConnectBackend,
    onConnectDrive,
    onRefreshDrive,
    onDisconnectDrive,
    onSaveComposio,
    onSaveGiphy,
    onDisconnectGiphy,
    composioKey,
    giphyKey,
    onComposioKey,
    onGiphyKey,
    onOpenOnboarding,
    onModelChange = () => {},
    onRestartOnboarding = () => {},
    onResetPrompt = () => {},
    onBack,
  }: any = $props();

  const tabs = [
    ["speech", "Speech"],
    ["system", "System"],
    ["data", "Data"],
  ] as const;

  let active = $state<SettingsTab>("speech");
  let search = $state("");
  let modelOpen = $state(false);
  let micOpen = $state(false);
  let shortcutEditing = $state(false);
  let dataVisited = $state(false);

  $effect(() => {
    const requested = normalizeTab(initialTab);
    active = requested;
    if (requested === "data") dataVisited = true;
    modelOpen = false;
    micOpen = false;
    shortcutEditing = false;
  });

  function normalizeTab(value: unknown): SettingsTab {
    if (value === "system" || value === "data" || value === "writing" || value === "connections") return value;
    return "speech";
  }

  function selectTab(tab: SettingsTab) {
    active = tab;
    if (tab === "data") dataVisited = true;
    modelOpen = false;
    micOpen = false;
    shortcutEditing = false;
  }

  function inputValue(event: Event): string {
    return (event.currentTarget as HTMLInputElement | HTMLTextAreaElement).value;
  }

  function matchesSearch(...terms: string[]): boolean {
    const query = search.trim().toLowerCase();
    return !query || terms.some((term) => term.toLowerCase().includes(query));
  }

  function motionDuration(): number {
    return typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches ? 0 : 190;
  }

  function modelLabel(): string {
    if (speech?.provider === "local") {
      return speech.localModelName
        || speech.modelName
        || localModels?.models?.find((model: any) => model.id === localModels.selectedModelId)?.name
        || "Local model not selected";
    }
    if (speech?.provider === "backend") return "Remote speech service";
    return speech?.provider || "Not configured";
  }

  function microphoneLabel(): string {
    if (!mic) return "System default";
    return devices?.find((device: MediaDeviceInfo) => device.deviceId === mic)?.label || "Selected microphone";
  }

  function statusLabel(allowed: boolean): string {
    return native ? (allowed ? "Allowed" : "Needs permission") : "Native app only";
  }

  function tabIndex(tab: SettingsTab): number {
    return tab === "system" ? 1 : tab === "data" ? 2 : 0;
  }

  function searchLabel(): string {
    return active === "data" ? "data" : active === "system" ? "system" : "speech";
  }
</script>

<section class="settings-shell" aria-label="Settings">
  <div class="settings-frame">
    <header class="settings-header">
      <div class="settings-title-row">
        <h1>Settings</h1>
        <button class="panel-close" type="button" onclick={onBack} aria-label="Close Settings">
          <SettingsIcon expanded />
        </button>
      </div>

      <label class="search-field" aria-label={"Search " + searchLabel() + " settings"}>
        <Search size={18} strokeWidth={1.7} aria-hidden="true" />
        <input bind:value={search} type="search" placeholder={"Search " + searchLabel() + " settings"} />
      </label>

      <div class="settings-tabs" aria-label="Settings sections" role="tablist" style={"--active-shift:" + (tabIndex(active) * 100) + "%"}>
        {#each tabs as [id, label]}
          <button
            type="button"
            class:active={active === id}
            aria-selected={active === id}
            role="tab"
            onclick={() => selectTab(id)}
          >{label}</button>
        {/each}
      </div>
    </header>

    <div class="settings-scroll">
      {#key active}
        <div class="tab-panel" in:fly={{ y: 8, duration: motionDuration() }}>
          {#if active === "speech"}
            <section class="settings-section" aria-labelledby="voice-input-heading">
              <h2 id="voice-input-heading">Voice &amp; input</h2>
              <div class="soft-group">
                {#if matchesSearch("speech", "speech model", "model", "local model")}
                  <div class="setting-row">
                    <div class="setting-copy">
                      <strong>Speech model</strong>
                      <small>{modelLabel()}</small>
                    </div>
                    <button class="soft-button" type="button" onclick={() => { modelOpen = !modelOpen; onModelChange(); }}>
                      {modelOpen ? "Close" : "Change"}
                    </button>
                  </div>
                  {#if modelOpen}
                    <div class="option-panel" transition:slide={{ duration: motionDuration() }}>
                      {#if speech?.provider === "local"}
                        <LocalModelPicker
                          state={localModels}
                          {native}
                          compact
                          busy={localBusy}
                          onDownload={onDownload}
                          onSelect={onSelectModel}
                          onRemove={onRemoveModel}
                          onNativeOnly={onOpenOnboarding}
                        />
                      {:else}
                        <p>Select a speech provider in setup to change the active model.</p>
                      {/if}
                    </div>
                  {/if}
                {/if}

                {#if matchesSearch("microphone", "input", "recording", "device")}
                  <div class="setting-row">
                    <div class="setting-copy">
                      <strong>Microphone</strong>
                      <small>{permissions?.microphone ? "Ready to record." : "Allow microphone access to record."}</small>
                    </div>
                    <div class="row-actions">
                      <button class="soft-button" type="button" onclick={onMic}>{micTesting ? "Stop test" : "Test"}</button>
                      <button class="soft-button" type="button" onclick={() => micOpen = !micOpen} aria-expanded={micOpen}>
                        {micOpen ? "Close" : "Change"}
                      </button>
                    </div>
                  </div>
                  {#if micTesting}
                    <div class="meter-row" role="status">
                      <span>Live microphone level</span>
                      <meter class="mic-meter" min="0" max="1" value={Math.max(0, Math.min(1, micTestLevel))} aria-label="Microphone level"></meter>
                    </div>
                  {/if}
                  {#if micOpen}
                    <div class="option-panel compact-panel" transition:slide={{ duration: motionDuration() }}>
                      <label for="settings-microphone">Input device</label>
                      <select id="settings-microphone" value={mic} onchange={onMicChange}>
                        <option value="">System default</option>
                        {#each devices ?? [] as device}
                          <option value={device.deviceId}>{device.label || "Microphone"}</option>
                        {/each}
                      </select>
                      <p>{microphoneLabel()}</p>
                    </div>
                  {/if}
                {/if}

                {#if matchesSearch("shortcut", "hotkey", "keyboard", "dictation")}
                  <div class="setting-row">
                    <div class="setting-copy">
                      <strong>Shortcut</strong>
                      <small>Key combination to start dictation.</small>
                    </div>
                    {#if shortcutEditing}
                      <div class="shortcut-editor">
                        <input class="shortcut-input" value={shortcut} aria-label="Dictation shortcut" oninput={(event) => onShortcutChange(inputValue(event))} />
                        <button class="primary-action" type="button" onclick={() => { onShortcutSave(); shortcutEditing = false; }}>Save</button>
                      </div>
                    {:else}
                      <div class="shortcut-actions">
                        <ShortcutKeys value={shortcut} compact />
                        <button class="soft-button" type="button" onclick={() => shortcutEditing = true}>Change</button>
                      </div>
                    {/if}
                  </div>
                {/if}

                {#if matchesSearch("language", "locale", "speech")}
                  <div class="setting-row">
                    <div class="setting-copy">
                      <strong>Language</strong>
                      <small>Dictation language.</small>
                    </div>
                    <span class="setting-value">{speech?.language || "Not set"}</span>
                  </div>
                {/if}
              </div>
              {#if !matchesSearch("speech", "speech model", "model", "local model", "microphone", "input", "recording", "device", "shortcut", "hotkey", "keyboard", "dictation", "language", "locale", "setup", "onboarding", "restart", "reset", "delete", "local data", "models")}
                <p class="empty-state" role="status">No speech settings match “{search}”. <button type="button" onclick={() => search = ""}>Clear search</button></p>
              {/if}
            </section>

            {#if matchesSearch("setup", "onboarding", "restart", "reset", "delete", "local data", "models")}
            <section class="settings-section setup-section" aria-labelledby="setup-heading">
              <h2 id="setup-heading">Your setup</h2>
              <div class="soft-group">
                {#if matchesSearch("setup", "onboarding", "restart")}
                  <div class="setting-row setup-row">
                    <div class="setting-copy">
                      <strong>Restart onboarding</strong>
                      <small>Walk through the setup process again.</small>
                    </div>
                    <button class="soft-button" type="button" onclick={onRestartOnboarding}>Restart</button>
                  </div>
                {/if}
                {#if matchesSearch("setup", "reset", "delete", "local data", "models")}
                  <div class="setting-row setup-row danger-row">
                    <div class="setting-copy">
                      <strong>Delete local data &amp; reset</strong>
                      <small>Confirmation required. Downloaded models can be kept.</small>
                    </div>
                    <button class="danger-button" type="button" onclick={onResetPrompt}>Reset</button>
                  </div>
                {/if}
              </div>
            </section>
            {/if}
          {:else if active === "system"}
            <section class="settings-section" aria-labelledby="system-heading">
              <h2 id="system-heading">System status</h2>
              <div class="soft-group">
                {#if matchesSearch("system", "native", "desktop", "status")}
                  <div class="status-row">
                    <Monitor size={18} strokeWidth={1.6} aria-hidden="true" />
                    <div class="setting-copy"><strong>Desktop app</strong><small>{native ? "Native capabilities are available." : "Run the native app to manage system permissions."}</small></div>
                    <span class:status-ok={native} class="status-pill">{native ? "Ready" : "Preview"}</span>
                  </div>
                {/if}
                {#if matchesSearch("microphone", "permission", "recording")}
                  <div class="status-row">
                    <Mic size={18} strokeWidth={1.6} aria-hidden="true" />
                    <div class="setting-copy"><strong>Microphone access</strong><small>Required before Destroy can record.</small></div>
                    <span class:status-ok={native && permissions?.microphone} class="status-pill">{statusLabel(Boolean(permissions?.microphone))}</span>
                  </div>
                {/if}
                {#if matchesSearch("accessibility", "insertion", "permission")}
                  <div class="status-row">
                    <Accessibility size={18} strokeWidth={1.6} aria-hidden="true" />
                    <div class="setting-copy"><strong>Direct insertion</strong><small>Accessibility access lets Destroy insert into the original field.</small></div>
                    <span class:status-ok={native && permissions?.accessibility} class="status-pill">{statusLabel(Boolean(permissions?.accessibility))}</span>
                  </div>
                {/if}
              </div>
              {#if native && permissions?.microphone && permissions?.accessibility}
                <p class="system-note"><Check size={14} aria-hidden="true" /> Permissions are ready for direct dictation.</p>
              {:else}
                <p class="system-note"><ShieldCheck size={14} aria-hidden="true" /> Permission state is reported by the native app; no settings are changed here.</p>
              {/if}
              {#if !matchesSearch("system", "native", "desktop", "status", "microphone", "permission", "recording", "accessibility", "insertion")}
                <p class="empty-state" role="status">No system settings match “{search}”. <button type="button" onclick={() => search = ""}>Clear search</button></p>
              {/if}
            </section>
          {:else if active === "writing"}
            <div class="legacy-view"><WritingSettings {scope} styleAvailable={speech?.provider === "backend"} /></div>
          {:else if active === "connections"}
            <div class="legacy-view">
              <SettingsConnections
                {integrations}
                {native}
                {connecting}
                {composioKey}
                {giphyKey}
                {backendUrl}
                {backendToken}
                {onConnectDrive}
                {onRefreshDrive}
                {onDisconnectDrive}
                {onSaveComposio}
                {onSaveGiphy}
                {onDisconnectGiphy}
                {onComposioKey}
                {onGiphyKey}
                {onBackendUrl}
                {onBackendToken}
                {onConnectBackend}
              />
            </div>
          {/if}
        </div>
      {/key}
      {#if dataVisited}
        <div class="persistent-panel" class:is-hidden={active !== "data"} aria-hidden={active !== "data"}>
          <SettingsData
            {vars}
            {linksJson}
            {user}
            searchQuery={search}
            remoteConnected={speech?.provider === "backend" && !!user}
            {deleteConfirm}
            {onVarsChange}
            {onLinksChange}
            {onSaveExtras}
            {onExport}
            {onDeletePrompt}
            {onDelete}
            {onKeepData}
            {onResetPrompt}
            {onRestartOnboarding}
          />
        </div>
      {/if}
    </div>
  </div>

  <div class="settings-art" aria-hidden="true"></div>
</section>

<style>
  .settings-shell { position: relative; isolation: isolate; display: flex; flex: 1; min-height: 0; height: auto; margin: 0 -38px -14px; padding: 8px 48px 0; overflow: hidden; background: #000; color: #f1eee7; }
  .settings-frame { position: relative; z-index: 1; display: flex; flex: 1; min-width: 0; min-height: 0; flex-direction: column; }
  .settings-shell .settings-header { display: block; flex: 0 0 auto; margin: 0; }
  .settings-shell .settings-header > .settings-title-row { display: flex; align-items: center; justify-content: space-between; gap: 18px; margin-top: 0; }
  .settings-header h1 { margin: 0; color: #f1eee7; font-family: DestroyEditorial, Georgia, serif; font-size: clamp(38px, 6vw, 52px); font-weight: 500; letter-spacing: -.035em; line-height: .95; }
  .panel-close { width: 38px; height: 38px; display: grid; place-items: center; padding: 8px; border: 1px solid transparent; border-radius: 50%; background: transparent; color: #f1eee7; }
  .panel-close:hover { border-color: #ffffff12; background: #111; }
  .search-field { display: flex; flex-direction:row; align-items: center; gap: 12px; min-height: 54px; margin: 25px 0 26px; padding: 0 17px; border: 1px solid #ffffff18; border-radius: 999px; background: #111; color: #aaa9a2; }
  .search-field:focus-within { border-color: #ffffff32; box-shadow: 0 0 0 3px #ffffff08; }
  .search-field input { min-width: 0; width: 100%; border: 0; outline: 0; background: transparent; color: #f1eee7; padding: 0; font-size: 16px; }
  .search-field input::placeholder { color: #777a72; }
  .search-field input::-webkit-search-cancel-button { filter: invert(1); opacity: .55; }
  .settings-shell .settings-tabs { position: relative; display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 4px; margin: 0; padding: 4px; border: 1px solid #ffffff18; border-bottom: 1px solid #ffffff18; border-radius: 999px; background: #111; }
  .settings-shell .settings-tabs::before { content: ""; position: absolute; z-index: 0; top: 4px; bottom: 4px; left: 4px; width: calc((100% - 8px) / 3); border-radius: 999px; background: #ebe7df; box-shadow: 0 3px 12px #0008; pointer-events: none; transform: translateX(var(--active-shift)); transition: transform 190ms ease; }
  .settings-shell .settings-tabs button { position: relative; z-index: 1; min-height: 48px; border: 0; border-radius: 999px; background: transparent; color: #aaa9a2; padding: 7px 10px; font-size: 16px; transition: color 190ms ease; }
  .settings-tabs button:hover { background: #1b1b1b; color: #f1eee7; }
  .settings-shell .settings-tabs button:hover { background: transparent; }
  .settings-shell .settings-tabs button.active { background: transparent; box-shadow: none; color: #111; }
  .settings-shell .settings-tabs button.active::after { display: none; }
  .settings-tabs button:focus-visible { outline: 2px solid #d94c40; outline-offset: 2px; }
  .settings-scroll { min-height: 0; flex: 1; overflow: auto; padding: 28px 2px 180px; scrollbar-width: thin; }
  .tab-panel, .persistent-panel { min-width: 0; }
  .persistent-panel.is-hidden { display: none; }
  .persistent-panel:not(.is-hidden){animation:data-enter 220ms ease-out both}
  @keyframes data-enter{from{opacity:0;transform:translateY(8px)}to{opacity:1;transform:translateY(0)}}
  .settings-section { min-width: 0; margin: 0 0 30px; }
  .settings-section h2 { margin: 0 0 13px 4px; color: #f1eee7; font-family: DestroyEditorial, Georgia, serif; font-size: 30px; font-weight: 500; letter-spacing: -.015em; line-height: 1; }
  .soft-group { overflow: hidden; border: 1px solid #ffffff14; border-radius: 20px; background: #111; box-shadow: 0 14px 40px #0005; }
  .setting-row { display: grid; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 16px; min-height: 92px; padding: 18px 20px; }
  .status-row + .status-row { border-top: 1px solid #ffffff0e; }
  .setting-copy { min-width: 0; }
  .setting-copy strong { display: block; color: #f1eee7; font-size: 23px; font-weight: 500; letter-spacing: -.015em; line-height: 1.05; }
  .setting-copy small { display: block; margin-top: 6px; color: #aaa9a2; font-size: 17px; line-height: 1.3; }
  .setting-value { max-width: 260px; overflow-wrap: anywhere; color: #aaa9a2; font-size: 16px; text-align: right; }
  .row-actions, .shortcut-actions, .shortcut-editor { display: flex; align-items: center; justify-content: flex-end; gap: 10px; min-width: 0; }
  .soft-button, .primary-action, .danger-button { min-height: 46px; border-radius: 999px; padding: 8px 20px; font-size: 16px; white-space: nowrap; }
  .soft-button { border: 1px solid #ffffff16; background: #1b1b1b; color: #f1eee7; }
  .soft-button:hover { border-color: #ffffff2b; background: #242424; }
  .primary-action { border: 1px solid #ebe7df; background: #ebe7df; color: #111; }
  .primary-action:hover { border-color: #fffaf0; background: #fffaf0; }
  .danger-button { border: 1px solid #71372f; background: #2b1715; color: #ff9c90; }
  .danger-button:hover { border-color: #a95249; background: #3a1d1a; }
  .shortcut-actions :global(.shortcut-keys) { gap:8px; margin:0 14px 0 0; }
  .shortcut-actions :global(.shortcut-keys kbd){min-width:44px;height:44px;font-size:21px}
  .shortcut-editor { flex-wrap: wrap; }
  .shortcut-input { width: 190px; min-height: 46px; border: 1px solid #ffffff1c; border-radius: 999px; background: #1b1b1b; color: #f1eee7; padding: 8px 13px; font-size: 16px; }
  .shortcut-input:focus-visible { border-color: #d94c40; outline: 2px solid #d94c40; outline-offset: 2px; }
  .option-panel { overflow: hidden; border-top: 1px solid #ffffff0e; padding: 16px 20px 19px; color: #aaa9a2; }
  .option-panel p { margin: 10px 0 0; color: #aaa9a2; font-size: 14px; line-height: 1.5; }
  .compact-panel label { display: block; margin: 0 0 7px; color: #aaa9a2; font-size: 14px; }
  .compact-panel select { width: 100%; min-height: 46px; border: 1px solid #ffffff18; border-radius: 10px; background: #1b1b1b; color: #f1eee7; padding: 8px 10px; font-size: 16px; }
  .compact-panel select:focus-visible { border-color: #d94c40; outline: 2px solid #d94c40; outline-offset: 2px; }
  .meter-row { display: flex; align-items: center; gap: 12px; border-top: 1px solid #ffffff0e; padding: 12px 20px; color: #aaa9a2; font-size: 14px; }
  .mic-meter { width: 100px; height: 5px; accent-color: #ebe7df; }
  .setup-section { margin-top: 28px; }
  .setup-row { min-height: 76px; }
  .danger-row .setting-copy strong { color: #e95d52; }
  .system-note { display: flex; align-items: flex-start; gap: 7px; margin: 14px 5px 0; color: #aaa9a2; font-size: 11px; line-height: 1.5; }
  .system-note :global(svg) { flex: 0 0 auto; color: #ebe7df; margin-top: 1px; }
  .status-row { display: grid; grid-template-columns: 24px minmax(0, 1fr) auto; align-items: center; gap: 13px; min-height: 78px; padding: 15px 20px; }
  .status-row > :global(svg) { color: #aaa9a2; }
  .status-pill { color: #aaa9a2; font-size: 15px; white-space: nowrap; }
  .status-pill.status-ok { color: #ebe7df; }
  .status-pill.status-ok::before { content: ""; display: inline-block; width: 5px; height: 5px; margin: 0 6px 2px 0; border-radius: 50%; background: #ebe7df; }
  .empty-state { margin: 12px 5px 0; color: #aaa9a2; font-size: 14px; line-height: 1.5; }
  .empty-state button { border: 0; background: transparent; color: #f1eee7; padding: 0; text-decoration: underline; text-underline-offset: 3px; }
  .legacy-view { min-width: 0; }
  .settings-art { position:absolute;z-index:0;right:0;bottom:0;width:min(400px,65%);aspect-ratio:1217 / 550;pointer-events:none;background:url('/art/daily-ink-approved-source.png') right top / 122.1857% auto no-repeat;mix-blend-mode:lighten;clip-path:polygon(49% 0,100% 0,100% 98%,45% 98%,45% 91%,0 91%,0 69%,27.8% 69%,30% 60%,33% 50%,40% 40%,49% 33%);filter:contrast(1.12)}
  .settings-shell .setting-row{border-bottom:0;min-height:84px}
  .setting-copy strong{font-family:DestroyEditorial,Georgia,serif;font-size:24px}
  .settings-shell .settings-tabs::before{background:#303030;transition:transform 320ms cubic-bezier(.22,1,.36,1)}
  .settings-shell .settings-tabs button.active{color:#f1eee7}
  .settings-header h1{font-size:54px}
  .search-field input{font-size:18px}
  @media (max-width: 620px) {
    .settings-shell { margin: 0 -20px -12px; padding: 18px 24px 0; }
    .settings-header h1 { font-size: 42px; }
    .search-field { margin-top: 21px; }
    .settings-scroll { padding-top: 24px; }
    .setting-row { grid-template-columns: minmax(0, 1fr); gap: 11px; align-items: start; }
    .row-actions, .shortcut-actions, .shortcut-editor { justify-content: flex-start; }
    .setting-value { max-width: none; text-align: left; }
    .status-row { grid-template-columns: 24px minmax(0, 1fr); }
    .status-pill { grid-column: 2; }
    .shortcut-actions :global(.shortcut-keys) { transform-origin: left center; margin-left: -9px; }
  }
  @media (prefers-reduced-motion: reduce) {
    .persistent-panel:not(.is-hidden){animation:none}
    .settings-tabs::before, .settings-tabs button, .settings-art, .soft-button, .primary-action, .danger-button { transition: none; }
    .settings-shell :global(*) { scroll-behavior: auto !important; }
  }
</style>
