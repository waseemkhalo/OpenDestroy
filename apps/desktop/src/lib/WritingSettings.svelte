<script lang="ts">
  import {
    BookOpenText,
    Check,
    ChevronDown,
    Languages,
    Plus,
    Quote,
    Save,
    Search,
    ShieldCheck,
    Sparkles,
    Trash2,
    WandSparkles,
    X,
  } from "@lucide/svelte";
  import {
    addPersonalDictationTerm,
    loadPersonalDictationTerms,
    removePersonalDictationTerm,
  } from "./dictation/dictationState";
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
  } from "./dictation/dictationComposer";

  type Tone = "natural" | "professional" | "casual";
  type Length = "concise" | "balanced" | "detailed";
  type CleanupKey = "self_correction" | "remove_fillers" | "app_formatting" | "selected_text_editing";

  let {
    scope,
    mode = "all",
    embedded = false,
    styleAvailable = false,
  }: {
    scope: string | null;
    mode?: "all" | "writing" | "dictionary";
    embedded?: boolean;
    styleAvailable?: boolean;
  } = $props();

  const isDictionary = $derived(mode === "dictionary");
  const isWriting = $derived(mode === "writing");
  const showDictionary = $derived(mode !== "writing");
  const showWriting = $derived(mode !== "dictionary");

  let preferences = $state<DictationPreferences>(currentDictationPreferences(null));
  let preferencesBusy = $state(false);
  let preferencesError = $state("");
  let styleInputError = $state("");
  let styleSample = $state("");
  let styleLearning = $state(false);
  let writingTab = $state<"style" | "shortcuts">("style");

  let personalTerms = $state<string[]>([]);
  let customTerm = $state("");
  let termSearch = $state("");
  let termsBusy = $state(false);
  let termsError = $state("");
  let termComposerOpen = $state(false);
  let dictionaryTab = $state<"words" | "shortcuts">("words");

  let snippets = $state<DictationSnippet[]>([]);
  let snippetsBusy = $state(false);
  let snippetsError = $state("");
  let snippetId = $state<string | undefined>(undefined);
  let snippetTitle = $state("");
  let snippetTrigger = $state("");
  let snippetBody = $state("");

  const languages = [
    ["auto", "Detect automatically"], ["en", "English"], ["es", "Spanish"], ["fr", "French"],
    ["de", "German"], ["pt", "Portuguese"], ["it", "Italian"], ["nl", "Dutch"],
    ["ja", "Japanese"], ["zh", "Chinese"], ["ko", "Korean"],
  ] as const;

  const tones: Array<[Tone, string, string]> = [
    ["natural", "Natural", "Like you, just clearer."],
    ["professional", "Professional", "Polished and formal."],
    ["casual", "Casual", "More relaxed."],
  ];
  const lengths: Array<[Length, string, string]> = [
    ["concise", "Concise", "Short and to the point."],
    ["balanced", "Balanced", "A bit more detail."],
    ["detailed", "Detailed", "More context."],
  ];
  const cleanupOptions: Array<[CleanupKey, string, string]> = [
    ["self_correction", "Natural corrections", "Keep the latest correction."],
    ["remove_fillers", "Clean up speech", "Remove filler and repetition."],
    ["app_formatting", "Match the app", "Shape output for the app."],
    ["selected_text_editing", "Edit selected text", "Rewrite selected text by voice."],
  ];

  const filteredTerms = $derived(
    personalTerms.filter((term) => term.toLocaleLowerCase().includes(termSearch.trim().toLocaleLowerCase())),
  );
  const parsedStyle = $derived(parseStyleNote(preferences.style_note));
  const savedStyleLabel = $derived(
    parsedStyle.custom.trim() ? "Save changes to apply these instructions" : "Add a personal style note",
  );
  const styleNoteLength = $derived(preferences.style_note.length);
  const styleNoteTooLong = $derived(styleNoteLength > 400);
  const instructionMax = $derived(
    Math.max(0, 400 - (/^\s*\[\[(?:tone|length):[^\]]+\]\]/iu.test(preferences.style_note) ? composeStyleNote(parsedStyle.tone, parsedStyle.length, "").length : 0)),
  );

  function parseStyleNote(note: string): { tone: Tone; length: Length; custom: string } {
    let remaining = note.trimStart();
    const hadLeadingSpace = remaining.length !== note.length;
    let tone: Tone = "natural";
    let length: Length = "balanced";
    let match: RegExpMatchArray | null;
    const tonePrefix = /^\[\[tone:(natural|professional|casual)\]\]\s*/iu;
    const lengthPrefix = /^\[\[length:(concise|balanced|detailed)\]\]\s*/iu;
    let changed = true;
    while (changed) {
      changed = false;
      match = remaining.match(tonePrefix);
      if (match) { tone = match[1].toLowerCase() as Tone; remaining = remaining.slice(match[0].length); changed = true; }
      match = remaining.match(lengthPrefix);
      if (match) { length = match[1].toLowerCase() as Length; remaining = remaining.slice(match[0].length); changed = true; }
    }
    const hasStructuredPrefix = remaining !== note.trimStart() || tone !== "natural" || length !== "balanced";
    return { tone, length, custom: hasStructuredPrefix ? remaining : (hadLeadingSpace ? note : remaining) };
  }

  function composeStyleNote(tone: Tone, length: Length, custom: string): string {
    return `[[tone:${tone}]][[length:${length}]]${custom ? ` ${custom}` : ""}`;
  }

  function updateStyleInstructions(value: string) {
    const current = parseStyleNote(preferences.style_note);
    const hasStructuredPrefix = /^\s*\[\[(?:tone:(?:natural|professional|casual)|length:(?:concise|balanced|detailed))\]\]/iu.test(preferences.style_note);
    const nextStyleNote = hasStructuredPrefix ? composeStyleNote(current.tone, current.length, value) : value;
    styleInputError = nextStyleNote.length > 400 ? `Writing instructions are ${nextStyleNote.length - 400} characters over the 400-character limit.` : "";
    preferences = {
      ...preferences,
      style_note: nextStyleNote,
    };
  }

  function selectTone(tone: Tone) {
    void updatePreferences({ ...preferences, style_note: composeStyleNote(tone, parsedStyle.length, parsedStyle.custom) }, "Could not save your tone preference.");
  }

  function selectLength(length: Length) {
    void updatePreferences({ ...preferences, style_note: composeStyleNote(parsedStyle.tone, length, parsedStyle.custom) }, "Could not save your writing length.");
  }

  $effect(() => {
    const requested = scope;
    // Clear every customer-authored value immediately at an account boundary.
    // The guards below also prevent a late response from repainting another account.
    preferences = currentDictationPreferences(requested);
    preferencesError = "";
    styleInputError = "";
    styleSample = "";
    styleLearning = false;
    personalTerms = [];
    customTerm = "";
    termSearch = "";
    termComposerOpen = false;
    termsError = "";
    snippets = [];
    snippetsError = "";
    clearSnippetForm();
    preferencesBusy = requested !== null;
    termsBusy = requested !== null;
    snippetsBusy = requested !== null;
    if (!requested) return;

    void loadDictationPreferences(requested)
      .then((profile) => { if (requested === scope) preferences = profile; })
      .catch(() => { if (requested === scope) preferencesError = "Could not load dictation settings."; })
      .finally(() => { if (requested === scope) preferencesBusy = false; });
    void loadPersonalDictationTerms(requested)
      .then((terms) => { if (requested === scope) personalTerms = terms; })
      .catch(() => { if (requested === scope) termsError = "Could not load your vocabulary."; })
      .finally(() => { if (requested === scope) termsBusy = false; });
    void loadDictationSnippets()
      .then((value) => { if (requested === scope) snippets = value; })
      .catch(() => { if (requested === scope) snippetsError = "Could not load voice shortcuts."; })
      .finally(() => { if (requested === scope) snippetsBusy = false; });
  });

  function inputValue(event: Event): string {
    return (event.currentTarget as HTMLInputElement | HTMLTextAreaElement).value;
  }

  function describeError(error: unknown, fallback: string): string {
    const text = error instanceof Error ? error.message : String(error ?? "");
    try {
      const parsed = JSON.parse(text) as { error?: string };
      if (parsed?.error) return parsed.error;
    } catch { /* Not a JSON API error — use the raw text below. */ }
    return text.trim() || fallback;
  }

  async function updatePreferences(next: DictationPreferences, fallback = "Could not save writing settings.") {
    const requested = scope;
    if (preferencesBusy || !requested) return;
    if (next.style_note.length > 400) {
      preferencesError = `Writing instructions are ${next.style_note.length - 400} characters over the 400-character limit.`;
      return;
    }
    preferencesBusy = true;
    preferencesError = "";
    const before = preferences;
    preferences = next;
    try {
      const saved = await saveDictationPreferences(next, requested);
      if (requested === scope && preferences.style_note === next.style_note) preferences = saved;
    } catch (error) {
      if (requested === scope) { if (preferences.style_note === next.style_note) preferences = before; preferencesError = describeError(error, fallback); }
    } finally {
      if (requested === scope) preferencesBusy = false;
    }
  }

  function togglePreference(key: CleanupKey) {
    void updatePreferences({ ...preferences, [key]: !preferences[key] });
  }

  async function learnStyle() {
    const requested = scope;
    if (!styleAvailable || styleLearning || styleSample.trim().length < 40 || !requested) return;
    styleLearning = true;
    preferencesError = "";
    try {
      const styleNote = await learnDictationStyle(styleSample, requested);
      if (requested === scope) {
        if (styleNote.length > 400) {
          preferencesError = `The learned style is ${styleNote.length - 400} characters over the 400-character limit.`;
        } else {
          preferences = { ...preferences, style_note: styleNote };
          styleSample = "";
        }
      }
    } catch (error) {
      if (requested === scope) preferencesError = describeError(error, "Could not learn this style.");
    } finally {
      if (requested === scope) styleLearning = false;
    }
  }

  async function addTerm() {
    const requested = scope;
    const term = customTerm.trim();
    if (!term || termsBusy || !requested) return;
    termsBusy = true; termsError = "";
    try {
      const next = await addPersonalDictationTerm(requested, term);
      if (requested === scope) { personalTerms = next; customTerm = ""; termComposerOpen = false; }
    } catch (error) {
      if (requested === scope) termsError = describeError(error, "Could not save that term.");
    } finally {
      if (requested === scope) termsBusy = false;
    }
  }

  async function removeTerm(term: string) {
    const requested = scope;
    if (termsBusy || !requested) return;
    termsBusy = true; termsError = "";
    try {
      const next = await removePersonalDictationTerm(requested, term);
      if (requested === scope) personalTerms = next;
    } catch (error) {
      if (requested === scope) termsError = describeError(error, "Could not remove that term.");
    } finally {
      if (requested === scope) termsBusy = false;
    }
  }

  function clearSnippetForm() { snippetId = undefined; snippetTitle = ""; snippetTrigger = ""; snippetBody = ""; }
  function editSnippet(snippet: DictationSnippet) { snippetId = snippet.id; snippetTitle = snippet.title; snippetTrigger = snippet.trigger; snippetBody = snippet.body; }

  async function saveSnippet() {
    const requested = scope;
    if (snippetsBusy || !requested || !snippetTitle.trim() || !snippetTrigger.trim() || !snippetBody.trim()) return;
    snippetsBusy = true; snippetsError = "";
    try {
      const saved = await saveDictationSnippet({ id: snippetId, title: snippetTitle, trigger: snippetTrigger, body: snippetBody });
      if (requested === scope) { snippets = [saved, ...snippets.filter((snippet) => snippet.id !== saved.id)]; clearSnippetForm(); }
    } catch (error) {
      if (requested === scope) snippetsError = describeError(error, "Could not save that shortcut.");
    } finally {
      if (requested === scope) snippetsBusy = false;
    }
  }

  async function removeSnippet(id: string) {
    const requested = scope;
    if (snippetsBusy || !requested) return;
    snippetsBusy = true; snippetsError = "";
    try {
      await deleteDictationSnippet(id);
      if (requested === scope) { snippets = snippets.filter((snippet) => snippet.id !== id); if (snippetId === id) clearSnippetForm(); }
    } catch (error) {
      if (requested === scope) snippetsError = describeError(error, "Could not remove that shortcut.");
    } finally {
      if (requested === scope) snippetsBusy = false;
    }
  }
