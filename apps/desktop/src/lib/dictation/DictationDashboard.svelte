<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import {
    BookOpenText,
    Check,
    ChevronLeft,
    Clipboard,
    Clock3,
    Flame,
    Gauge,
    Languages,
    Plus,
    Quote,
    Save,
    ShieldCheck,
    SlidersHorizontal,
    Sparkles,
    Trash2,
    Type,
    WandSparkles,
  } from "@lucide/svelte";
  import {
    clearLastDictation,
    addPersonalDictationTerm,
    dictationAverageWpm,
    dictationDayStreak,
    dictationTimeSavedMs,
    readDictationSnapshot,
    syncDictationUsage,
    loadPersonalDictationTerms,
    removePersonalDictationTerm,
    DICTATION_UPDATED_EVENT,
    type DictationSnapshot,
  } from "./dictationState";
  import {
    currentDictationPreferences,
    deleteDictationSnippet,
    learnDictationStyle,
    loadDictationPreferences,
    loadDictationSnippets,
    saveDictationPreferences,
    saveDictationSnippet,
    type DictationPreferences,
    type DictationSnippet,
  } from "./dictationComposer";

  let {
    teamId,
    permissionRole,
  }: {
    teamId: string | null;
    permissionRole: string | null;
  } = $props();

  let view = $state<"dashboard" | "writing" | "snippets" | "vocabulary">("dashboard");

  let snapshot = $state<DictationSnapshot>({
    words: 0,
    speakingMs: 0,
    days: [],
    lastText: "",
  });
  let copied = $state(false);
  let copyError = $state("");
  let customTerm = $state("");
  let personalTerms = $state<string[]>([]);
  let termsBusy = $state(false);
  let termsError = $state("");
  let preferences = $state<DictationPreferences>(currentDictationPreferences(null));
  let preferencesBusy = $state(false);
  let preferencesError = $state("");
  let styleSample = $state("");
  let styleLearning = $state(false);
  let snippets = $state<DictationSnippet[]>([]);
  let snippetsBusy = $state(false);
  let snippetsError = $state("");
  let snippetId = $state<string | undefined>(undefined);
  let snippetTitle = $state("");
  let snippetTrigger = $state("");
  let snippetBody = $state("");


  // Metrics are stored per team, so without a team id there is nothing to read
  // — not a zeroed history. Rendering 0 there tells the rep their streak and
  // word count were wiped when the account simply has not resolved yet.
  const metricsKnown = $derived(teamId !== null);
  const averageWpm = $derived(dictationAverageWpm(snapshot));
  const streak = $derived(dictationDayStreak(snapshot));
  const savedMs = $derived(dictationTimeSavedMs(snapshot));
  const viewTitle = $derived({
    dashboard: "Dictation",
    writing: "Writing",
    snippets: "Voice shortcuts",
    vocabulary: "Vocabulary",
  }[view]);
  const viewSubtitle = $derived({
    dashboard: "Hold ⌘ ` · speak · release",
    writing: "How speech becomes writing",
    snippets: "Say a phrase, insert the whole message",
    vocabulary: "Names and terms Destroy should recognize",
  }[view]);

  $effect(() => {
    const requested = teamId;
    snapshot = readDictationSnapshot(requested);
    // The cache renders immediately; the rep's durable counters are the truth.
    // A wiped or reinstalled Mac gets its history back here, and anything this
    // machine counted while offline is reported in the same exchange.
    void syncDictationUsage(requested)
      .then((synced) => {
        if (requested === teamId) snapshot = synced;
      })
      .catch(() => {
        // Offline: the cached counters above are still shown.
      });
    // Clear every customer-authored value synchronously at a team boundary.
    // Async guards alone are insufficient because stale content remains
    // visible (and saveable) while the next team's requests are in flight.
    personalTerms = [];
    customTerm = "";
    termsError = "";
    preferences = currentDictationPreferences(requested);
    preferencesError = "";
    styleSample = "";
    styleLearning = false;
    snippets = [];
    snippetsError = "";
    clearSnippetForm();
    termsBusy = requested !== null;
    preferencesBusy = requested !== null;
    snippetsBusy = requested !== null;
    if (!requested) return;
    void loadPersonalDictationTerms(requested)
      .then((terms) => {
        if (requested === teamId) personalTerms = terms;
      })
      .catch(() => {
        if (requested === teamId) termsError = "Could not load your vocabulary.";
      })
      .finally(() => {
        if (requested === teamId) termsBusy = false;
      });
    void loadDictationPreferences(requested)
      .then((profile) => {
        if (requested === teamId) preferences = profile;
      })
      .catch(() => {
        if (requested === teamId) preferencesError = "Could not load dictation settings.";
      })
      .finally(() => {
        if (requested === teamId) preferencesBusy = false;
      });
    void loadDictationSnippets()
      .then((value) => {
        if (requested === teamId) snippets = value;
      })
      .catch(() => {
        if (requested === teamId) snippetsError = "Could not load voice shortcuts.";
      })
      .finally(() => {
        if (requested === teamId) snippetsBusy = false;
      });
  });

  $effect(() => {
    const refresh = () => { snapshot = readDictationSnapshot(teamId); };
    window.addEventListener(DICTATION_UPDATED_EVENT, refresh);
    return () => window.removeEventListener(DICTATION_UPDATED_EVENT, refresh);
  });

  function durationLabel(ms: number): string {
    const minutes = Math.floor(ms / 60_000);
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    return `${hours}h ${minutes % 60}m`;
  }

  async function copyLast() {
    if (!snapshot.lastText) return;
    try {
      await invoke("destroy_dictation_copy", { text: snapshot.lastText });
      copied = true;
      copyError = "";
      window.setTimeout(() => { copied = false; }, 1200);
    } catch (error) {
      copied = false;
      copyError = error instanceof Error && error.message
        ? error.message
        : "Could not copy the dictation. Try again.";
    }
  }

  function describeError(error: unknown): string {
    const text = error instanceof Error ? error.message : String(error ?? "");
    try {
      const parsed = JSON.parse(text) as { error?: string };
      if (parsed?.error) return parsed.error;
    } catch {
      // Not a JSON API error — fall through to the raw text.
    }
    return text.trim() || "Could not save that term.";
  }

  async function addTerm() {
    const requested = teamId;
    const term = customTerm.trim();
    if (!term || termsBusy || !requested) return;
    termsBusy = true;
    termsError = "";
    try {
      const next = await addPersonalDictationTerm(requested, term);
      if (requested === teamId) {
        personalTerms = next;
        customTerm = "";
      }
    } catch (error) {
      if (requested === teamId) termsError = describeError(error);
    } finally {
      if (requested === teamId) termsBusy = false;
    }
  }

  async function removeTerm(term: string) {
    const requested = teamId;
    if (termsBusy || !requested) return;
    termsBusy = true;
    termsError = "";
    try {
      const next = await removePersonalDictationTerm(requested, term);
      if (requested === teamId) personalTerms = next;
    } catch (error) {
      if (requested === teamId) termsError = describeError(error);
    } finally {
      if (requested === teamId) termsBusy = false;
    }
  }

  async function updatePreferences(next: DictationPreferences) {
    const requested = teamId;
    if (preferencesBusy || !requested) return;
    preferencesBusy = true;
    preferencesError = "";
    const before = preferences;
    preferences = next;
    try {
      const saved = await saveDictationPreferences(next, requested);
      if (requested === teamId) preferences = saved;
    } catch (error) {
      if (requested === teamId) {
        preferences = before;
        preferencesError = describeError(error);
      }
    } finally {
      if (requested === teamId) preferencesBusy = false;
    }
  }

  function togglePreference(
    key: "self_correction" | "remove_fillers" | "app_formatting" | "selected_text_editing",
  ) {
    void updatePreferences({ ...preferences, [key]: !preferences[key] });
  }

  async function learnStyle() {
    const requested = teamId;
    if (styleLearning || styleSample.trim().length < 40 || !requested) return;
    styleLearning = true;
    preferencesError = "";
    try {
      const styleNote = await learnDictationStyle(styleSample, requested);
      if (requested === teamId) {
        preferences = { ...preferences, style_note: styleNote };
        styleSample = "";
      }
    } catch (error) {
      if (requested === teamId) preferencesError = describeError(error);
    } finally {
      if (requested === teamId) styleLearning = false;
    }
  }

  function clearSnippetForm() {
    snippetId = undefined;
    snippetTitle = "";
    snippetTrigger = "";
    snippetBody = "";
  }

  function editSnippet(snippet: DictationSnippet) {
    snippetId = snippet.id;
    snippetTitle = snippet.title;
    snippetTrigger = snippet.trigger;
    snippetBody = snippet.body;
  }

  async function saveSnippet() {
    const requested = teamId;
    if (snippetsBusy || !requested || !snippetTitle.trim() || !snippetTrigger.trim() || !snippetBody.trim()) return;
    snippetsBusy = true;
    snippetsError = "";
    try {
      const saved = await saveDictationSnippet({
        id: snippetId,
        title: snippetTitle,
        trigger: snippetTrigger,
        body: snippetBody,
      });
      if (requested === teamId) {
        snippets = [saved, ...snippets.filter((snippet) => snippet.id !== saved.id)];
        clearSnippetForm();
      }
    } catch (error) {
      if (requested === teamId) snippetsError = describeError(error);
    } finally {
      if (requested === teamId) snippetsBusy = false;
    }
  }

  async function removeSnippet(id: string) {
    const requested = teamId;
    if (snippetsBusy || !requested) return;
    snippetsBusy = true;
    snippetsError = "";
    try {
      await deleteDictationSnippet(id);
      if (requested === teamId) {
        snippets = snippets.filter((snippet) => snippet.id !== id);
        if (snippetId === id) clearSnippetForm();
      }
    } catch (error) {
      if (requested === teamId) snippetsError = describeError(error);
    } finally {
      if (requested === teamId) snippetsBusy = false;
    }
  }
