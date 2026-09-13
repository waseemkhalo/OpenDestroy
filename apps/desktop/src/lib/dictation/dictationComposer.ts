import { invokeAccount as invoke } from "../account";

export type DictationPreferences = {
  self_correction: boolean;
  remove_fillers: boolean;
  app_formatting: boolean;
  selected_text_editing: boolean;
  language: string;
  style_note: string;
};

export type DictationSnippet = {
  id: string;
  title: string;
  trigger: string;
  body: string;
};

export type DictationTransformOperation =
  | "dictate"
  | "rewrite"
  | "edit_selected"
  | "correct_previous";

export type DictationTransformResponse = {
  text: string;
  applied: string[];
};

const DEFAULTS: DictationPreferences = {
  self_correction: true,
  remove_fillers: true,
  app_formatting: true,
  selected_text_editing: true,
  language: "auto",
  style_note: "",
};

const cachedPreferences = new Map<string, DictationPreferences>();
const loadedPreferenceTeams = new Set<string>();

function teamKey(teamId: string | null | undefined): string | null {
  const value = teamId?.trim();
  return value || null;
}

async function api<T>(method: string, path: string, body: unknown = null): Promise<T> {
  return invoke<T>("backend_api", { method, path, body });
}

export function currentDictationPreferences(teamId?: string | null): DictationPreferences {
  const key = teamKey(teamId);
  return { ...((key ? cachedPreferences.get(key) : undefined) ?? DEFAULTS) };
}

export function hasLoadedDictationPreferences(teamId?: string | null): boolean {
  const key = teamKey(teamId);
  return key !== null && loadedPreferenceTeams.has(key);
}

export async function loadDictationPreferences(
  teamId?: string | null,
): Promise<DictationPreferences> {
  const key = teamKey(teamId);
  const profile = await api<DictationPreferences>("GET", "/v1/dictation/preferences");
  if (key) {
    cachedPreferences.set(key, { ...profile });
    loadedPreferenceTeams.add(key);
  }
  return { ...profile };
}

export async function saveDictationPreferences(
  profile: DictationPreferences,
  teamId?: string | null,
): Promise<DictationPreferences> {
  const saved = await api<DictationPreferences>(
    "PUT",
    "/v1/dictation/preferences",
    profile,
  );
  const key = teamKey(teamId);
  if (key) {
    cachedPreferences.set(key, { ...saved });
    loadedPreferenceTeams.add(key);
  }
  return { ...saved };
}

export async function learnDictationStyle(
  sample: string,
  teamId?: string | null,
): Promise<string> {
  const response = await api<{ style_note: string }>(
    "POST",
    "/v1/dictation/style/learn",
    { sample },
  );
  const key = teamKey(teamId);
  if (key) {
    cachedPreferences.set(key, {
      ...currentDictationPreferences(key),
      style_note: response.style_note,
    });
    loadedPreferenceTeams.add(key);
  }
  return response.style_note;
}

export async function loadDictationSnippets(): Promise<DictationSnippet[]> {
  const response = await api<{ snippets: DictationSnippet[] }>(
    "GET",
    "/v1/dictation/snippets",
  );
  return response.snippets ?? [];
}

export async function saveDictationSnippet(
  snippet: Omit<DictationSnippet, "id"> & { id?: string },
): Promise<DictationSnippet> {
  return api<DictationSnippet>("PUT", "/v1/dictation/snippets", snippet);
}

export async function deleteDictationSnippet(id: string): Promise<void> {
  await api<unknown>("DELETE", `/v1/dictation/snippets/${encodeURIComponent(id)}`);
}

export async function transformDictation(input: {
  text: string;
  appKind: string;
  selectedText?: string | null;
  previousText?: string | null;
  operation: DictationTransformOperation;
}): Promise<DictationTransformResponse> {
  return api<DictationTransformResponse>("POST", "/v1/dictation/transform", {
    text: input.text,
    app_kind: input.appKind,
    selected_text: input.selectedText || null,
    previous_text: input.previousText || null,
    operation: input.operation,
  });
}

export function isUndoDictationCommand(value: string): boolean {
  return /^(?:please\s+)?(?:undo(?:\s+(?:that|dictation|last))?|take\s+that\s+back|scratch\s+that)[.!?]*$/iu.test(
    value.trim(),
  );
}

export function isPreviousDictationCorrection(value: string): boolean {
  const input = value.trim();
  return /^(?:please\s+)?(?:change|replace|correct)\s+.+\s+(?:to|with)\s+.+[.!?]*$/iu.test(input)
    || /^(?:please\s+)?(?:in\s+that|in\s+the\s+(?:last|previous)\s+dictation|make\s+that)\b.+/iu.test(input);
}

/**
 * Returns the content before an explicit terminal rewrite command. A bare
 * noun phrase such as "this is a rewrite" remains ordinary dictation; the
 * command must either be separated by punctuation or follow enough content
 * to be unambiguous. Selected-text "rewrite" is handled by edit_selected and
 * does not pass through this parser.
 */
export function extractTerminalRewriteCommand(value: string): string | null {
  const input = value.trim();
  const match = input.match(/^(.*?)(?:(\s*[,;:\u2014-]\s*)|(\s+))(?:please\s+)?rewrite[.!?]*$/iu);
  const content = match?.[1]?.trim().replace(/[,;:\u2014-]+$/u, "").trim();
  if (!content) return null;

  const punctuationDelimited = Boolean(match?.[2]);
  const words = content.split(/\s+/u);
  if (!punctuationDelimited) {
    if (words.length < 4) return null;
    const previous = words.at(-1)?.replace(/[^\p{L}\p{N}']/gu, "").toLocaleLowerCase();
    if (["a", "an", "the", "this", "that", "my", "your", "our", "to"].includes(previous ?? "")) {
      return null;
    }
  }
  return content;
}

export function resetDictationComposerCache(teamId?: string | null): void {
  const key = teamKey(teamId);
  if (key) {
    cachedPreferences.delete(key);
    loadedPreferenceTeams.delete(key);
    return;
  }
  cachedPreferences.clear();
  loadedPreferenceTeams.clear();
}