</script>

<section class="writing-settings" class:embedded aria-label={isDictionary ? "Dictionary" : isWriting ? "Writing settings" : "Writing and dictionary settings"}>
  <div class="settings-content">
    {#if !embedded}
      <header class="page-heading">
        <p class="eyebrow">Personalize</p>
        <h1>{isDictionary ? "Dictionary" : isWriting ? "Writing" : "Writing & dictionary"}</h1>
        <p class="page-description">{isDictionary ? "The words that make you, you." : isWriting ? "Make every word sound like you." : "Shape the words Destroy helps you put down."}</p>
      </header>
    {/if}

    {#if !scope}
      <p class="scope-note" role="status">Finish speech setup to edit these settings. Your controls will stay here until an account is connected.</p>
    {/if}

    {#if showDictionary}
      <section class="surface" class:embedded aria-label="Dictionary" aria-busy={termsBusy || snippetsBusy}>
        <div class="surface-heading"><div><p class="eyebrow">Dictionary</p><h2>The words that make you, you.</h2></div><BookOpenText size={22} strokeWidth={1.45} aria-hidden="true" /></div>
        {#if embedded}<div class="embedded-intro"><p>A little context. Better dictation.</p><span>Teach Destroy the names, products, and shorthand that matter to you.</span></div>{/if}
        <div class="segmented" class:second={dictionaryTab === "shortcuts"} role="tablist" aria-label="Dictionary sections">
          <button type="button" role="tab" aria-selected={dictionaryTab === "words"} class:active={dictionaryTab === "words"} onclick={() => dictionaryTab = "words"}>Words</button>
          <button type="button" role="tab" aria-selected={dictionaryTab === "shortcuts"} class:active={dictionaryTab === "shortcuts"} onclick={() => dictionaryTab = "shortcuts"}>Shortcuts</button>
        </div>
        {#if dictionaryTab === "words"}
          <div class="tab-panel" role="tabpanel" aria-label="Words">
            {#if !embedded}<div class="panel-intro"><p>A little context. Better dictation.</p><span>Teach Destroy the names, products, and shorthand that matter to you.</span></div>{/if}
            <div class="word-toolbar">
              <label class="search-field"><Search size={18} strokeWidth={1.65} aria-hidden="true" /><span class="sr-only">Search your words</span><input value={termSearch} oninput={(event) => termSearch = inputValue(event)} placeholder="Search your words" disabled={termsBusy && personalTerms.length === 0} /></label>
              <button class="primary-button add-word-button" type="button" disabled={!scope || termsBusy} onclick={() => termComposerOpen = !termComposerOpen}><Plus size={16} strokeWidth={1.8} /> {termComposerOpen ? "Close" : "Add word"}</button>
            </div>
            {#if termComposerOpen}
              <form class="add-term-form" onsubmit={(event) => { event.preventDefault(); void addTerm(); }}><label><span>Word or phrase</span><input value={customTerm} oninput={(event) => customTerm = inputValue(event)} placeholder="A name, company, or product" aria-label="Personal vocabulary term" disabled={!scope || termsBusy} /></label><button class="primary-button" type="submit" disabled={!scope || !customTerm.trim() || termsBusy}><Check size={15} strokeWidth={1.9} /> Save word</button></form>
            {/if}
            {#if termsError}<p class="error-note" role="alert">{termsError}</p>{/if}
            <p class="privacy-note"><ShieldCheck size={14} strokeWidth={1.7} /> Saved to this setup and used only for your transcriptions.</p>
            {#if termsBusy && personalTerms.length === 0}<p class="empty-note">Loading your words…</p>
            {:else if filteredTerms.length === 0}<div class="empty-state"><BookOpenText size={24} strokeWidth={1.35} aria-hidden="true" /><strong>{termSearch.trim() ? "No matching words" : "Your dictionary is ready for its first word"}</strong><span>{termSearch.trim() ? "Try another search." : "Add names Destroy commonly misses — customers, accounts, and product terms."}</span></div>
            {:else}<div class="term-grid">{#each filteredTerms as term (term)}<article class="term-tile"><span>{term}</span><button class="tile-action" type="button" aria-label={`Remove ${term}`} disabled={termsBusy || !scope} onclick={() => void removeTerm(term)}><X size={15} strokeWidth={1.7} /></button></article>{/each}</div>{/if}
          </div>
        {:else}
          <div class="tab-panel" role="tabpanel" aria-label="Shortcuts"><div class="panel-intro"><p>Say less. Get more done.</p><span>Say a phrase and insert the whole message wherever you write.</span></div>{@render SnippetEditor({ scope, snippetsBusy, snippetId, snippetTitle, snippetTrigger, snippetBody, snippets, snippetsError, onTitle: (value: string) => snippetTitle = value, onTrigger: (value: string) => snippetTrigger = value, onBody: (value: string) => snippetBody = value, onClear: clearSnippetForm, onEdit: editSnippet, onRemove: removeSnippet, onSave: saveSnippet })}</div>
        {/if}
      </section>
    {/if}

    {#if showWriting}
      <section class="surface" class:embedded aria-label="Writing preferences" aria-busy={preferencesBusy || snippetsBusy}>
        <div class="surface-heading"><div><p class="eyebrow">Writing</p><h2>Make every word sound like you.</h2></div><WandSparkles size={22} strokeWidth={1.45} aria-hidden="true" /></div>
        <div class="segmented" class:second={writingTab === "shortcuts"} role="tablist" aria-label="Writing sections">
          <button type="button" role="tab" aria-selected={writingTab === "style"} class:active={writingTab === "style"} onclick={() => writingTab = "style"}>Style</button>
          <button type="button" role="tab" aria-selected={writingTab === "shortcuts"} class:active={writingTab === "shortcuts"} onclick={() => writingTab = "shortcuts"}>Voice shortcuts</button>
        </div>
        {#if writingTab === "style"}
          <div class="tab-panel" role="tabpanel" aria-label="Style">
            <div class="writing-layout"><div class="writing-controls">
              <p class="capability-note">{styleAvailable ? "Tone and style preferences are saved for your connected writing backend." : "Tone and style apply with a connected writing backend. Local dictation uses cleanup settings below."}</p>
              <p class="section-label">Tone</p>
              <div class="choice-grid tone-grid">{#each tones as [value, label, description]}<button type="button" class:selected={parsedStyle.tone === value} aria-pressed={parsedStyle.tone === value} disabled={!scope || preferencesBusy} onclick={() => selectTone(value)}><span class="choice-mark">{#if parsedStyle.tone === value}<Check size={13} strokeWidth={2.1} />{/if}</span><span><strong>{label}</strong><small>{description}</small></span></button>{/each}</div>
              <p class="section-label section-gap">Writing length</p>
              <div class="choice-grid length-grid">{#each lengths as [value, label, description]}<button type="button" class:selected={parsedStyle.length === value} aria-pressed={parsedStyle.length === value} disabled={!scope || preferencesBusy} onclick={() => selectLength(value)}><span class="choice-mark">{#if parsedStyle.length === value}<Check size={13} strokeWidth={2.1} />{/if}</span><span><strong>{label}</strong><small>{description}</small></span></button>{/each}</div>
              <label class="instruction-field"><span class="field-label"><Quote size={16} strokeWidth={1.6} /> Personal instructions</span><small>{savedStyleLabel}. Use a short note about rhythm or formatting.</small><textarea value={parsedStyle.custom} maxlength={instructionMax} disabled={!scope} oninput={(event) => updateStyleInstructions(inputValue(event))} placeholder="Keep my phrasing. Short paragraphs. No em dash." aria-label="Personal writing instructions"></textarea><span class:over-limit={styleNoteTooLong} class="character-count">{styleNoteLength}/400 characters</span></label>
              {#if styleInputError}<p class="error-note" role="alert">{styleInputError} Shorten the personal instructions before saving.</p>{/if}
              <div class="action-line"><span class="field-hint">Only this summary is saved.</span><button class="primary-button" type="button" disabled={!scope || preferencesBusy || styleNoteTooLong} onclick={() => void updatePreferences(preferences, "Could not save your writing style.")}><Save size={14} strokeWidth={1.8} /> Save changes</button></div>
              <details class="sample-disclosure"><summary><Sparkles size={15} strokeWidth={1.7} /><span>Learn from a writing sample</span><ChevronDown class="summary-chevron" size={15} /></summary><div class="disclosure-body"><p>Paste something that already sounds like you. Destroy extracts the style and discards the sample.</p><textarea bind:value={styleSample} disabled={!scope} placeholder="Paste at least a few sentences…" aria-label="Writing sample"></textarea><button class="secondary-button" type="button" disabled={!scope || !styleAvailable || styleLearning || styleSample.trim().length < 40} title={!styleAvailable ? "Connect a writing backend to learn a style." : undefined} onclick={() => void learnStyle()}><Sparkles size={13} strokeWidth={1.7} /> {styleLearning ? "Learning…" : styleAvailable ? "Learn this style" : "Unavailable for local dictation"}</button></div></details>
              <details class="behavior-disclosure"><summary><Languages size={15} strokeWidth={1.7} /><span>Dictation behavior</span><ChevronDown class="summary-chevron" size={15} /></summary><div class="disclosure-body"><label class="language-field"><span class="field-label">Language</span><small>Choose a language or let Destroy detect it.</small><select value={preferences.language} disabled={!scope || preferencesBusy} aria-label="Dictation language" onchange={(event) => void updatePreferences({ ...preferences, language: inputValue(event) }, "Could not save the language.")}>{#each languages as [value, label]}<option {value}>{label}</option>{/each}</select></label><p class="section-label behavior-label">Cleanup</p><div class="cleanup-grid">{#each cleanupOptions as [key, label, description]}<button type="button" class:selected={preferences[key]} role="switch" aria-checked={preferences[key]} disabled={!scope || preferencesBusy} onclick={() => togglePreference(key)}><span class="choice-mark">{#if preferences[key]}<Check size={13} strokeWidth={2.1} />{/if}</span><span><strong>{label}</strong><small>{description}</small></span></button>{/each}</div></div></details>
            </div><aside class="preview-card" aria-label="Illustrative writing preview"><div class="preview-heading"><span>Illustrative preview</span><small>Not a live transcript</small></div><div class="preview-window"><div class="window-bar"><i></i><i></i><i></i></div><div class="preview-copy"><p>Hi Alex,</p><p>Thanks for the update. Friday works for me.<br />See you then.</p><small>{parsedStyle.custom.trim() ? "Guided by your saved instructions." : "Save instructions to guide future writing."}</small></div></div><p class="preview-note">The final result still depends on your speech provider and the app you are writing in.</p></aside></div>
          </div>
        {:else}
          <div class="tab-panel" role="tabpanel" aria-label="Voice shortcuts"><div class="panel-intro"><p>Say a phrase, insert the whole message.</p><span>These shortcuts are available while you dictate in an editable field.</span></div>{@render SnippetEditor({ scope, snippetsBusy, snippetId, snippetTitle, snippetTrigger, snippetBody, snippets, snippetsError, onTitle: (value: string) => snippetTitle = value, onTrigger: (value: string) => snippetTrigger = value, onBody: (value: string) => snippetBody = value, onClear: clearSnippetForm, onEdit: editSnippet, onRemove: removeSnippet, onSave: saveSnippet })}</div>
        {/if}
      </section>
    {/if}

    {#if preferencesError}<p class="error-note page-error" role="alert">{preferencesError}</p>{/if}
    {#if showWriting}<p class="privacy-note footer-note"><ShieldCheck size={14} strokeWidth={1.7} /> Writing samples are processed for the requested edit and are not saved.</p>{/if}
  </div>
</section>

{#snippet SnippetEditor({ scope, snippetsBusy, snippetId, snippetTitle, snippetTrigger, snippetBody, snippets, snippetsError, onTitle, onTrigger, onBody, onClear, onEdit, onRemove, onSave }: { scope: string | null; snippetsBusy: boolean; snippetId: string | undefined; snippetTitle: string; snippetTrigger: string; snippetBody: string; snippets: DictationSnippet[]; snippetsError: string; onTitle: (value: string) => void; onTrigger: (value: string) => void; onBody: (value: string) => void; onClear: () => void; onEdit: (snippet: DictationSnippet) => void; onRemove: (id: string) => void; onSave: () => void; })}
  <form class="snippet-form" aria-label="Voice shortcut editor" onsubmit={(event) => { event.preventDefault(); onSave(); }}><div class="snippet-fields"><label for={`${mode}-snippet-title`}><span>Name</span><input id={`${mode}-snippet-title`} value={snippetTitle} oninput={(event) => onTitle(inputValue(event))} placeholder="Booking link" disabled={!scope || snippetsBusy} /></label><label for={`${mode}-snippet-trigger`}><span>Say</span><input id={`${mode}-snippet-trigger`} value={snippetTrigger} oninput={(event) => onTrigger(inputValue(event))} placeholder="my booking link" disabled={!scope || snippetsBusy} /></label></div><label for={`${mode}-snippet-body`}><span>Insert</span><textarea id={`${mode}-snippet-body`} value={snippetBody} oninput={(event) => onBody(inputValue(event))} placeholder="Pick a time here: https://…" disabled={!scope || snippetsBusy}></textarea></label><div class="snippet-actions"><small>User-defined variables: <code>{"{{my_company}}"}</code> <code>{"{{product_icp}}"}</code></small><div>{#if snippetId}<button type="button" class="quiet-button" onclick={onClear}>Cancel</button>{/if}<button class="primary-button" type="submit" disabled={!scope || snippetsBusy || !snippetTitle.trim() || !snippetTrigger.trim() || !snippetBody.trim()}><Save size={13} strokeWidth={1.8} /> {snippetId ? "Update" : "Add shortcut"}</button></div></div></form>
  {#if snippetsError}<p class="error-note" role="alert">{snippetsError}</p>{/if}<p class="privacy-note"><ShieldCheck size={14} strokeWidth={1.7} /> Saved to this setup. Define variables in Connection settings.</p><div class="saved-list">{#if snippets.length === 0}<p class="empty-note">Create one shortcut, then say “insert” followed by its phrase.</p>{:else}{#each snippets as snippet (snippet.id)}<div class="saved-row"><button type="button" class="saved-edit" onclick={() => onEdit(snippet)} disabled={snippetsBusy}><span><strong>{snippet.title}</strong><small>“{snippet.trigger}”</small></span></button><button class="icon-button" type="button" aria-label={`Delete ${snippet.title}`} disabled={snippetsBusy} onclick={() => onRemove(snippet.id)}><Trash2 size={14} strokeWidth={1.7} /></button></div>{/each}{/if}</div>
{/snippet}

<style>
  .writing-settings { --ink:#000; --card:#111; --card-strong:#151515; --ivory:#f4f0e6; --muted:#aaa9a2; --line:#ffffff14; --line-strong:#ffffff22; --signal:#d94c40; min-height:100%; box-sizing:border-box; color:var(--ivory); background:var(--ink); font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif; }
  .writing-settings button,.writing-settings input,.writing-settings select,.writing-settings textarea{font:inherit}.writing-settings button{cursor:pointer}.writing-settings button:disabled,.writing-settings input:disabled,.writing-settings select:disabled,.writing-settings textarea:disabled{cursor:default;opacity:.5}.settings-content{min-width:0;max-width:1180px;margin:0 auto}.page-heading{margin:0 0 30px}.page-heading h1{margin:4px 0 7px;color:var(--ivory);font:500 clamp(38px,5vw,62px)/.98 DestroyEditorial,Georgia,serif;letter-spacing:-.03em}.page-description{margin:0;color:#c5c2ba;font-size:15px;line-height:1.45}.eyebrow{margin:0;color:#aaa9a2;font-size:10px;font-weight:650;letter-spacing:.18em;line-height:1.2;text-transform:uppercase}.scope-note{margin:0 0 18px;padding:12px 14px;border:1px solid #d94c4033;border-radius:12px;background:#d94c400f;color:#edb3ad;font-size:12px;line-height:1.45}
  .surface{margin:0 0 22px;padding:26px;border:1px solid var(--line);border-radius:20px;background:var(--card);box-shadow:0 20px 50px #00000040}.surface-heading{display:flex;align-items:flex-start;justify-content:space-between;gap:20px;margin-bottom:22px}.surface-heading h2{margin:6px 0 0;color:var(--ivory);font:500 clamp(27px,3vw,42px)/1.02 DestroyEditorial,Georgia,serif;letter-spacing:-.025em}.surface-heading>:global(svg){color:#b6b3ab}.segmented{position:relative;display:inline-grid;grid-template-columns:repeat(2,minmax(130px,1fr));gap:4px;margin-bottom:26px;padding:4px;border:1px solid var(--line-strong);border-radius:999px;background:#1b1b1b;isolation:isolate}.segmented::before{position:absolute;z-index:0;top:4px;bottom:4px;left:4px;width:calc(50% - 2px);border-radius:999px;background:#4a4a4a;box-shadow:inset 0 1px #ffffff2e,0 2px 7px #00000050;content:"";transition:transform 220ms cubic-bezier(.2,.8,.2,1)}.segmented.second::before{transform:translateX(100%)}.segmented button{position:relative;z-index:1;min-height:37px;padding:0 18px;border:0;border-radius:999px;background:transparent;color:var(--muted);font-size:12px;font-weight:600;transition:color 180ms ease}.segmented button.active{background:transparent;color:var(--ivory);box-shadow:none}.segmented button:hover:not(.active){color:var(--ivory)}.segmented button:focus-visible{outline:2px solid var(--ivory);outline-offset:2px}.tab-panel{animation:panel-in 180ms ease both}.panel-intro{margin:0 0 20px}.panel-intro p{margin:0 0 5px;color:var(--ivory);font:500 25px/1.05 DestroyEditorial,Georgia,serif}.panel-intro span{color:var(--muted);font-size:12px;line-height:1.5}.embedded-intro{margin:0 0 24px}.embedded-intro p{margin:0 0 8px;color:var(--ivory);font:500 clamp(30px,3.4vw,38px)/1 DestroyEditorial,Georgia,serif;letter-spacing:-.02em}.embedded-intro span{color:#c5c2ba;font-size:18px;line-height:1.45}
  .word-toolbar{display:flex;align-items:center;gap:12px}.search-field{display:flex;flex-direction:row;align-items:center;gap:10px;flex:1;min-width:0;height:48px;margin:0;padding:0 15px;border:1px solid var(--line-strong);border-radius:999px;background:#171717;color:var(--muted)}.search-field:focus-within{border-color:#ffffff55;box-shadow:0 0 0 3px #ffffff0d}.search-field input{width:100%;min-width:0;padding:0;border:0;outline:0;background:transparent;color:var(--ivory);font-size:13px}.primary-button,.secondary-button,.quiet-button{display:inline-flex;align-items:center;justify-content:center;gap:7px;min-height:38px;padding:0 16px;border-radius:999px;font-size:12px;font-weight:650;white-space:nowrap}.primary-button{border:1px solid var(--ivory);background:var(--ivory);color:#111}.primary-button:hover:not(:disabled),.primary-button:focus-visible{border-color:#fff;background:#fff;outline:none;box-shadow:0 0 0 3px #f4f0e61f}.secondary-button,.quiet-button{border:1px solid var(--line-strong);background:transparent;color:var(--ivory)}.secondary-button:hover:not(:disabled),.secondary-button:focus-visible,.quiet-button:hover:not(:disabled),.quiet-button:focus-visible{border-color:#ffffff66;outline:none}.add-word-button{flex:0 0 auto;min-width:126px}.add-term-form{display:flex;align-items:flex-end;gap:12px;margin-top:14px;padding:14px;border:1px solid var(--line);border-radius:14px;background:var(--card-strong);animation:panel-in 180ms ease both}.add-term-form label{flex:1;margin:0}.add-term-form label>span,.snippet-form label>span{display:block;margin:0 0 7px;color:var(--muted);font-size:10px;font-weight:650;letter-spacing:.05em;text-transform:uppercase}.add-term-form input,.snippet-form input,.snippet-form textarea,.instruction-field textarea,.sample-disclosure textarea{box-sizing:border-box;width:100%;border:1px solid var(--line-strong);border-radius:10px;outline:0;background:#0b0b0b;color:var(--ivory)}.add-term-form input,.snippet-form input{height:40px;padding:0 12px;font-size:12px}.writing-settings input::placeholder,.writing-settings textarea::placeholder{color:#aaa9a277}.writing-settings input:focus-visible,.writing-settings textarea:focus-visible,.writing-settings select:focus-visible{border-color:#ffffff66;box-shadow:0 0 0 3px #ffffff0d;outline:none}.privacy-note{display:flex;align-items:flex-start;gap:7px;margin:14px 0 0;color:#aaa9a2b5;font-size:10px;line-height:1.45}.privacy-note>:global(svg){flex:0 0 auto;color:var(--signal)}.term-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px;margin-top:20px}.term-tile{display:flex;align-items:center;justify-content:space-between;gap:10px;min-height:78px;padding:16px 14px 16px 18px;border:1px solid var(--line);border-radius:16px;background:#151515;color:var(--ivory);box-shadow:inset 0 1px #ffffff06;transition:border-color 160ms ease,transform 160ms ease}.term-tile:hover{border-color:#ffffff2a;transform:translateY(-1px)}.term-tile>span{overflow-wrap:anywhere;font:500 21px/1.05 DestroyEditorial,Georgia,serif}.tile-action,.icon-button{display:inline-flex;align-items:center;justify-content:center;flex:0 0 auto;width:30px;height:30px;padding:0;border:0;border-radius:999px;background:transparent;color:var(--muted)}.tile-action:hover:not(:disabled),.tile-action:focus-visible,.icon-button:hover:not(:disabled),.icon-button:focus-visible{background:#ffffff0f;color:var(--ivory);outline:2px solid var(--signal);outline-offset:2px}.empty-state{display:flex;flex-direction:column;align-items:center;gap:8px;margin-top:20px;padding:38px 20px;border:1px dashed var(--line-strong);border-radius:16px;color:var(--muted);text-align:center}.empty-state>:global(svg){color:#bdb9af}.empty-state strong{color:var(--ivory);font:500 21px/1.1 DestroyEditorial,Georgia,serif}.empty-state span,.empty-note{max-width:420px;color:var(--muted);font-size:11px;line-height:1.5}.empty-note{margin:20px 0 0;padding:24px;border:1px dashed var(--line-strong);border-radius:14px;text-align:center}
  .writing-layout{display:grid;grid-template-columns:minmax(0,1.05fr) minmax(280px,.95fr);gap:28px;align-items:start}.capability-note{margin:0 0 22px;padding:11px 13px;border:1px solid var(--line);border-radius:11px;background:#151515;color:#aaa9a2;font-size:11px;line-height:1.45}.section-label,.field-label{display:flex;align-items:center;gap:7px;color:var(--ivory);font-size:13px;font-weight:600}.section-label{margin:0 0 12px}.section-gap{margin-top:22px}.choice-grid,.cleanup-grid{display:grid;gap:10px}.tone-grid,.length-grid{grid-template-columns:repeat(3,minmax(0,1fr))}.choice-grid button,.cleanup-grid button{display:grid;grid-template-columns:20px 1fr;gap:9px;align-items:start;min-height:102px;padding:14px 11px;border:1px solid var(--line);border-radius:16px;background:#151515;color:var(--ivory);text-align:left;transition:border-color 170ms ease,background 170ms ease,transform 170ms ease}.choice-grid button:hover:not(:disabled),.cleanup-grid button:hover:not(:disabled){border-color:#ffffff30;transform:translateY(-1px)}.choice-grid button.selected,.cleanup-grid button.selected{border-color:#ffffff3d;background:#1c1c1c;box-shadow:inset 0 1px #ffffff12}.choice-grid button:focus-visible,.cleanup-grid button:focus-visible{outline:2px solid var(--ivory);outline-offset:3px}.choice-mark{display:grid;place-items:center;width:20px;height:20px;border:1px solid #aaa9a277;border-radius:50%;color:#111}.selected .choice-mark{border-color:var(--ivory);background:var(--ivory)}.choice-grid strong,.cleanup-grid strong{display:block;font-size:12px;font-weight:600}.choice-grid small,.cleanup-grid small,.language-field>small,.instruction-field>small{display:block;margin-top:5px;color:var(--muted);font-size:10px;line-height:1.4}.language-field,.instruction-field{display:block;margin:22px 0 0}.language-field select{width:100%;height:40px;margin-top:11px;padding:0 11px;border:1px solid var(--line-strong);border-radius:10px;background:#0b0b0b;color:var(--ivory);font-size:12px}.instruction-field textarea{min-height:100px;margin-top:11px;padding:12px;font-size:12px;line-height:1.5;resize:vertical}.character-count{display:block;margin-top:6px;color:#85847f;font-size:10px;text-align:right}.character-count.over-limit{color:#edb3ad}.action-line{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-top:12px}.field-hint{color:#aaa9a299;font-size:10px}.sample-disclosure,.behavior-disclosure{margin-top:20px;border-top:1px solid var(--line)}.sample-disclosure summary,.behavior-disclosure summary{display:flex;align-items:center;gap:8px;padding:16px 0 4px;cursor:pointer;color:var(--ivory);font-size:12px;list-style:none}.sample-disclosure summary::-webkit-details-marker,.behavior-disclosure summary::-webkit-details-marker{display:none}:global(.summary-chevron){margin-left:auto;color:var(--muted);transition:transform 180ms ease}details[open]>summary :global(.summary-chevron){transform:rotate(180deg)}.disclosure-body p{margin:10px 0;color:var(--muted);font-size:11px;line-height:1.5}.sample-disclosure textarea{min-height:90px;padding:12px;font-size:12px;line-height:1.5;resize:vertical}.sample-disclosure .secondary-button{margin-top:9px}.behavior-label{margin-top:22px}.cleanup-grid{grid-template-columns:repeat(2,minmax(0,1fr));margin-top:10px}.cleanup-grid button{min-height:82px}.preview-card{padding:22px;border:1px solid var(--line);border-radius:18px;background:#151515}.preview-heading{display:flex;align-items:baseline;justify-content:space-between;gap:12px;margin-bottom:22px;color:#c7c3b9;font-size:12px}.preview-heading small{color:#85847f;font-size:9px;text-transform:uppercase;letter-spacing:.08em}.preview-window{overflow:hidden;min-height:325px;border:1px solid var(--line-strong);border-radius:14px;background:#0f0f0f}.window-bar{display:flex;gap:6px;height:42px;padding:15px;border-bottom:1px solid var(--line)}.window-bar i{width:7px;height:7px;border-radius:50%;background:#76746f}.preview-copy{padding:26px 22px;color:#ece8df;font-size:14px;line-height:1.55}.preview-copy p{margin:0 0 22px}.preview-copy small{display:block;margin-top:70px;color:#85847f;font-size:10px;line-height:1.45}.preview-note{margin:15px 0 0;color:#85847f;font-size:10px;line-height:1.45}
  .snippet-form{padding-top:2px}.snippet-fields{display:grid;grid-template-columns:.8fr 1.2fr;gap:10px}.snippet-form label{display:block;margin:0 0 12px}.snippet-form textarea{min-height:100px;padding:12px;font-size:12px;line-height:1.5;resize:vertical}.snippet-actions{display:flex;align-items:flex-end;justify-content:space-between;gap:12px;margin-top:2px}.snippet-actions>small{max-width:250px;color:#aaa9a299;font-size:10px;line-height:1.45}.snippet-actions code{color:#e3aaa4;font:inherit}.snippet-actions>div{display:flex;gap:8px}.saved-list{margin-top:20px;border-top:1px solid var(--line);border-bottom:1px solid var(--line)}.saved-row{display:grid;grid-template-columns:1fr 34px;align-items:center;min-height:58px}.saved-row+.saved-row{border-top:1px solid var(--line)}.saved-edit{min-width:0;padding:9px 10px;border:0;background:transparent;color:var(--ivory);text-align:left}.saved-edit:hover,.saved-edit:focus-visible{background:#ffffff08;outline:none}.saved-edit span{display:flex;flex-direction:column}.saved-edit strong{color:var(--ivory);font-size:12px;font-weight:600}.saved-edit small{margin-top:3px;overflow:hidden;color:#e3aaa4;font-size:10px;text-overflow:ellipsis;white-space:nowrap}.error-note{margin:12px 0 0;color:#edb3ad;font-size:11px;line-height:1.45}.page-error{margin:0 0 14px}.footer-note{margin:0 4px 14px}.sr-only{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
  @keyframes panel-in{from{opacity:.4;transform:translateY(4px)}to{opacity:1;transform:translateY(0)}}
  .writing-settings.embedded .surface{margin:0 0 28px;padding:0;border:0;border-radius:0;background:transparent;box-shadow:none}.writing-settings.embedded .surface-heading{display:none}.writing-settings.embedded .segmented{margin-bottom:30px}.writing-settings.embedded .panel-intro p{font-size:38px}.writing-settings.embedded .panel-intro span{font-size:18px}.writing-settings.embedded .word-toolbar{gap:16px}.writing-settings.embedded .search-field{height:56px}.writing-settings.embedded .add-word-button{min-height:56px;padding-inline:24px;font-size:14px}.writing-settings.embedded .term-grid{gap:16px;margin-top:24px}.writing-settings.embedded .term-tile{min-height:108px;padding:22px 18px 22px 22px;border-radius:18px}.writing-settings.embedded .term-tile>span{font-size:30px}.writing-settings.embedded .tile-action{width:34px;height:34px}.writing-settings.embedded .writing-layout{gap:34px}.writing-settings.embedded .capability-note{font-size:13px}.writing-settings.embedded .section-label{font-size:18px}.writing-settings.embedded .choice-grid button{min-height:100px;padding:20px 16px}.writing-settings.embedded .choice-grid strong{font-size:22px}.writing-settings.embedded .choice-grid small{font-size:14px}.writing-settings.embedded .choice-mark{width:24px;height:24px}.writing-settings.embedded .instruction-field textarea{min-height:128px;font-size:15px}.writing-settings.embedded .field-label{font-size:18px}.writing-settings.embedded .instruction-field>small{font-size:13px}.writing-settings.embedded .preview-copy{font-size:18px}.writing-settings.embedded .preview-window{min-height:390px}.writing-settings.embedded .preview-note{font-size:12px}.writing-settings.embedded .behavior-disclosure{margin-top:28px}
  .writing-settings.embedded{background:transparent;container-type:inline-size;container-name:writing-surface}
  .writing-settings .search-field input:focus,.writing-settings .search-field input:focus-visible{outline:none;border:0;box-shadow:none;background:transparent}
  .writing-settings.embedded .embedded-intro{padding-top:44px;margin-bottom:44px}
  .writing-settings.embedded .segmented button{font-size:18px;font-weight:400;min-height:40px}
  .writing-settings.embedded .search-field{max-width:655px}
  .writing-settings.embedded .search-field input{font-size:18px}
  .writing-settings.embedded .add-word-button{margin-left:auto;font-size:18px}
  .writing-settings.embedded .privacy-note{font-size:13px}
  .writing-settings.embedded .primary-button,.writing-settings.embedded .secondary-button{font-size:16px;min-height:44px}
  .writing-settings.embedded .snippet-form input,.writing-settings.embedded .snippet-form textarea,.writing-settings.embedded .add-term-form input{font-size:17px}
  .writing-settings.embedded .sample-disclosure summary,.writing-settings.embedded .behavior-disclosure summary{font-size:16px}
  .writing-settings.embedded .writing-layout{grid-template-columns:minmax(0,1.3fr) minmax(0,1fr)}
  .writing-settings.embedded .choice-grid strong{font:500 21px/1.15 DestroyEditorial,Georgia,serif}
  .writing-settings.embedded .choice-grid button{grid-template-columns:24px minmax(0,1fr);gap:14px}
  .writing-settings.embedded .selected .choice-mark{background:#e84438;border-color:#e84438;color:#fff}
  .writing-settings.embedded .section-label,.writing-settings.embedded .field-label{font-weight:400;color:#b6b5b0}
  .writing-settings.embedded .preview-heading{font-size:16px}
  .writing-settings.embedded .preview-copy small{font-size:13px}
  .writing-settings.embedded .capability-note{background:transparent;border:0;padding:0;margin-bottom:22px;line-height:1.5}
  @media(max-width:820px){.surface{padding:20px}.writing-layout{grid-template-columns:1fr}.preview-card{order:-1}.preview-window{min-height:220px}.preview-copy small{margin-top:35px}.term-grid{grid-template-columns:repeat(2,minmax(0,1fr))}.tone-grid,.length-grid{grid-template-columns:repeat(3,minmax(0,1fr))}.writing-settings.embedded .embedded-intro{padding-top:12px;margin-bottom:28px}}
  @media(max-width:560px){.surface{padding:16px;border-radius:16px}.writing-settings.embedded .surface{padding:0;border-radius:0}.surface-heading{margin-bottom:18px}.surface-heading h2{font-size:30px}.segmented{width:100%}.segmented button{padding-inline:10px}.word-toolbar{align-items:stretch;flex-direction:column}.add-word-button{width:100%}.add-term-form,.snippet-actions{align-items:stretch;flex-direction:column}.add-term-form .primary-button,.snippet-actions>div{align-self:flex-end}.tone-grid,.length-grid,.snippet-fields,.term-grid{grid-template-columns:1fr}.preview-card{padding:16px}.writing-settings.embedded .embedded-intro p{font-size:32px}.writing-settings.embedded .embedded-intro span{font-size:16px}.writing-settings.embedded .term-tile>span{font-size:27px}}
  @media(max-width:820px){.writing-settings.embedded .writing-layout{grid-template-columns:1fr}.writing-settings.embedded .preview-card{order:0}}
  .writing-settings.embedded .choice-grid button{min-height:76px;padding:14px 16px;align-items:center}
  .writing-settings.embedded .choice-grid small{font-size:13px}
  @container writing-surface (max-width:960px){
    .writing-settings.embedded .writing-layout{grid-template-columns:minmax(0,1fr);gap:24px}
    .writing-settings.embedded .preview-card{order:0;padding:18px}
    .writing-settings.embedded .preview-window{min-height:0}
    .writing-settings.embedded .preview-copy{padding:18px;font-size:16px}
    .writing-settings.embedded .preview-copy small{margin-top:16px}
    .writing-settings.embedded .preview-copy p{margin-bottom:12px}
    .writing-settings.embedded .preview-heading{margin-bottom:14px}
    .writing-settings.embedded .instruction-field textarea{min-height:104px}
  }
  @container writing-surface (max-width:520px){
    .writing-settings.embedded .tone-grid,.writing-settings.embedded .length-grid{grid-template-columns:1fr}
    .writing-settings.embedded .choice-grid button{min-height:66px}
    .writing-settings.embedded .choice-grid button>span:last-child{display:flex;flex-wrap:wrap;align-items:baseline;gap:5px 12px}
    .writing-settings.embedded .choice-grid small{margin:0}
  }
  @media(prefers-reduced-motion:reduce){.segmented::before,.segmented button,.tab-panel,.term-tile,.choice-grid button,:global(.summary-chevron){animation:none;transition:none}}
</style>
