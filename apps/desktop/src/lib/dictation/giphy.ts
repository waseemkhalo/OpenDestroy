import { invokeAccount as invoke } from "../account";
import { invoke as invokeNative } from "@tauri-apps/api/core";

export type DictationMediaKind = "gif" | "sticker";

export type DictationMediaResult = {
  id: string;
  title: string;
  alt_text: string;
  preview_url: string;
  content_url: string;
  source_url: string;
  width: number;
  height: number;
  kind: DictationMediaKind;
  /** Local-development fixture; never deliver it into another app. */
  mock?: boolean;
  /** Server id when this result came from the rep's encrypted favorites. */
  favorite_id?: string;
};

export type DictationMediaFavorite = Omit<DictationMediaResult, "id" | "mock" | "favorite_id"> & {
  id: string;
  provider_id: string;
  query: string;
};

export type DictationMediaSearchResponse = {
  query: string;
  kind: DictationMediaKind;
  provider: "giphy";
  attribution: string;
  results: DictationMediaResult[];
};

export type DictationMediaIntent = {
  kind: DictationMediaKind;
  query: string;
  leadingText: string;
};

export type DictationMediaDeliveryRequest = {
  leadingText: string;
  contentUrl: string;
  sourceUrl: string;
  altText: string;
};

/** Keep provider results on the native binary-media delivery route. The native
 * side revalidates the GIPHY URLs, bounds the download, preserves clipboard
 * contents, and supplies the plain link fallback when inline media paste is
 * unavailable. */
export function mediaDeliveryRequest(
  result: DictationMediaResult,
  leadingText: string,
): DictationMediaDeliveryRequest {
  return {
    leadingText,
    contentUrl: result.content_url,
    sourceUrl: result.source_url,
    altText: result.alt_text,
  };
}

export type DictationMediaVoiceChoice =
  | { type: "select"; index: number }
  | { type: "kind"; kind: DictationMediaKind }
  | { type: "rotate" }
  | { type: "cancel" };

let mediaAvailable = false;
const MAX_MEDIA_QUERY_CHARS = 50;
const MAX_MEDIA_OFFSET = 4999;

function validateMediaRequest(query: string, kind: DictationMediaKind, offset: number): string {
  const value = query.trim();
  if (!value || [...value].length > MAX_MEDIA_QUERY_CHARS) {
    throw new Error("Enter a GIPHY search query");
  }
  if (!Number.isSafeInteger(offset) || offset < 0 || offset > MAX_MEDIA_OFFSET) {
    throw new Error("GIPHY search offset is out of range");
  }
  if (kind !== "gif" && kind !== "sticker") {
    throw new Error("Choose GIF or sticker search");
  }
  return value;
}

/** Refreshes the server-owned provider capability without exposing its key. */
export async function refreshDictationMediaAvailability(): Promise<boolean> {
  try {
    const response = await invoke<{ enabled?: boolean }>("backend_api", {
      method: "GET",
      path: "/v1/dictation/media/status",
      body: null,
    });
    mediaAvailable = response?.enabled === true;
  } catch {
    mediaAvailable = false;
  }
  return mediaAvailable;
}

export function dictationMediaAvailable(): boolean {
  return mediaAvailable;
}

const KIND = "(?<kind>gif|jif|giphy|sticker)s?";
const VERB = "(?:add|insert|find|use|show|give|put|paste|drop|send)";

/**
 * Boundary between the rep's message and the media command.
 *
 * Speech-to-text punctuation is unreliable, so this cannot require a sentence
 * terminator: "omg that is so funny find me a gif of spongebob laughing" is the
 * same request as the punctuated version, and the transcript Destroy gets back is
 * whichever one the provider felt like emitting. The costs are asymmetric — a
 * miss pastes the literal spoken command into the customer's field, while a
 * false positive only opens a picker the rep cancels — so the verb-anchored
 * patterns treat any whitespace as a valid boundary.
 */
const LEADING = "(?:(?<leading>.+?)[.,!?;:]?\\s+)?";
/**
 * `BARE_KIND` has no verb to anchor on, so a free boundary would let any
 * sentence merely containing "gif" capture a query ("People communicate with
 * GIFs all the time"). Real punctuation is the only usable signal there.
 */
const LEADING_PUNCTUATED = "(?:(?<leading>.+?)[.!?]\\s+)?";
const KIND_FIRST = new RegExp(
  `^${LEADING}(?:please\\s+)?${VERB}\\s+(?:me\\s+)?(?:a|an|some)?\\s*${KIND}(?:\\s+(?:of|for|that\\s+says))?\\s+(?<query>.+?)[.!?]*$`,
  "iu",
);
const QUERY_FIRST = new RegExp(
  `^${LEADING}(?:please\\s+)?${VERB}\\s+(?:me\\s+)?(?:a|an|some)?\\s*(?<query>.+?)\\s+${KIND}[.!?]*$`,
  "iu",
);
const BARE_KIND = new RegExp(
  `^${LEADING_PUNCTUATED}(?:please\\s+)?${KIND}(?:\\s+(?:of|for))?\\s+(?<query>.+?)[.!?]*$`,
  "iu",
);