</script>

<section class="dictation-root" aria-label="Dictation">
  <header class="dictation-head">
    {#if view !== "dashboard"}
      <button class="icon-button" type="button" aria-label="Back to Dictation" onclick={() => { view = "dashboard"; }}>
        <ChevronLeft size={17} strokeWidth={1.8} />
      </button>
    {:else}
      <span aria-hidden="true"></span>
    {/if}
    <div>
      <span>{viewTitle}</span>
      <small>{viewSubtitle}</small>
    </div>
    {#if view === "dashboard"}
      <button class="icon-button" type="button" aria-label="Open vocabulary" title="Vocabulary" onclick={() => { view = "vocabulary"; }}>
        <BookOpenText size={16} strokeWidth={1.8} />
      </button>
    {/if}
  </header>

  {#if view === "dashboard"}
    <div class="metric-grid" aria-label="Dictation activity">
      <article><Type size={15} /><div><strong>{metricsKnown ? snapshot.words.toLocaleString() : "—"}</strong><span>words</span></div></article>
      <article><Clock3 size={15} /><div><strong>{metricsKnown ? durationLabel(savedMs) : "—"}</strong><span>saved</span></div></article>
      <article><Flame size={15} /><div><strong>{metricsKnown ? streak : "—"}</strong><span>day streak</span></div></article>
      <article><Gauge size={15} /><div><strong>{metricsKnown ? averageWpm : "—"}</strong><span>wpm</span></div></article>
    </div>

    <section class="quick-copy">
      <header>
        <div><Clipboard size={15} /><span>Quick Copy</span></div>
        <div class="quick-actions">
          {#if snapshot.lastText}
            <button class="icon-button" type="button" aria-label="Clear last dictation" title="Clear" onclick={() => clearLastDictation(teamId)}>
              <Trash2 size={14} strokeWidth={1.8} />
            </button>
            <button class="icon-button primary-icon" type="button" aria-label="Copy last dictation" title="Copy" onclick={() => void copyLast()}>
              {#if copied}<Check size={14} strokeWidth={2} />{:else}<Clipboard size={14} strokeWidth={1.8} />{/if}
            </button>
          {/if}
        </div>
      </header>
      <p class:quick-copy--empty={!snapshot.lastText}>
        {snapshot.lastText || "Your last dictation will stay here until you sign out."}
      </p>
      {#if copyError}<small class="quick-copy-error" role="alert">{copyError}</small>{/if}
    </section>

    <div class="control-list dashboard-controls">
      <button type="button" onclick={() => { view = "writing"; }}>
        <SlidersHorizontal size={15} /><span><strong>Writing</strong><small>Corrections, cleanup, language, and your style</small></span><b>Open</b>
      </button>
      <button type="button" onclick={() => { view = "snippets"; }}>
        <Quote size={15} /><span><strong>Voice shortcuts</strong><small>{snippets.length ? `${snippets.length} saved` : "Turn a phrase into a complete message"}</small></span><b>Open</b>
      </button>
      <button type="button" onclick={() => { view = "vocabulary"; }}>
        <BookOpenText size={15} /><span><strong>Vocabulary</strong><small>Names and specialist terms</small></span><b>Open</b>
      </button>
    </div>
    <details class="command-guide">
      <summary><Sparkles size={13} /> What can I say?</summary>
      <div>
        <span>“Tuesday — actually Wednesday”</span>
        <span>“Add a laughing GIF”</span>
        <span>“Add a laughing emoji”</span>
        <span>“Record a voice note”</span>
        <span>“Undo that”</span>
        <span>Select text · “make this warmer”</span>
        <span>“Destroy, draft a follow-up…”</span>
      </div>
      <small>Destroy actions always open a preview before anything is sent or changed.</small>
    </details>
  {:else if view === "writing"}
    <div class="preference-list" aria-busy={preferencesBusy}>
      <button type="button" role="switch" aria-checked={preferences.self_correction} onclick={() => togglePreference("self_correction")}>
        <span><strong>Natural corrections</strong><small>“Tuesday — actually Wednesday” becomes “Wednesday.”</small></span>
        <i class:on={preferences.self_correction}></i>
      </button>
      <button type="button" role="switch" aria-checked={preferences.remove_fillers} onclick={() => togglePreference("remove_fillers")}>
        <span><strong>Clean up speech</strong><small>Remove filler and accidental repetition, not uncertainty.</small></span>
        <i class:on={preferences.remove_fillers}></i>
      </button>
      <button type="button" role="switch" aria-checked={preferences.app_formatting} onclick={() => togglePreference("app_formatting")}>
        <span><strong>Match the app</strong><small>Messages stay brief; email and documents get paragraphs.</small></span>
        <i class:on={preferences.app_formatting}></i>
      </button>
      <button type="button" role="switch" aria-checked={preferences.selected_text_editing} onclick={() => togglePreference("selected_text_editing")}>
        <span><strong>Edit selected text</strong><small>Select text, hold the shortcut, then say “make this warmer.”</small></span>
        <i class:on={preferences.selected_text_editing}></i>
      </button>
    </div>

    <label class="field-block language-field">
      <span><Languages size={14} /> Language</span>
      <select value={preferences.language} onchange={(event) => void updatePreferences({ ...preferences, language: event.currentTarget.value })}>
        <option value="auto">Detect automatically</option>
        <option value="en">English</option>
        <option value="es">Spanish</option>
        <option value="fr">French</option>
        <option value="de">German</option>
        <option value="pt">Portuguese</option>
        <option value="it">Italian</option>
        <option value="nl">Dutch</option>
        <option value="ja">Japanese</option>
        <option value="zh">Chinese</option>
        <option value="ko">Korean</option>
        <option value="hi">Hindi</option>
        <option value="ar">Arabic</option>
      </select>
    </label>

    <section class="style-block">
      <header><span><WandSparkles size={14} /> Your writing style</span><small>Only the summary is saved</small></header>
      <textarea
        value={preferences.style_note}
        oninput={(event) => { preferences = { ...preferences, style_note: event.currentTarget.value }; }}
        placeholder="Warm, concise, direct. Short paragraphs. No em dash."
        aria-label="Writing style"
      ></textarea>
      <button class="save-button" type="button" disabled={preferencesBusy} onclick={() => void updatePreferences(preferences)}><Save size={13} /> Save style</button>
      <details>
        <summary>Learn from a writing sample</summary>
        <p>Paste something that already sounds like you. Destroy extracts the style and discards the sample.</p>
        <textarea bind:value={styleSample} placeholder="Paste at least a few sentences…" aria-label="Writing sample"></textarea>
        <button class="save-button" type="button" disabled={styleLearning || styleSample.trim().length < 40} onclick={() => void learnStyle()}><Sparkles size={13} /> {styleLearning ? "Learning…" : "Learn this style"}</button>
      </details>
    </section>
    {#if preferencesError}<p class="term-error" role="alert">{preferencesError}</p>{/if}
    <p class="privacy-note"><ShieldCheck size={13} /> Preferences are encrypted for your account. Selections and writing samples are processed for the requested edit and are not saved.</p>
  {:else if view === "snippets"}
    <form class="snippet-form" onsubmit={(event) => { event.preventDefault(); void saveSnippet(); }}>
      <div class="snippet-two">
        <label><span>Name</span><input bind:value={snippetTitle} placeholder="Booking link" /></label>
        <label><span>Say</span><input bind:value={snippetTrigger} placeholder="my booking link" /></label>
      </div>
      <label><span>Insert</span><textarea bind:value={snippetBody} placeholder="Pick a time here: https://…"></textarea></label>
      <div class="snippet-actions">
        <small>User-defined variables: <code>{"{{my_company}}"}</code> <code>{"{{product_icp}}"}</code></small>
        <div>
          {#if snippetId}<button type="button" class="quiet-button" onclick={clearSnippetForm}>Cancel</button>{/if}
          <button class="save-button" type="submit" disabled={snippetsBusy || !snippetTitle.trim() || !snippetTrigger.trim() || !snippetBody.trim()}><Save size={13} /> {snippetId ? "Update" : "Add"}</button>
        </div>
      </div>
    </form>
    {#if snippetsError}<p class="term-error" role="alert">{snippetsError}</p>{/if}
    <p class="privacy-note"><ShieldCheck size={13} /> Shortcut text is encrypted. Define variables in Connection settings.</p>
    <div class="snippet-list">
      {#if snippets.length === 0}
        <p class="empty-note">Create one shortcut, then say “insert” followed by its phrase.</p>
      {:else}
        {#each snippets as snippet (snippet.id)}
          <div class="snippet-row">
            <button type="button" class="snippet-edit" onclick={() => editSnippet(snippet)}>
              <Quote size={14} /><span><strong>{snippet.title}</strong><small>“{snippet.trigger}”</small></span>
            </button>
            <button class="icon-button" type="button" aria-label={`Delete ${snippet.title}`} onclick={(event) => { event.stopPropagation(); void removeSnippet(snippet.id); }}><Trash2 size={13} /></button>
          </div>
        {/each}
      {/if}
    </div>
  {:else}
      <form class="term-form" onsubmit={(event) => { event.preventDefault(); void addTerm(); }}>
        <input bind:value={customTerm} placeholder="Add a name or term" aria-label="Personal vocabulary term" />
        <button class="icon-button primary-icon" type="submit" aria-label="Add vocabulary term" disabled={!customTerm.trim() || termsBusy}><Plus size={15} /></button>
      </form>
      {#if termsError}
        <p class="term-error" role="alert">{termsError}</p>
      {/if}
      <p class="privacy-note"><ShieldCheck size={13} /> Encrypted and saved to your account — only you and your transcriptions use these.</p>
      <div class="term-list">
        {#if personalTerms.length === 0}
          <p class="empty-note">Add names Destroy commonly misses — customers, accounts, product terms.</p>
        {:else}
          {#each personalTerms as term (term)}
            <div><span>{term}</span><button class="icon-button" type="button" aria-label={`Remove ${term}`} onclick={() => void removeTerm(term)}><Trash2 size={13} /></button></div>
          {/each}
        {/if}
      </div>
  {/if}
</section>

<style>
  button, input { font: inherit; }
  button { cursor: pointer; }
  .dictation-root { height: 100%; min-height: 0; overflow: auto; color: rgba(255,255,255,.94); scrollbar-width: none; padding: 2px 2px 14px; }
  .dictation-root::-webkit-scrollbar { display: none; }
  .dictation-head { display: grid; grid-template-columns: 30px 1fr 30px; gap: 8px; align-items: center; margin-bottom: 12px; }
  .dictation-head > div { display: flex; flex-direction: column; min-width: 0; }
  .dictation-head span { font-size: 15px; font-weight: 650; letter-spacing: -.02em; }
  .dictation-head small { color: rgba(255,255,255,.42); font-size: 9px; margin-top: 1px; }
  .icon-button { display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; padding: 0; border: 1px solid rgba(255,255,255,.08); border-radius: 8px; background: rgba(255,255,255,.045); color: rgba(255,255,255,.68); }
  .icon-button:hover, .icon-button:focus-visible { background: rgba(255,255,255,.1); color: #fff; outline: none; }
  .primary-icon { background: rgba(255,255,255,.92); color: #090909; }
  .primary-icon:hover { background: #fff; color: #000; }
  .metric-grid { display: grid; grid-template-columns: repeat(4, minmax(0,1fr)); gap: 6px; }
  .metric-grid article { display: flex; align-items: center; gap: 7px; min-width: 0; padding: 10px; border: 1px solid rgba(255,255,255,.07); border-radius: 11px; background: rgba(255,255,255,.035); color: rgba(255,255,255,.42); }
  .metric-grid article div { display: flex; flex-direction: column; min-width: 0; }
  .metric-grid strong { color: #fff; font-size: 14px; line-height: 1.1; }
  .metric-grid span { font-size: 8px; margin-top: 2px; }
  .quick-copy { margin-top: 8px; padding: 11px; border: 1px solid rgba(255,255,255,.08); border-radius: 12px; background: rgba(255,255,255,.045); }
  .quick-copy header { display: flex; align-items: center; justify-content: space-between; }
  .quick-copy header > div:first-child { display: flex; align-items: center; gap: 6px; color: rgba(255,255,255,.7); font-size: 10px; font-weight: 600; }
  .quick-actions { display: flex; gap: 5px; }
  .quick-copy p { margin: 9px 0 0; color: rgba(255,255,255,.88); font-size: 11px; line-height: 1.45; max-height: 64px; overflow: auto; white-space: pre-wrap; }
  .quick-copy p.quick-copy--empty { color: rgba(255,255,255,.35); }
  .quick-copy-error { display: block; margin-top: 7px; color: rgba(255, 154, 154, .9); font-size: 10px; }
  .control-list { margin-top: 8px; border: 1px solid rgba(255,255,255,.07); border-radius: 12px; overflow: hidden; }
  .control-list b { justify-self: end; color: rgba(255,255,255,.76); font-weight: 500; text-align: right; }
  .dashboard-controls > button { display: grid; grid-template-columns: 18px 1fr auto; gap: 8px; align-items: center; width: 100%; min-height: 45px; padding: 7px 10px; border: 0; background: transparent; color: rgba(255,255,255,.45); text-align: left; }
  .dashboard-controls > button + button { border-top: 1px solid rgba(255,255,255,.06); }
  .dashboard-controls > button:hover,
  .dashboard-controls > button:focus-visible { background: rgba(255,255,255,.055); color: #fff; outline: none; }
  .dashboard-controls span { display: flex; flex-direction: column; min-width: 0; }
  .dashboard-controls strong { color: rgba(255,255,255,.84); font-size: 10px; }
  .dashboard-controls small { margin-top: 2px; overflow: hidden; color: rgba(255,255,255,.36); font-size: 8px; text-overflow: ellipsis; white-space: nowrap; }
  .dashboard-controls b { color: rgba(255,255,255,.48); font-size: 8px; }
  .command-guide { margin-top: 8px; padding: 9px 10px; border: 1px solid rgba(255,255,255,.07); border-radius: 12px; background: rgba(255,255,255,.025); }
  .command-guide summary { display: flex; align-items: center; gap: 6px; color: rgba(255,255,255,.7); font-size: 9px; font-weight: 600; cursor: pointer; list-style: none; }
  .command-guide summary::-webkit-details-marker { display: none; }
  .command-guide > div { display: flex; flex-wrap: wrap; gap: 5px; margin-top: 9px; }
  .command-guide > div span { padding: 4px 6px; border: 1px solid rgba(255,255,255,.065); border-radius: 7px; background: rgba(255,255,255,.035); color: rgba(255,255,255,.54); font-size: 8px; }
  .command-guide > small { display: block; margin-top: 8px; color: rgba(255,255,255,.3); font-size: 7px; line-height: 1.45; }
  .preference-list { border: 1px solid rgba(255,255,255,.075); border-radius: 12px; overflow: hidden; }
  .preference-list button { display: grid; grid-template-columns: 1fr 29px; gap: 10px; align-items: center; width: 100%; min-height: 48px; padding: 8px 10px; border: 0; background: rgba(255,255,255,.028); color: #fff; text-align: left; }
  .preference-list button + button { border-top: 1px solid rgba(255,255,255,.06); }
  .preference-list button:hover, .preference-list button:focus-visible { background: rgba(255,255,255,.055); outline: none; }
  .preference-list span { display: flex; flex-direction: column; }
  .preference-list strong { font-size: 10px; font-weight: 600; }
  .preference-list small { margin-top: 2px; color: rgba(255,255,255,.38); font-size: 8px; line-height: 1.3; }
  .preference-list i { position: relative; display: block; width: 27px; height: 16px; border: 1px solid rgba(255,255,255,.16); border-radius: 999px; background: rgba(255,255,255,.08); transition: background 140ms ease, border-color 140ms ease; }
  .preference-list i::after { content: ""; position: absolute; top: 2px; left: 2px; width: 10px; height: 10px; border-radius: 50%; background: rgba(255,255,255,.64); transition: transform 140ms ease; }
  .preference-list i.on { border-color: rgba(59,151,255,.72); background: rgba(28,119,255,.58); }
  .preference-list i.on::after { transform: translateX(11px); background: #fff; }
  .field-block, .style-block, .snippet-form { display: block; margin-top: 8px; padding: 10px; border: 1px solid rgba(255,255,255,.075); border-radius: 12px; background: rgba(255,255,255,.03); }
  .field-block > span, .style-block header > span, .snippet-form label > span { display: flex; align-items: center; gap: 5px; color: rgba(255,255,255,.72); font-size: 9px; font-weight: 600; }
  select, textarea, .snippet-form input { width: 100%; box-sizing: border-box; border: 1px solid rgba(255,255,255,.1); border-radius: 8px; outline: none; background: #0c0c0d; color: rgba(255,255,255,.9); font: inherit; font-size: 9px; }
  select:focus-visible, textarea:focus-visible, .snippet-form input:focus-visible { border-color: rgba(62,150,255,.75); box-shadow: 0 0 0 2px rgba(38,118,255,.16); }
  select { height: 30px; margin-top: 7px; padding: 0 8px; }
  textarea { min-height: 58px; margin-top: 7px; padding: 8px; resize: vertical; line-height: 1.4; }
  .style-block header { display: flex; align-items: center; justify-content: space-between; }
  .style-block header small { color: rgba(255,255,255,.3); font-size: 7px; }
  .save-button, .quiet-button { display: inline-flex; align-items: center; justify-content: center; gap: 5px; min-height: 27px; margin-top: 6px; padding: 0 9px; border: 1px solid rgba(255,255,255,.12); border-radius: 8px; background: rgba(255,255,255,.9); color: #080808; font-size: 8px; font-weight: 650; }
  .save-button:disabled { cursor: default; opacity: .42; }
  .quiet-button { background: transparent; color: rgba(255,255,255,.55); }
  .style-block details { margin-top: 8px; padding-top: 7px; border-top: 1px solid rgba(255,255,255,.06); }
  .style-block summary { color: rgba(255,255,255,.62); font-size: 8px; cursor: pointer; }
  .style-block details p { margin: 6px 0 0; color: rgba(255,255,255,.34); font-size: 8px; line-height: 1.4; }
  .snippet-form label { display: block; }
  .snippet-two { display: grid; grid-template-columns: .8fr 1.2fr; gap: 7px; }
  .snippet-form input { height: 29px; margin-top: 5px; padding: 0 8px; }
  .snippet-form > label { margin-top: 8px; }
  .snippet-actions { display: flex; align-items: flex-end; justify-content: space-between; gap: 8px; }
  .snippet-actions > div { display: flex; gap: 5px; }
  .snippet-actions small { max-width: 170px; color: rgba(255,255,255,.32); font-size: 7px; line-height: 1.4; }
  .snippet-actions code { color: rgba(118,179,255,.86); font: inherit; }
  .snippet-list { border: 1px solid rgba(255,255,255,.07); border-radius: 12px; overflow: hidden; }
  .snippet-row { display: grid; grid-template-columns: 1fr 28px; align-items: center; min-height: 42px; padding: 4px 6px 4px 0; }
  .snippet-row + .snippet-row { border-top: 1px solid rgba(255,255,255,.06); }
  .snippet-edit { display: grid; grid-template-columns: 20px 1fr; gap: 5px; align-items: center; min-width: 0; height: 100%; padding: 5px 9px; border: 0; background: transparent; color: rgba(255,255,255,.45); text-align: left; }
  .snippet-edit:hover, .snippet-edit:focus-visible { color: #fff; outline: none; }
  .snippet-edit span { display: flex; flex-direction: column; min-width: 0; }
  .snippet-edit strong { color: rgba(255,255,255,.84); font-size: 9px; }
  .snippet-edit small { margin-top: 2px; overflow: hidden; color: rgba(118,179,255,.7); font-size: 8px; text-overflow: ellipsis; white-space: nowrap; }
  @media (prefers-reduced-motion: reduce) {
    .preference-list i, .preference-list i::after { transition: none; }
  }
  .term-form { display: grid; grid-template-columns: 1fr 28px; gap: 7px; margin-top: 9px; }
  .term-form input { min-width: 0; height: 32px; padding: 0 10px; border: 1px solid rgba(255,255,255,.1); border-radius: 9px; outline: none; background: rgba(255,255,255,.045); color: #fff; font-size: 10px; }
  .term-error {
    margin: 0 0 6px;
    color: #ff8f8f;
    font-size: 11px;
    line-height: 1.35;
  }

  .privacy-note { display: flex; align-items: center; gap: 5px; margin: 8px 2px; color: rgba(255,255,255,.38); font-size: 8px; }
  .term-list { border: 1px solid rgba(255,255,255,.07); border-radius: 12px; overflow: hidden; }
  .term-list > div { display: flex; align-items: center; justify-content: space-between; padding: 7px 9px; color: rgba(255,255,255,.8); font-size: 10px; }
  .term-list > div + div { border-top: 1px solid rgba(255,255,255,.06); }
  .empty-note { margin: 0; padding: 16px; color: rgba(255,255,255,.35); font-size: 9px; text-align: center; }
</style>
