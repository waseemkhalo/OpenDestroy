<script lang="ts">
  import { Check, Cloud, Folder, Image, KeyRound, Laptop, LockKeyhole, Mic, Keyboard, TextCursorInput } from "@lucide/svelte";
  import LocalModelPicker from "./LocalModelPicker.svelte";
  import type { PublicLocalModels } from "./localModels";

  type Provider = "local" | "openai" | "gemini" | "backend";
  type SpeechStatus = {
    provider: Provider | null;
    ready: boolean;
    modelReady: boolean;
    modelDownloading?: boolean;
    localModelId?: string | null;
    openaiKeyPresent: boolean;
    geminiKeyPresent: boolean;
    user_id: string | null;
    backend_url: string;
    error?: string;
  };
  type Integrations = {
    composioKeyPresent: boolean;
    driveConnected: boolean;
    driveAccountLabel: string;
    giphyKeyPresent: boolean;
  };
  type Permissions = { microphone: boolean; accessibility: boolean };

  let {
    step,
    speech,
    selectedProvider,
    speechKey,
    permissions,
    shortcut,
    integrations,
    composioKey,
    giphyKey,
    localModels,
    modelBusy,
    backendUrl,
    backendToken,
    connecting,
    native,
    onProvider,
    onSpeechKey,
    onConfigureSpeech,
    onDownloadModel,
    onSelectModel,
    onRemoveModel,
    onMic,
    onAccessibility,
    onShortcutChange,
    onShortcutSave,
    onIntegration,
    onComposioKey,
    onSaveComposio,
    onConnectDrive,
    onRefreshDrive,
    onDisconnectDrive,
    onGiphyKey,
    onSaveGiphy,
    onDisconnectGiphy,
    onBackendUrl,
    onBackendToken,
    onConnectBackend,
    onContinue,
    onBack,
    onFinish,
    onNativeOnly,
  }: {
    step: number;
    speech: SpeechStatus;
    selectedProvider: Provider;
    speechKey: string;
    permissions: Permissions;
    shortcut: string;
    integrations: Integrations;
    composioKey: string;
    giphyKey: string;
    localModels: PublicLocalModels;
    modelBusy: boolean;
    backendUrl: string;
    backendToken: string;
    connecting: boolean;
    native: boolean;
    onProvider: (provider: Provider) => void;
    onSpeechKey: (value: string) => void;
    onConfigureSpeech: () => void;
    onDownloadModel: (modelId: string) => void;
    onSelectModel: (modelId: string) => void;
    onRemoveModel: (modelId: string) => void;
    onMic: () => void;
    onAccessibility: () => void;
    onShortcutChange: (value: string) => void;
    onShortcutSave: () => void;
    onIntegration: (value: "giphy" | "composio") => void;
    onComposioKey: (value: string) => void;
    onSaveComposio: () => void;
    onConnectDrive: () => void;
    onRefreshDrive: () => void;
    onDisconnectDrive: () => void;
    onGiphyKey: (value: string) => void;
    onSaveGiphy: () => void;
    onDisconnectGiphy: () => void;
    onBackendUrl: (value: string) => void;
    onBackendToken: (value: string) => void;
    onConnectBackend: () => void;
    onContinue: () => void;
    onBack: () => void;
    onFinish: () => void;
    onNativeOnly: () => void;
  } = $props();

  let selectedIntegration = $state<"giphy" | "composio" | null>(null);
  let advancedOpen = $state(false);
  const steps = ["Speech", "Your Mac", "Connections"];
  const providerCopy: Record<Provider, { title: string; detail: string }> = {
    local: { title: "On this Mac", detail: "Download a model. Audio stays on this Mac." },
    openai: { title: "OpenAI", detail: "Use your own API key. Provider usage is billed to you." },
    gemini: { title: "Google Gemini", detail: "Use your own API key. Provider usage is billed to you." },
    backend: { title: "Existing backend", detail: "Keep using the relay you already run." },
  };
  const statusText = $derived(
    speech.error || (speech.provider === "local" && speech.modelDownloading ? "Downloading the local model…" : speech.ready && (speech.provider !== "local" || speech.modelReady) ? "Ready." : "Not configured yet.")
  );
  function nativeOnly() { if (!native) onNativeOnly(); }
  function continueStep() { onContinue(); }
