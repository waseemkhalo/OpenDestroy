<script lang="ts">
  import { Check, ChevronDown, KeyRound, RefreshCw, Server, Unplug } from "@lucide/svelte";
  import { slide } from "svelte/transition";

  type IntegrationStatus = {
    composioKeyPresent: boolean;
    driveConnected: boolean;
    driveAccountLabel: string;
    giphyKeyPresent: boolean;
  };

  type Service = "drive" | "giphy" | "backend";

  let {
    embedded = false,
    integrations,
    native,
    connecting,
    composioKey,
    giphyKey,
    backendUrl,
    backendToken,
    onConnectDrive,
    onRefreshDrive,
    onDisconnectDrive,
    onSaveComposio,
    onSaveGiphy,
    onDisconnectGiphy,
    onComposioKey,
    onGiphyKey,
    onBackendUrl,
    onBackendToken,
    onConnectBackend,
  }: {
    embedded?: boolean;
    integrations: IntegrationStatus;
    native: boolean;
    connecting: boolean;
    composioKey: string;
    giphyKey: string;
    backendUrl: string;
    backendToken: string;
    onConnectDrive: () => void;
    onRefreshDrive: () => void;
    onDisconnectDrive: () => void;
    onSaveComposio: () => void;
    onSaveGiphy: () => void;
    onDisconnectGiphy: () => void;
    onComposioKey: (value: string) => void;
    onGiphyKey: (value: string) => void;
    onBackendUrl: (value: string) => void;
    onBackendToken: (value: string) => void;
    onConnectBackend: () => void;
  } = $props();

  let expandedService = $state<Service | null>(null);
  const prefersReducedMotion = typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  function toggle(service: Service) {
    expandedService = expandedService === service ? null : service;
  }

  function inputValue(event: Event): string {
    return (event.currentTarget as HTMLInputElement).value;
  }

  function invalidBackendUrl(value: string): boolean {
    if (!value.trim()) return false;
    try {
      const url = new URL(value);
      return url.protocol !== "http:" && url.protocol !== "https:";
    } catch {
      return true;
    }
  }
</script>

