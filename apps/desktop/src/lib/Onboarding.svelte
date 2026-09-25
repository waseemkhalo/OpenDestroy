<script lang="ts">
  import { Check, Cloud, Folder, Image, KeyRound, Laptop, LockKeyhole, Mic, Keyboard, TextCursorInput } from "@lucide/svelte";
  import LocalModelPicker from "./LocalModelPicker.svelte";
  import ShortcutKeys from "./ShortcutKeys.svelte";
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
  <!-- Same ink language as Home: painted landscape on the right, editorial type on the left. -->
  <div class="ink-backdrop" aria-hidden="true"></div>

  <header class="walkthrough-head" data-tauri-drag-region>
    <span class="brand">DESTROY</span>
    <ol class="step-rail" aria-label="Setup progress">
      {#each steps as label, index}
        <li class:active={index === step && !advancedOpen} class:done={index < step && !advancedOpen} aria-current={index === step && !advancedOpen ? "step" : undefined}>
          <span class="step-number">{index < step && !advancedOpen ? "✓" : String(index + 1).padStart(2, "0")}</span>{label}
        </li>
      {/each}
    </ol>
  </header>

  {#key `${advancedOpen}-${step}`}
  {#if advancedOpen}
    <div class="setup-shell">
      <div class="setup-content">
        <button class="back-link" type="button" onclick={() => advancedOpen = false}>← Back to speech options</button>
        <h1>Use an existing backend.</h1>
        <p class="lede">Keep the relay and provider configuration you already run.</p>
        <form class="panel" onsubmit={(event) => { event.preventDefault(); onConnectBackend(); }}>
          <label>Service URL<input value={backendUrl} type="url" required oninput={(event) => onBackendUrl((event.currentTarget as HTMLInputElement).value)} /></label>
          <label>Access token<input value={backendToken} type="password" autocomplete="off" required oninput={(event) => onBackendToken((event.currentTarget as HTMLInputElement).value)} /></label>
          <p class="privacy-note"><LockKeyhole size={14} aria-hidden="true" /> Stored in macOS Keychain. The configured backend controls provider routing.</p>
          <button class="primary" type="submit" disabled={connecting || !native}>{connecting ? "Connecting…" : "Connect backend"}</button>
        </form>
      </div>
    </div>
  {:else if step === 0}
    <div class="setup-shell">
      <div class="setup-content">
        <h1>How would you like to dictate?</h1>
        <p class="lede">Choose where your speech is processed. You can change this anytime in Settings.</p>
        <div class="choice-grid" role="listbox" aria-label="Speech provider">
          {#each (["local", "openai", "gemini"] as Provider[]) as provider}
            <button type="button" class:selected={selectedProvider === provider} class="choice-card" aria-pressed={selectedProvider === provider} onclick={() => onProvider(provider)}>
              {#if provider === "local"}<Laptop size={20} strokeWidth={1.6} aria-hidden="true" />{:else if provider === "openai"}<KeyRound size={20} strokeWidth={1.6} aria-hidden="true" />{:else}<Cloud size={20} strokeWidth={1.6} aria-hidden="true" />{/if}
              <strong>{providerCopy[provider].title}</strong>
              <small>{providerCopy[provider].detail}</small>
            </button>
          {/each}
        </div>

        <div class="panel">
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
      </div>
      <div class="step-actions"><button class="quiet" type="button" onclick={() => advancedOpen = true}>Advanced connection</button><button class="primary" type="button" onclick={continueStep}>Continue →</button></div>
    </div>
  {:else if step === 1}
    <div class="setup-shell">
      <div class="setup-content">
        <h1>Make this Mac ready.</h1>
        <p class="lede">Speak into the field you’re already using.</p>
        <div class="panel permission-list">
          <div class="setup-row"><Mic size={20} strokeWidth={1.6} aria-hidden="true" /><div><strong>Microphone</strong><small>Record while you hold your shortcut.</small></div><button type="button" class:granted={permissions.microphone} onclick={() => native ? onMic() : nativeOnly()}>{#if permissions.microphone}<Check size={13} aria-hidden="true" /> Allowed{:else}Allow microphone{/if}</button></div>
          <div class="setup-row"><TextCursorInput size={20} strokeWidth={1.6} aria-hidden="true" /><div><strong>Accessibility</strong><small>Insert into the original field. Otherwise, copy and paste.</small></div><button type="button" class:granted={permissions.accessibility} onclick={() => native ? onAccessibility() : nativeOnly()}>{#if permissions.accessibility}<Check size={13} aria-hidden="true" /> Allowed{:else}Open System Settings{/if}</button></div>
          <div class="setup-row shortcut-row"><Keyboard size={20} strokeWidth={1.6} aria-hidden="true" /><div><strong>Hold to speak</strong><small>Release to insert; choose an available combination.</small></div><ShortcutKeys value={shortcut} compact interactive /><div class="shortcut"><input aria-label="Hold to speak shortcut" value={shortcut} oninput={(event) => onShortcutChange((event.currentTarget as HTMLInputElement).value)} /><button type="button" onclick={() => native ? onShortcutSave() : nativeOnly()}>Save</button></div></div>
        </div>
        <p class="notice">The recording HUD stays out of your way. Screen Recording is not required.</p>
      </div>
      <div class="step-actions"><button class="quiet" type="button" onclick={onBack}>← Back</button><button class="primary" type="button" onclick={continueStep}>Continue →</button></div>
    </div>
  {:else}
    <div class="setup-shell">
      <div class="setup-content">
        <h1>What else would you like to insert?</h1>
        <p class="lede">Optional connections. Add them now or anytime in Settings.</p>
        <div class="choice-grid integrations" role="listbox" aria-label="Optional integrations">
          <button type="button" class:selected={selectedIntegration === "giphy"} class="choice-card" aria-pressed={selectedIntegration === "giphy"} onclick={() => { selectedIntegration = "giphy"; onIntegration("giphy"); }}><Image size={20} strokeWidth={1.6} aria-hidden="true" /><strong>GIFs & stickers</strong><small>{integrations.giphyKeyPresent ? "Personal key saved in Keychain." : "Search directly with your own key."}</small></button>
          <button type="button" class:selected={selectedIntegration === "composio"} class="choice-card" aria-pressed={selectedIntegration === "composio"} onclick={() => { selectedIntegration = "composio"; onIntegration("composio"); }}><Folder size={20} strokeWidth={1.6} aria-hidden="true" /><strong>Google Drive</strong><small>{integrations.driveConnected ? `Connected — ${integrations.driveAccountLabel}` : "Find a file and insert its existing link."}</small></button>
        </div>

      {#if selectedIntegration === "giphy"}
        <div class="panel integration-detail"><h2>Use your own GIPHY key.</h2><p>Search words go directly to GIPHY. Results insert as attributed source links.</p><label>GIPHY API key<input type="password" autocomplete="off" value={giphyKey} oninput={(event) => onGiphyKey((event.currentTarget as HTMLInputElement).value)} /></label><div class="key-actions"><button class="primary" type="button" onclick={() => native ? onSaveGiphy() : nativeOnly()} disabled={!giphyKey || !native}>{integrations.giphyKeyPresent ? "Replace key" : "Save key"}</button>{#if integrations.giphyKeyPresent}<button type="button" onclick={() => native ? onDisconnectGiphy() : nativeOnly()}>Disconnect</button>{/if}</div><p class="notice">Your key stays in Keychain. Favorites are disabled for direct GIPHY searches.</p></div>
      {:else if selectedIntegration === "composio"}
        <div class="panel integration-detail"><h2>Use your own Composio project.</h2><p>Save a dedicated project key, connect Google, then return here and refresh.</p><label>Composio project key<input type="password" autocomplete="off" value={composioKey} oninput={(event) => onComposioKey((event.currentTarget as HTMLInputElement).value)} /></label><div class="key-actions"><button class="primary" type="button" onclick={() => native ? onSaveComposio() : nativeOnly()} disabled={!composioKey || !native}>Save project key</button>{#if integrations.composioKeyPresent}<button type="button" onclick={() => native ? onConnectDrive() : nativeOnly()}>Connect Google Drive →</button>{/if}</div>{#if integrations.driveConnected}<p class="connected-line"><Check size={14} aria-hidden="true" /> {integrations.driveAccountLabel || "Google Drive connected"}<button type="button" onclick={() => native ? onDisconnectDrive() : nativeOnly()}>Disconnect</button></p>{:else if integrations.composioKeyPresent}<p class="notice">A browser window may open for Google approval. After returning, refresh the connection.</p><button type="button" onclick={() => native ? onRefreshDrive() : nativeOnly()}>Refresh Google Drive</button>{/if}<p class="notice">Composio handles the Google connection. Sharing permissions stay unchanged.</p></div>
      {:else}
        <p class="notice">No Destroy account is required. These connections are optional and can be managed in Settings.</p>
      {/if}
      </div>
      <div class="step-actions"><button class="quiet" type="button" onclick={onBack}>← Back</button><button class="primary" type="button" onclick={onFinish}>Finish setup →</button></div>
    </div>
  {/if}
  {/key}
</section>

<style>
  /* Bleed to the window edges like Home, so the painting frames the whole setup. */
  .onboarding{position:relative;isolation:isolate;flex:1;min-height:0;display:flex;flex-direction:column;margin:-48px -30px -14px;padding:0 clamp(28px,5vw,64px) 28px;overflow:hidden;background:#000;color:#f1eee7}
  .ink-backdrop{position:absolute;inset:0;z-index:0;pointer-events:none;background:url('/art/settings-landscape-v2.png') right bottom / cover no-repeat;filter:contrast(1.12);mix-blend-mode:lighten;mask-image:linear-gradient(90deg,transparent 0%,#0003 30%,#000 62%);-webkit-mask-image:linear-gradient(90deg,transparent 0%,#0003 30%,#000 62%)}
  .walkthrough-head,.setup-shell{position:relative;z-index:1}

  .walkthrough-head{display:flex;justify-content:space-between;align-items:center;gap:24px;min-height:96px;padding:40px 0 18px;flex:none}
  .brand{font-family:DestroyEditorial,Georgia,serif;font-size:22px;letter-spacing:.29em;white-space:nowrap;color:#f1eee7}
  .step-rail{display:flex;gap:clamp(14px,3vw,36px);margin:0;padding:0;list-style:none}
  .step-rail li{position:relative;display:flex;align-items:baseline;gap:8px;padding-bottom:10px;font-size:14px;color:#8e8e89;white-space:nowrap;transition:color 200ms}
  .step-rail li.active{color:#f1eee7}.step-rail li.done{color:#c9c8c3}
  .step-rail li.active::after{content:'';position:absolute;left:-4px;right:-4px;bottom:0;height:8px;background:url('/art/selection-brush.png') center / 100% auto no-repeat;mix-blend-mode:lighten;mask-image:radial-gradient(ellipse at center,#000 35%,transparent 70%);-webkit-mask-image:radial-gradient(ellipse at center,#000 35%,transparent 70%)}
  .step-number{font-family:DestroyEditorial,Georgia,serif;font-size:15px;color:#d94c40}.step-rail li:not(.active):not(.done) .step-number{color:#6e6f6a}

  .setup-shell{max-width:660px;min-height:0;display:flex;flex-direction:column;flex:1;animation:page-settle 340ms cubic-bezier(.22,1,.36,1) both}
  .setup-content{min-height:0;overflow:auto;display:flex;flex-direction:column;flex:1;padding:clamp(8px,3vh,32px) 4px 24px 0;scrollbar-width:thin;mask-image:linear-gradient(180deg,#000 calc(100% - 28px),transparent);-webkit-mask-image:linear-gradient(180deg,#000 calc(100% - 28px),transparent)}
  .onboarding h1{font:500 clamp(38px,5vw,64px)/1.04 DestroyEditorial,Georgia,serif;letter-spacing:-.02em;margin:0 0 14px;color:#f1eee7;text-wrap:balance}
  .onboarding .lede{max-width:520px;margin:0 0 28px;font-size:16px;line-height:1.55;color:#b6b5b0}
  .onboarding h2{font:500 24px/1.2 DestroyEditorial,Georgia,serif;margin:0 0 8px;color:#f1eee7}

  .choice-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:14px;margin:0 0 18px}.choice-grid.integrations{grid-template-columns:repeat(2,minmax(0,1fr))}
  .choice-card{position:relative;min-height:152px;padding:20px;text-align:left;display:flex;flex-direction:column;align-items:flex-start;gap:0;color:#c9c8c3;background:linear-gradient(160deg,#171717e8,#0b0b0be8);border:1px solid #ffffff17;border-radius:16px;backdrop-filter:blur(6px);-webkit-backdrop-filter:blur(6px);transition:border-color 180ms,transform 180ms,box-shadow 180ms}
  .choice-card:hover{background:linear-gradient(160deg,#1d1d1de8,#0e0e0ee8);border-color:#ffffff33;transform:translateY(-1px)}
  .choice-card.selected{color:#f1eee7;border-color:#d94c40a6;background:linear-gradient(160deg,#2a1714ee,#0f0b0aee);box-shadow:0 0 0 1px #d94c4033,0 18px 40px #000a}
  .choice-card strong{position:relative;font:500 25px/1.1 DestroyEditorial,Georgia,serif;margin:18px 0 8px;color:#f1eee7}
  .choice-card small{font-size:12px;line-height:1.5;color:#9d9d97}

  .panel{padding:4px 20px 16px;background:#0d0d0dd9;border:1px solid #ffffff12;border-radius:16px;backdrop-filter:blur(6px);-webkit-backdrop-filter:blur(6px)}
  .panel :global(.local-models){border-top:0;margin-top:0}
  .setup-row{display:flex;align-items:center;gap:16px;padding:18px 0;border-bottom:1px solid #ffffff12}.setup-row:last-child{border-bottom:0}
  .setup-row>div:first-of-type{flex:1;min-width:0}.setup-row strong{display:block;font:500 21px/1.2 DestroyEditorial,Georgia,serif;margin-bottom:3px;color:#f1eee7}.setup-row small{font-size:12px;line-height:1.5;color:#9d9d97}
  .setup-row>:global(svg){flex-shrink:0;color:#c9c8c3}
  .setup-row>button,.shortcut button{white-space:nowrap}
  .granted{display:inline-flex;align-items:center;gap:6px;border-color:transparent!important;background:transparent!important;color:#c9c8c3!important}
  .shortcut{display:flex;gap:6px;align-items:center}.shortcut input{width:150px;padding:8px 12px;border-radius:999px;font-size:12px}
  .shortcut-row :global(.shortcut-keys){margin-bottom:-4px}

  .onboarding :global(button:not(.choice-card)){border-radius:999px}
  .onboarding button:not(.choice-card):not(.quiet):not(.back-link):not(.primary){padding:9px 16px;font-size:13px;background:#1a1a1a;border-color:#ffffff24}
  .onboarding button:not(.choice-card):not(.quiet):not(.back-link):not(.primary):hover{background:#262626}
  .onboarding .primary{padding:11px 22px;font-size:14px;background:#c0352c;border-color:#d94c40;box-shadow:0 10px 24px #b8332b33}.onboarding .primary:hover{background:#d83a30}
  .onboarding input{background:#141414;border-color:#ffffff24;border-radius:10px}.onboarding input:focus{border-color:#d94c40}
  .onboarding label{font-size:13px;color:#d8d5cf}

  .notice,.privacy-note{display:flex;align-items:flex-start;gap:7px;margin:14px 0;font-size:12px;line-height:1.55;color:#8e8e89}
  .setup-status{margin:2px 0 0;font-size:12px;color:#c9c6bf}
  .key-actions{display:flex;align-items:center;gap:10px;flex-wrap:wrap}.key-actions span{display:inline-flex;gap:5px;align-items:center;font-size:11px;color:#8e8e89}
  .step-actions{display:flex;align-items:center;justify-content:space-between;gap:12px;padding-top:22px;flex:none}
  .back-link{align-self:flex-start;border:0;background:transparent;color:#c9c6bf;padding:0 0 22px;font-size:13px}.back-link:hover{color:#fff;background:transparent}
  .integration-detail{padding:20px}.integration-detail p{font-size:13px;line-height:1.55;color:#b6b5b0}
  .connected-line{display:flex;align-items:center;gap:7px;color:#d7d5ce!important}.connected-line button{margin-left:auto}
  .integration-detail>.notice{border-top:1px solid #ffffff12;padding-top:12px}
  .quiet{background:transparent;border-color:transparent;color:#c9c6bf;font-size:14px}.quiet:hover{background:#ffffff0a;color:#fff}
  .onboarding button:disabled{cursor:default}

  @keyframes page-settle{from{opacity:0;transform:translateY(10px)}to{opacity:1;transform:translateY(0)}}
  @media(max-width:900px){.step-rail li:not(.active){font-size:0;gap:0}.step-rail li:not(.active) .step-number{font-size:15px}}
  @media(max-width:760px){.shortcut-row :global(.shortcut-keys){display:none}}
  @media(max-width:620px){.onboarding{margin:-48px -20px -12px;padding-inline:20px}.brand{font-size:17px}.choice-grid,.choice-grid.integrations{grid-template-columns:1fr}.choice-card{min-height:96px}.choice-card strong{margin-top:8px}.setup-row{flex-wrap:wrap}.setup-row>div:first-of-type{min-width:calc(100% - 38px)}.setup-row>button{margin-left:36px}.shortcut{width:100%;margin-left:36px}.shortcut input{flex:1;width:auto}.ink-backdrop{opacity:.55}}
  @media(prefers-reduced-motion:reduce){.setup-shell{animation:none}.choice-card,.step-rail li{transition:none}}
</style>