</script>

<section class="onboarding" aria-label="Destroy Dictation setup">
  <header class="walkthrough-head" style="justify-content:flex-end">
    <span class="step-count wordmark" style="font-size:11px;letter-spacing:normal;font-weight:400">{step + 1} / 3 — {steps[step]}</span>
  </header>

  {#if advancedOpen}
    <div class="setup-content">
      <button class="back-link" type="button" onclick={() => advancedOpen = false}>← Back to speech options</button>
      <h1>Use an existing backend.</h1>
      <p class="lede">Keep the relay and provider configuration you already run.</p>
      <form onsubmit={(event) => { event.preventDefault(); onConnectBackend(); }}>
        <label>Service URL<input value={backendUrl} type="url" required oninput={(event) => onBackendUrl((event.currentTarget as HTMLInputElement).value)} /></label>
        <label>Access token<input value={backendToken} type="password" autocomplete="off" required oninput={(event) => onBackendToken((event.currentTarget as HTMLInputElement).value)} /></label>
        <p class="privacy-note"><LockKeyhole size={14} aria-hidden="true" /> Stored in macOS Keychain. The configured backend controls provider routing.</p>
        <button class="primary" type="submit" disabled={connecting || !native}>{connecting ? "Connecting…" : "Connect backend"}</button>
      </form>
    </div>
  {:else if step === 0}
    <div class="setup-shell">
      <div class="setup-content">
        <h1>How would you like to dictate?</h1>
        <p class="lede">Choose where your speech is processed.</p>
        <div class="choice-grid" role="listbox" aria-label="Speech provider">
          {#each (["local", "openai", "gemini"] as Provider[]) as provider}
            <button type="button" class:selected={selectedProvider === provider} class="choice-card" aria-pressed={selectedProvider === provider} onclick={() => onProvider(provider)}>
              {#if provider === "local"}<Laptop size={19} strokeWidth={1.7} aria-hidden="true" />{:else if provider === "openai"}<KeyRound size={19} strokeWidth={1.7} aria-hidden="true" />{:else}<Cloud size={19} strokeWidth={1.7} aria-hidden="true" />{/if}
              <strong>{providerCopy[provider].title}</strong>
              <small>{providerCopy[provider].detail}</small>
            </button>
          {/each}
        </div>

        {#if selectedProvider === "local"}
          <LocalModelPicker state={localModels} {native} compact busy={modelBusy} onDownload={onDownloadModel} onSelect={onSelectModel} onRemove={onRemoveModel} onNativeOnly={nativeOnly} />
          <p class="notice">Local mode never switches to a cloud provider automatically.</p>
        {:else}
          <label>{providerCopy[selectedProvider].title} API key<input type="password" autocomplete="off" value={speechKey} oninput={(event) => onSpeechKey((event.currentTarget as HTMLInputElement).value)} /></label>
          <div class="key-actions"><button class="primary" type="button" onclick={() => native ? onConfigureSpeech() : nativeOnly()} disabled={!speechKey || !native}>Save key</button><span><LockKeyhole size={13} aria-hidden="true" /> Keychain only</span></div>
          <p class="notice">Audio goes only to the provider you choose. Destroy does not export your key.</p>
        {/if}
        {#if speech.error || speech.ready || speech.modelDownloading}<p class="setup-status" role="status">{statusText}</p>{/if}
      </div>
      <div class="step-actions" style="padding-left:clamp(0px,calc((100vw - 620px) * 2.7),160px)"><button class="quiet" type="button" onclick={() => advancedOpen = true}>Advanced connection</button><button class="primary" type="button" onclick={continueStep}>Continue →</button></div>
    </div>
  {:else if step === 1}
    <div class="setup-shell">
      <div class="setup-content">
        <h1>Make this Mac ready.</h1>
        <p class="lede">Speak into the field you’re already using.</p>
        <div class="permission-list">
          <div class="setup-row"><Mic size={18} strokeWidth={1.7} aria-hidden="true" /><div><strong>Microphone</strong><small>Record while you hold your shortcut.</small></div><button type="button" onclick={() => native ? onMic() : nativeOnly()}>{permissions.microphone ? "Allowed" : "Allow microphone"}</button></div>
          <div class="setup-row"><TextCursorInput size={18} strokeWidth={1.7} aria-hidden="true" /><div><strong>Accessibility</strong><small>Insert into the original field. Otherwise, copy and paste.</small></div><button type="button" onclick={() => native ? onAccessibility() : nativeOnly()}>{permissions.accessibility ? "Allowed" : "Open System Settings"}</button></div>
          <div class="setup-row"><Keyboard size={18} strokeWidth={1.7} aria-hidden="true" /><div><strong>Hold to speak</strong><small>Release to insert; choose an available combination.</small></div><div class="shortcut"><input aria-label="Hold to speak shortcut" value={shortcut} oninput={(event) => onShortcutChange((event.currentTarget as HTMLInputElement).value)} /><button type="button" onclick={() => native ? onShortcutSave() : nativeOnly()}>Save</button></div></div>
        </div>
        <p class="notice">The recording HUD stays out of your way. Screen Recording is not required.</p>
      </div>
      <div class="step-actions" style="padding-left:clamp(0px,calc((100vw - 620px) * 2.7),160px)"><button class="quiet" type="button" onclick={onBack}>← Back</button><button class="primary" type="button" onclick={continueStep}>Continue →</button></div>
    </div>
  {:else}
    <div class="setup-shell">
      <div class="setup-content">
        <h1>What else would you like to insert?</h1>
        <p class="lede">Optional connections. Add them now or anytime in Settings.</p>
        <div class="choice-grid integrations" role="listbox" aria-label="Optional integrations">
          <button type="button" class:selected={selectedIntegration === "giphy"} class="choice-card" aria-pressed={selectedIntegration === "giphy"} onclick={() => { selectedIntegration = "giphy"; onIntegration("giphy"); }}><Image size={19} strokeWidth={1.7} aria-hidden="true" /><strong>GIFs & stickers</strong><small>{integrations.giphyKeyPresent ? "Personal key saved in Keychain." : "Search directly with your own key."}</small></button>
          <button type="button" class:selected={selectedIntegration === "composio"} class="choice-card" aria-pressed={selectedIntegration === "composio"} onclick={() => { selectedIntegration = "composio"; onIntegration("composio"); }}><Folder size={19} strokeWidth={1.7} aria-hidden="true" /><strong>Google Drive</strong><small>{integrations.driveConnected ? `Connected — ${integrations.driveAccountLabel}` : "Find a file and insert its existing link."}</small></button>
        </div>

      {#if selectedIntegration === "giphy"}
        <div class="integration-detail"><h2>Use your own GIPHY key.</h2><p>Search words go directly to GIPHY. Results insert as attributed source links.</p><label>GIPHY API key<input type="password" autocomplete="off" value={giphyKey} oninput={(event) => onGiphyKey((event.currentTarget as HTMLInputElement).value)} /></label><div class="key-actions"><button class="primary" type="button" onclick={() => native ? onSaveGiphy() : nativeOnly()} disabled={!giphyKey || !native}>{integrations.giphyKeyPresent ? "Replace key" : "Save key"}</button>{#if integrations.giphyKeyPresent}<button type="button" onclick={() => native ? onDisconnectGiphy() : nativeOnly()}>Disconnect</button>{/if}</div><p class="notice">Your key stays in Keychain. Favorites are disabled for direct GIPHY searches.</p></div>
      {:else if selectedIntegration === "composio"}
        <div class="integration-detail"><h2>Use your own Composio project.</h2><p>Save a dedicated project key, connect Google, then return here and refresh.</p><label>Composio project key<input type="password" autocomplete="off" value={composioKey} oninput={(event) => onComposioKey((event.currentTarget as HTMLInputElement).value)} /></label><div class="key-actions"><button class="primary" type="button" onclick={() => native ? onSaveComposio() : nativeOnly()} disabled={!composioKey || !native}>Save project key</button>{#if integrations.composioKeyPresent}<button type="button" onclick={() => native ? onConnectDrive() : nativeOnly()}>Connect Google Drive →</button>{/if}</div>{#if integrations.driveConnected}<p class="connected-line"><Check size={14} aria-hidden="true" /> {integrations.driveAccountLabel || "Google Drive connected"}<button type="button" onclick={() => native ? onDisconnectDrive() : nativeOnly()}>Disconnect</button></p>{:else if integrations.composioKeyPresent}<p class="notice">A browser window may open for Google approval. After returning, refresh the connection.</p><button type="button" onclick={() => native ? onRefreshDrive() : nativeOnly()}>Refresh Google Drive</button>{/if}<p class="notice">Composio handles the Google connection. Sharing permissions stay unchanged.</p></div>
      {:else}
        <p class="notice">No Destroy account is required. These connections are optional and can be managed in Settings.</p>
      {/if}
      </div>
      <div class="step-actions" style="padding-left:clamp(0px,calc((100vw - 620px) * 2.7),160px)"><button class="quiet" type="button" onclick={onBack}>← Back</button><button class="primary" type="button" onclick={onFinish}>Finish setup →</button></div>
    </div>
  {/if}
</section>

<style>
  .onboarding{max-width:680px;min-height:0;overflow:hidden;display:flex;flex-direction:column;flex:1;padding:0 4px 8px}.walkthrough-head{display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid #363735;padding:8px 0 14px;margin-bottom:24px}.wordmark{font-size:10px;letter-spacing:.15em;font-weight:600}.step-count{font-size:11px;color:#95958f}.setup-shell{max-width:590px;min-height:0;display:flex;flex-direction:column;flex:1}.setup-content{min-height:0;overflow:auto;display:flex;flex-direction:column;flex:1}.lede{margin:0 0 20px}.choice-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:11px;margin:4px 0 20px}.choice-grid.integrations{grid-template-columns:repeat(2,minmax(0,1fr))}.choice-card{min-height:140px;padding:17px;text-align:left;display:flex;flex-direction:column;align-items:flex-start;gap:0;background:linear-gradient(130deg,#282a2b,#202223);border-color:#484b49}.choice-card:hover{border-color:#777a73}.choice-card.selected{background:linear-gradient(130deg,#372b29,#242322);border-color:#7d4b44;box-shadow:inset 2px 0 #d83a30}.choice-card strong{font-size:15px;font-weight:500;margin-top:15px;margin-bottom:7px}.choice-card small,.setup-row small{font-size:11px;line-height:1.5;color:#95958f}.setup-row{display:flex;align-items:center;gap:13px;padding:15px 0;border-bottom:1px solid #363735}.setup-row>div:first-of-type{flex:1;min-width:0}.setup-row strong{display:block;font-size:13px;font-weight:500;margin-bottom:4px}.setup-row>button{font-size:11px;white-space:nowrap}.notice,.privacy-note{display:flex;align-items:flex-start;gap:7px;margin:14px 0;font-size:11px;line-height:1.55;color:#95958f}.setup-status{margin:2px 0 0;font-size:11px;color:#c9c6bf}.key-actions{display:flex;align-items:center;gap:10px;flex-wrap:wrap}.key-actions span{display:inline-flex;gap:5px;align-items:center;font-size:10px;color:#95958f}.step-actions{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-top:0;padding-top:34px;flex:none}.back-link{align-self:flex-start;border:0;background:transparent;color:#c9c6bf;padding:0 0 22px;font-size:12px}.permission-list{margin-top:4px}.shortcut{display:flex;gap:6px;align-items:center}.shortcut input{width:160px;padding:7px 9px}.shortcut button{padding:7px 9px;font-size:10px}.integration-detail{background:#222427;border:1px solid #363735;border-radius:8px;padding:17px}.integration-detail h2{margin:0 0 7px}.integration-detail p{font-size:12px;line-height:1.55;color:#c9c6bf}.connected-line{display:flex;align-items:center;gap:7px;color:#d7d5ce!important}.connected-line button{margin-left:auto;font-size:10px}.integration-detail>.notice{border-top:1px solid #363735;padding-top:12px}.quiet{background:transparent;border-color:transparent;color:#c9c6bf}.quiet:hover{background:#242628}.onboarding button:disabled{cursor:default}@media(max-width:620px){.choice-grid,.choice-grid.integrations{grid-template-columns:1fr}.choice-card{min-height:90px}.choice-card strong{margin-top:8px}.setup-row{flex-wrap:wrap}.setup-row>div:first-of-type{min-width:calc(100% - 38px)}.step-actions{padding-top:24px}.shortcut{width:100%;margin-left:31px}.shortcut input{flex:1;width:auto}}
</style>