<div class:embedded class="settings-connections" aria-label="Connections">
  {#if !embedded}
    <header class="settings-subhead">
      <div>
        <span class="settings-eyebrow">Connections</span>
        <h2>Bring your tools into the conversation.</h2>
      </div>
      <p>Credentials stay in this installation’s Keychain.</p>
    </header>
  {:else}
    <p class="embedded-note"><span>Optional.</span> Dictation works without connecting anything.</p>
  {/if}

  <section class="service-list" aria-label="Services">
    <div class:is-open={expandedService === "drive"} class="service-card">
      <div class="service-row">
        <span class="service-icon service-icon-drive" aria-hidden="true"><img src="/provider-logos/google-drive.png" alt="" /></span>
        <div class="service-copy">
          <div class="service-title">
            <strong>Google Drive</strong>
            <span class:status-ready={integrations.driveConnected || integrations.composioKeyPresent} class="service-status">
              <span class="status-dot" aria-hidden="true"></span>
              {integrations.driveConnected ? "Connected" : integrations.composioKeyPresent ? "Ready" : "Not connected"}
            </span>
          </div>
          <small>{integrations.driveConnected ? integrations.driveAccountLabel || "Find files and insert their links." : "Find files and insert their links."}</small>
        </div>
        <button class="setup-action" type="button" aria-expanded={expandedService === "drive"} aria-controls="google-drive-panel" onclick={() => toggle("drive")}>
          {expandedService === "drive" ? "Close" : integrations.driveConnected ? "Manage" : "Set up"}
          <span class="chevron" class:turned={expandedService === "drive"}><ChevronDown size={16} aria-hidden="true" /></span>
        </button>
      </div>

      {#if expandedService === "drive"}
        <div class="inline-panel" id="google-drive-panel" transition:slide={{ duration: prefersReducedMotion ? 0 : 220 }}>
          <div class="panel-heading">
            <div>
              <span class="panel-kicker">Google Drive</span>
              <strong>Use your own Composio project.</strong>
              <p>Save a project key, then approve Google in your browser.</p>
            </div>
            <KeyRound size={19} aria-hidden="true" />
          </div>
          <label for="composio-project-key">Project API key</label>
          <input id="composio-project-key" class="settings-input" type="password" autocomplete="off" value={composioKey} placeholder={integrations.composioKeyPresent ? "Saved key stays hidden" : "Paste your project key"} aria-describedby="composio-key-help" oninput={(event) => onComposioKey(inputValue(event))} />
          <span class="sr-only" id="composio-key-help">Your key is stored in this installation’s Keychain.</span>
          <div class="panel-actions">
            <button class="primary-action" type="button" disabled={!native || !composioKey.trim()} onclick={onSaveComposio}>Save key</button>
            {#if integrations.driveConnected}
              <button class="secondary-action" type="button" disabled={!native} onclick={onDisconnectDrive}><Unplug size={14} aria-hidden="true" />Disconnect</button>
            {:else}
              <button class="secondary-action" type="button" disabled={!native || !integrations.composioKeyPresent || connecting} aria-busy={connecting} onclick={onConnectDrive}>{connecting ? "Connecting…" : "Connect Google Drive"}</button>
              <button class="tertiary-action" type="button" disabled={!native || !integrations.composioKeyPresent || connecting} aria-busy={connecting} onclick={onRefreshDrive}><RefreshCw size={14} aria-hidden="true" />Refresh</button>
            {/if}
          </div>
          {#if integrations.driveConnected}
            <p class="panel-state state-connected" role="status" aria-live="polite"><Check size={15} aria-hidden="true" /> {integrations.driveAccountLabel || "Google Drive is connected."}</p>
          {:else if integrations.composioKeyPresent}
            <p class="panel-state" role="status" aria-live="polite">After Google approval, return here and refresh the connection.</p>
          {:else}
            <p class="panel-state" role="status" aria-live="polite">No project key is saved yet. Save one to enable Google Drive.</p>
          {/if}
        </div>
      {/if}
    </div>

    <div class:is-open={expandedService === "giphy"} class="service-card">
      <div class="service-row">
        <span class="service-icon service-icon-giphy" aria-hidden="true"><img src="/provider-logos/giphy.png" alt="" /></span>
        <div class="service-copy">
          <div class="service-title">
            <strong>GIPHY</strong>
            <span class:status-ready={integrations.giphyKeyPresent} class="service-status">
              <span class="status-dot" aria-hidden="true"></span>
              {integrations.giphyKeyPresent ? "Key saved" : "Not connected"}
            </span>
          </div>
          <small>Find a GIF while you dictate.</small>
        </div>
        <button class="setup-action" type="button" aria-expanded={expandedService === "giphy"} aria-controls="giphy-panel" onclick={() => toggle("giphy")}>
          {expandedService === "giphy" ? "Close" : integrations.giphyKeyPresent ? "Manage" : "Set up"}
          <span class="chevron" class:turned={expandedService === "giphy"}><ChevronDown size={16} aria-hidden="true" /></span>
        </button>
      </div>

      {#if expandedService === "giphy"}
        <div class="inline-panel" id="giphy-panel" transition:slide={{ duration: prefersReducedMotion ? 0 : 220 }}>
          <div class="panel-heading">
            <div>
              <span class="panel-kicker">GIPHY</span>
              <strong>Use your own GIPHY key.</strong>
              <p>Search words go directly to GIPHY. Results keep their attribution.</p>
            </div>
            <img class="panel-provider-logo panel-provider-logo-giphy" src="/provider-logos/giphy.png" alt="" aria-hidden="true" />
          </div>
          <label for="giphy-api-key">GIPHY API key</label>
          <input id="giphy-api-key" class="settings-input" type="password" autocomplete="off" value={giphyKey} placeholder={integrations.giphyKeyPresent ? "Saved key stays hidden" : "Paste your API key"} aria-describedby="giphy-key-help" oninput={(event) => onGiphyKey(inputValue(event))} />
          <span class="sr-only" id="giphy-key-help">Your key is stored in this installation’s Keychain.</span>
          <div class="panel-actions">
            <button class="primary-action" type="button" disabled={!native || !giphyKey.trim()} onclick={onSaveGiphy}>Save key</button>
            <button class="secondary-action" type="button" disabled={!native || !integrations.giphyKeyPresent} onclick={onDisconnectGiphy}><Unplug size={14} aria-hidden="true" />Remove key</button>
          </div>
          <p class="panel-state" role="status" aria-live="polite">{integrations.giphyKeyPresent ? "A key is saved in Keychain." : "No key is saved. Search remains unavailable until you add one."}</p>
        </div>
      {/if}
    </div>

    <div class:is-open={expandedService === "backend"} class="service-card advanced-card">
      <button class="advanced-row" type="button" aria-expanded={expandedService === "backend"} aria-controls="advanced-backend-panel" onclick={() => toggle("backend")}>
        <span class="service-icon service-icon-backend" aria-hidden="true"><Server size={21} strokeWidth={1.6} /></span>
        <span class="service-copy">
          <span class="service-title"><strong>Advanced backend</strong><span class="service-status">{backendUrl.trim() ? "Endpoint set" : "Not configured"}</span></span>
          <small>Manual service URL and access token.</small>
        </span>
        <span class="setup-action setup-action-quiet">{expandedService === "backend" ? "Close" : "Configure"}<span class="chevron" class:turned={expandedService === "backend"}><ChevronDown size={16} aria-hidden="true" /></span></span>
      </button>
      {#if expandedService === "backend"}
        <div class="inline-panel" id="advanced-backend-panel" transition:slide={{ duration: prefersReducedMotion ? 0 : 220 }}>
          <div class="panel-heading">
            <div>
              <span class="panel-kicker">Advanced connection</span>
              <strong>Use an existing backend.</strong>
              <p>For self-hosted deployments. The connection status comes from the app.</p>
            </div>
            <Server size={19} aria-hidden="true" />
          </div>
          <label for="backend-url">Service URL</label>
          <input id="backend-url" class="settings-input" class:error-state={invalidBackendUrl(backendUrl)} type="url" inputmode="url" value={backendUrl} placeholder="http://127.0.0.1:8787" aria-invalid={invalidBackendUrl(backendUrl)} aria-describedby="backend-url-hint" oninput={(event) => onBackendUrl(inputValue(event))} />
          <span class="field-hint" id="backend-url-hint">Use the full http:// or https:// address for your service.</span>
          {#if invalidBackendUrl(backendUrl)}
            <span class="field-error" role="alert">Enter a complete http:// or https:// service URL.</span>
          {/if}
          <label for="backend-token">Access token</label>
          <input id="backend-token" class="settings-input" type="password" autocomplete="off" value={backendToken} placeholder="Paste your token" aria-describedby="backend-token-help" oninput={(event) => onBackendToken(inputValue(event))} />
          <span class="sr-only" id="backend-token-help">This token is stored in this installation’s Keychain.</span>
          <div class="panel-actions">
            <button class="primary-action" type="button" disabled={!native || connecting || !backendUrl.trim()} aria-busy={connecting} onclick={onConnectBackend}>{connecting ? "Connecting…" : "Connect backend"}</button>
          </div>
        </div>
      {/if}
    </div>
  </section>
</div>

<style>
  .settings-connections {
    --ivory: #f1eee7;
    --muted: #aaa9a2;
    --quiet: #777a72;
    --line: #ffffff14;
    --card: #111111;
    min-width: 0;
    padding: 10px 0 32px;
    color: var(--ivory);
  }

  .settings-subhead {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 28px;
    padding: 4px 2px 24px;
  }

  .settings-eyebrow,
  .panel-kicker {
    color: var(--muted);
    font-size: 10px;
    font-weight: 650;
    letter-spacing: .14em;
    text-transform: uppercase;
  }

  .settings-subhead h2 {
    max-width: 560px;
    margin: 8px 0 0;
    color: var(--ivory);
    font-family: DestroyEditorial, Georgia, serif;
    font-size: clamp(26px, 4vw, 40px);
    font-weight: 500;
    letter-spacing: -.02em;
    line-height: 1;
  }

  .settings-subhead p,
  .embedded-note {
    max-width: 260px;
    margin: 0 0 3px;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.5;
  }

  .embedded-note {
    max-width: none;
    padding: 0 2px 18px;
    border-bottom: 1px solid var(--line);
  }

  .embedded-note span {
    color: var(--ivory);
  }

  .service-list {
    display: grid;
    gap: 14px;
  }

  .service-card {
    overflow: hidden;
    border: 1px solid var(--line);
    border-radius: 20px;
    background: var(--card);
    box-shadow: 0 14px 35px #00000026;
    transition: border-color 180ms ease, box-shadow 180ms ease;
  }

  .service-card.is-open {
    border-color: #ffffff24;
    box-shadow: 0 18px 42px #0000003d;
  }

  .service-row,
  .advanced-row {
    min-height: 102px;
    display: grid;
    grid-template-columns: 52px minmax(0, 1fr) auto;
    align-items: center;
    gap: 18px;
    padding: 18px 22px;
  }

  .service-icon {
    width: 52px;
    height: 52px;
    display: grid;
    place-items: center;
    border: 1px solid #ffffff17;
    border-radius: 15px;
  }

  .service-icon-drive {
    color: #111;
    border-color: #f1eee7;
    background: var(--ivory);
  }

  .service-icon-giphy {
    color: var(--ivory);
    background: #070707;
  }

  .service-icon-backend {
    color: var(--muted);
    background: #191919;
  }

  .service-icon img {
    display: block;
    width: 28px;
    height: 28px;
    object-fit: contain;
  }

  .panel-provider-logo {
    display: block;
    width: 28px;
    height: 28px;
    object-fit: contain;
  }

  .panel-provider-logo-giphy {
    width: 34px;
    height: 34px;
  }

  .service-copy {
    min-width: 0;
  }

  .service-title {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
  }

  .service-copy strong {
    display: block;
    color: var(--ivory);
    font-family: DestroyEditorial, Georgia, serif;
    font-size: 25px;
    font-weight: 500;
    line-height: 1.05;
  }

  .service-copy small {
    display: block;
    margin-top: 7px;
    overflow: hidden;
    color: var(--muted);
    font-size: 13px;
    line-height: 1.4;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .service-status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--quiet);
    font-size: 10px;
    font-weight: 560;
    letter-spacing: .04em;
    white-space: nowrap;
  }

  .status-ready {
    color: #d8d3c9;
  }

  .status-dot {
    width: 5px;
    height: 5px;
    display: inline-block;
    border-radius: 50%;
    background: #555852;
  }

  .status-ready .status-dot {
    background: #d8d3c9;
  }

  button {
    font: inherit;
  }

  .setup-action,
  .primary-action,
  .secondary-action,
  .tertiary-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-height: 42px;
    border: 1px solid transparent;
    border-radius: 11px;
    padding: 0 18px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: -.01em;
    transition: background 160ms ease, border-color 160ms ease, color 160ms ease, opacity 160ms ease, transform 160ms ease;
  }

  .setup-action {
    min-width: 116px;
    border-color: #f1eee7;
    background: var(--ivory);
    color: #111;
  }

  .setup-action:hover,
  .primary-action:hover {
    background: #fffaf1;
    transform: translateY(-1px);
  }

  .setup-action-quiet {
    min-width: 112px;
    border-color: #ffffff24;
    background: #1a1a1a;
    color: var(--ivory);
  }

  .setup-action-quiet:hover {
    border-color: #ffffff40;
    background: #222;
  }

  .chevron {
    display: inline-flex;
    transition: transform 180ms ease;
  }

  .chevron.turned {
    transform: rotate(180deg);
  }

  .inline-panel {
    margin: 0 22px 20px;
    padding: 22px 22px 2px;
    border-top: 1px solid var(--line);
  }

  .panel-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 18px;
    margin-bottom: 20px;
    color: var(--muted);
  }

  .panel-heading :global(svg) {
    flex: none;
    margin-top: 2px;
    color: var(--quiet);
  }

  .panel-heading strong {
    display: block;
    margin-top: 5px;
    color: var(--ivory);
    font-family: DestroyEditorial, Georgia, serif;
    font-size: 22px;
    font-weight: 500;
  }

  .panel-heading p {
    max-width: 520px;
    margin: 6px 0 0;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.5;
  }

  .inline-panel > label {
    display: block;
    margin: 0 0 8px;
    color: #d5d1c8;
    font-size: 12px;
    font-weight: 560;
  }

  .settings-input {
    width: min(100%, 600px);
    min-height: 48px;
    display: block;
    border: 1px solid #ffffff18;
    border-radius: 13px;
    background: #191919;
    color: var(--ivory);
    padding: 12px 14px;
    font-size: 13px;
    transition: border-color 160ms ease, box-shadow 160ms ease, background 160ms ease;
  }

  .settings-input::placeholder {
    color: #777a72;
  }

  .settings-input:hover {
    border-color: #ffffff2b;
  }

  .settings-input:focus-visible,
  .setup-action:focus-visible,
  .advanced-row:focus-visible,
  .panel-actions button:focus-visible {
    border-color: #f1eee7;
    outline: 2px solid #f1eee7;
    outline-offset: 3px;
  }

  .settings-input:invalid:not(:placeholder-shown),
  .settings-input.error-state {
    border-color: #c66b62;
    box-shadow: 0 0 0 1px #c66b6240;
  }

  .field-hint,
  .field-error {
    display: block;
    margin: 7px 0 17px;
    color: var(--quiet);
    font-size: 10px;
    line-height: 1.45;
  }

  .field-error {
    margin-top: -10px;
    color: #e69b93;
  }

  .panel-actions {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 9px;
    margin-top: 16px;
  }

  .primary-action {
    border-color: var(--ivory);
    background: var(--ivory);
    color: #111;
  }

  .secondary-action,
  .tertiary-action {
    border-color: #ffffff22;
    background: #1a1a1a;
    color: var(--ivory);
  }

  .tertiary-action {
    border-color: transparent;
    background: transparent;
    color: var(--muted);
  }

  .secondary-action:hover,
  .tertiary-action:hover {
    border-color: #ffffff3a;
    background: #222;
    color: var(--ivory);
  }

  .panel-state {
    display: flex;
    align-items: flex-start;
    gap: 7px;
    margin: 17px 0 0;
    color: var(--muted);
    font-size: 11px;
    line-height: 1.5;
  }

  .state-connected {
    color: #d8d3c9;
  }

  .advanced-row {
    width: 100%;
    border: 0;
    background: transparent;
    color: var(--ivory);
    text-align: left;
  }

  .advanced-row:hover .setup-action-quiet {
    border-color: #ffffff3a;
    background: #222;
  }

  .advanced-row .service-copy strong {
    font-family: inherit;
    font-size: 14px;
    font-weight: 620;
  }

  .advanced-row .service-title {
    gap: 12px;
  }

  .advanced-row .service-status {
    font-size: 10px;
  }

  .advanced-card .inline-panel {
    padding-top: 22px;
  }

  .sr-only {
    width: 1px;
    height: 1px;
    position: absolute;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
  }

  button:disabled,
  input:disabled {
    cursor: not-allowed;
    opacity: .42;
  }

  button:disabled:hover {
    transform: none;
  }

  .embedded .embedded-note{font-size:21px;border:0;padding:0;margin:14px 0 22px}
  .embedded .service-list{gap:20px}
  .embedded .service-row,.embedded .advanced-row{min-height:124px;padding:24px 28px;grid-template-columns:76px minmax(0,1fr) auto;gap:28px}
  .embedded .service-icon{width:76px;height:76px;border-radius:17px}
  .embedded .service-icon img{width:40px;height:40px}
  .embedded .service-copy strong{font-size:32px}
  .embedded .service-copy small{font-size:18px}
  .embedded .service-status{font-size:12px}
  .embedded .setup-action,.embedded .primary-action,.embedded .secondary-action,.embedded .tertiary-action{min-height:48px;font-size:17px;font-weight:500}
  .embedded .inline-panel{margin:0 46px 28px;padding-top:26px}
  .embedded .panel-heading strong{font-size:30px}
  .embedded .panel-heading p,.embedded .inline-panel>label,.embedded .panel-state{font-size:17px}
  .embedded .settings-input{font-size:17px;max-width:650px;width:100%}
  .embedded .field-hint,.embedded .field-error{font-size:14px}
  .embedded .panel-kicker{display:none}
  .embedded .advanced-row .service-copy strong{font-size:20px}
  .embedded .advanced-row .service-copy small{font-size:15px}
  @media (max-width: 900px) {
    .embedded .service-row,.embedded .advanced-row{grid-template-columns:52px minmax(0,1fr);gap:16px;padding:20px}
    .embedded .service-icon{width:52px;height:52px}.embedded .service-copy strong{font-size:28px}.embedded .service-copy small{font-size:16px;white-space:normal}
    .embedded .setup-action{grid-column:2;justify-self:start}.embedded .inline-panel{margin:0 20px 22px}.embedded .embedded-note{font-size:17px}
  }
  @media (max-width: 700px) {
    .settings-subhead {
      display: block;
      padding-bottom: 18px;
    }

    .settings-subhead p {
      max-width: none;
      margin-top: 12px;
    }

    .service-row,
    .advanced-row {
      grid-template-columns: 46px minmax(0, 1fr);
      gap: 14px;
      padding: 16px;
    }

    .service-icon {
      width: 46px;
      height: 46px;
      border-radius: 13px;
    }

    .service-copy strong {
      font-size: 22px;
    }

    .service-copy small {
      white-space: normal;
    }

    .setup-action {
      grid-column: 2;
      justify-self: start;
      min-height: 38px;
      margin-top: 2px;
      padding: 0 14px;
    }

    .inline-panel {
      margin: 0 16px 16px;
      padding: 18px 0 2px;
    }

    .advanced-row .setup-action-quiet {
      grid-column: 2;
      justify-self: start;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .service-card,
    .setup-action,
    .primary-action,
    .secondary-action,
    .tertiary-action,
    .settings-input,
    .chevron {
      transition: none;
    }
  }
</style>