function trimQuery(value: string): string {
  return value.trim().replace(/^[“”"']+|[“”"']+$/gu, "").trim();
}

const TRAILING_REQUEST =
  /(?:^|[,;:]\s*|\s)(?:(?:can|could|would|will)\s+you(?:\s+please)?|i\s+(?:need|want)\s+(?:you\s+)?to|go\s+ahead\s+and|please)$/iu;

function stripRequestScaffolding(value: string): string {
  let text = value.trim().replace(/\s+/gu, " ");
  let previous = "";
  while (text !== previous) {
    previous = text;
    text = text.replace(TRAILING_REQUEST, "").replace(/[,;:\u2014-]+$/u, "").trim();
  }
  return text;
}

/**
 * Recognizes only explicit media commands. The captured phrase is sent to
 * GIPHY literally; Destroy does not rewrite customer-authored search text.
 */
export function extractDictationMediaIntent(transcript: string): DictationMediaIntent | null {
  const input = transcript.trim();
  const match = KIND_FIRST.exec(input) ?? QUERY_FIRST.exec(input) ?? BARE_KIND.exec(input);
  if (!match?.groups) return null;
  const query = trimQuery(match.groups.query ?? "");
  if (!query || [...query].length > 50) return null;
  return {
    kind: /^sticker/iu.test(match.groups.kind ?? "") ? "sticker" : "gif",
    query,
    leadingText: stripRequestScaffolding(match.groups.leading ?? ""),
  };
}

/** Commands accepted while the picker owns the notch and backtick is held. */
export function parseDictationMediaVoiceChoice(value: string): DictationMediaVoiceChoice | null {
  const normalized = value.trim().toLocaleLowerCase().replace(/[.!?]+$/u, "");
  const spokenNumbers = new Map([
    ["one", 0], ["1", 0], ["first", 0],
    ["two", 1], ["2", 1], ["second", 1],
    ["three", 2], ["3", 2], ["third", 2],
  ]);
  const index = spokenNumbers.get(normalized.replace(/^(?:choose|select|pick)\s+/u, ""));
  if (index != null) return { type: "select", index };
  if (/^(?:show\s+)?gifs?$/u.test(normalized)) return { type: "kind", kind: "gif" };
  if (/^(?:show\s+)?stickers?$/u.test(normalized)) return { type: "kind", kind: "sticker" };
  // "None of these" is the most common thing a rep says at a three-result
  // picker, so rotation has to answer to plain rejection, not just to a verb.
  if (
    /^(?:(?:show|give|find)\s+me\s+)?(?:some\s+|any\s+|something\s+)?(?:more|others?|other\s+(?:ones|options)|another|next|different(?:\s+ones?)?|else|shuffle|rotate|refresh|new\s+ones?|more\s+options|keep\s+looking|try\s+again|next\s+(?:page|three))$/u
      .test(normalized)
    || /^(?:no|none|not)\s+(?:of\s+)?(?:these|those|them|good|right)$/u.test(normalized)
  ) {
    return { type: "rotate" };
  }
  if (/^(?:cancel|never mind|nevermind|close)$/u.test(normalized)) return { type: "cancel" };
  return null;
}

/**
 * `offset` is the provider result the window starts at. The picker advances it
 * in steps of three when the rep asks for different results.
 */
export async function searchDictationMedia(
  query: string,
  kind: DictationMediaKind,
  offset = 0,
): Promise<DictationMediaSearchResponse> {
  const value = validateMediaRequest(query, kind, offset);
  return invoke<DictationMediaSearchResponse>("backend_api", {
    method: "POST",
    path: "/v1/dictation/media/search",
    body: { query: value, kind, offset },
  });
}

/** Direct personal-key search. Results are provider output, never fixtures. */
export function searchDirectGiphy(
  query: string,
  kind: DictationMediaKind,
  offset = 0,
): Promise<DictationMediaSearchResponse> {
  const value = validateMediaRequest(query, kind, offset);
  return invokeNative<DictationMediaSearchResponse>("search_giphy", { query: value, kind, offset });
}

export async function loadDictationMediaFavorites(): Promise<DictationMediaFavorite[]> {
  const response = await invoke<{ favorites?: DictationMediaFavorite[] }>("backend_api", {
    method: "GET",
    path: "/v1/dictation/media/favorites",
    body: null,
  });
  return response.favorites ?? [];
}

export async function saveDictationMediaFavorite(
  query: string,
  result: DictationMediaResult,
): Promise<DictationMediaFavorite> {
  return invoke<DictationMediaFavorite>("backend_api", {
    method: "PUT",
    path: "/v1/dictation/media/favorites",
    body: {
      provider_id: result.id,
      query,
      title: result.title,
      alt_text: result.alt_text,
      preview_url: result.preview_url,
      content_url: result.content_url,
      source_url: result.source_url,
      width: result.width,
      height: result.height,
      kind: result.kind,
    },
  });
}

export async function deleteDictationMediaFavorite(id: string): Promise<void> {
  await invoke("backend_api", {
    method: "DELETE",
    path: `/v1/dictation/media/favorites/${encodeURIComponent(id)}`,
    body: null,
  });
}

export function favoriteAsMediaResult(favorite: DictationMediaFavorite): DictationMediaResult {
  return {
    id: favorite.provider_id,
    title: favorite.title,
    alt_text: favorite.alt_text,
    preview_url: favorite.preview_url,
    content_url: favorite.content_url,
    source_url: favorite.source_url,
    width: favorite.width,
    height: favorite.height,
    kind: favorite.kind,
    favorite_id: favorite.id,
  };
}
