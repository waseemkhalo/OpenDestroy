import { invokeAccount as invoke } from "../account";

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

export type DictationMediaVoiceChoice =
  | { type: "select"; index: number }
  | { type: "kind"; kind: DictationMediaKind }
  | { type: "rotate" }
  | { type: "cancel" };

/**
 * Why media commands are or are not usable.
 *
 * `disabled` and `unreachable` both fail closed, but they are different jobs for
 * whoever is looking: one is a `.env` edit on the backend, the other is a
 * connection problem in this app. Collapsing them into one boolean sends an
 * operator to the wrong file.
 */
export type DictationMediaAvailability =
  | { state: "enabled" }
  | { state: "disabled"; missing: string[] }
  | { state: "unreachable" };

let availability: DictationMediaAvailability = { state: "unreachable" };

/** Refreshes the server-owned provider capability without exposing its key. */
export async function refreshDictationMediaAvailability(): Promise<boolean> {
  try {
    const response = await invoke<{ enabled?: boolean; missing?: unknown }>("backend_api", {
      method: "GET",
      path: "/v1/dictation/media/status",
      body: null,
    });
    availability =
      response?.enabled === true
        ? { state: "enabled" }
        : {
            state: "disabled",
            // A backend older than this route reports no list; an empty one
            // degrades to the generic message rather than naming nothing.
            missing: Array.isArray(response?.missing)
              ? response.missing.filter((v): v is string => typeof v === "string")
              : [],
          };
  } catch {
    availability = { state: "unreachable" };
  }
  return availability.state === "enabled";
}

export function dictationMediaAvailable(): boolean {
  return availability.state === "enabled";
}

export function dictationMediaAvailability(): DictationMediaAvailability {
  return availability;
}

/** Operator-facing sentence for a blocked media command. */
export function dictationMediaUnavailableMessage(
  current: DictationMediaAvailability = availability,
): string {
  if (current.state === "enabled") return "";
  if (current.state === "unreachable") {
    return "Could not reach the backend to check media. Check Connection & device.";
  }
  return current.missing.length
    ? `Media is disabled on the backend. Set ${current.missing.join(", ")} in .env, then restart the service.`
    : "Media is disabled on the backend. See docs/SELF_HOSTING.md for the GIPHY settings.";
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
    leadingText: (match.groups.leading ?? "").trim(),
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
  try {
    return await invoke<DictationMediaSearchResponse>("backend_api", {
      method: "POST",
      path: "/v1/dictation/media/search",
      body: { query, kind, offset },
    });
  } catch (error) {
    throw error;
  }
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
