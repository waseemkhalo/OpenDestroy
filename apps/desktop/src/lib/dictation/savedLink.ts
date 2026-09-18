/**
 * Turning a pasted share URL into a saved link the spoken picker can find.
 *
 * The catalog is matched on `name` and `keywords` only — see the `/v1/links/search`
 * ranking — so a link saved with a poor name is a link the user will never reach
 * by voice. Asking someone to hand-write JSON for that is how the field ends up
 * holding one entry named "deck" and nothing else.
 *
 * What this module will *not* do is invent a title. A Google share URL carries a
 * file id and nothing else: there is no honest way to read "Q3 Pitch Deck" out of
 * `/document/d/1AbC.../edit`, and guessing one would put a wrong name in front of
 * a customer. So the rule is: derive what the URL actually contains — the kind of
 * file and, where the provider puts it in the path, the title — and leave the rest
 * to the person, with the recognized source shown so they know what they pasted.
 */

import type { DictationAttachmentKind } from "./attachment";

export type SavedLink = { name: string; url: string; keywords?: string };

export type SavedLinkDraft = {
  /** Normalized absolute HTTPS URL, safe to send to `/v1/links`. */
  url: string;
  /** Title read out of the URL, or `""` when it carries none. Never invented. */
  name: string;
  kind: DictationAttachmentKind;
  /** Suggested spoken-search terms. Editable before saving. */
  keywords: string;
  /** What the URL was recognized as, shown so the user can confirm the paste. */
  source: string;
};

export type SavedLinkRejection =
  | "empty"
  | "not-a-url"
  /** Kept distinct from `not-a-url`: an http:// link is understood and refused. */
  | "insecure"
  | "credentials";

export type SavedLinkParse =
  | { ok: true; draft: SavedLinkDraft }
  | { ok: false; reason: SavedLinkRejection };

/** Mirrors the backend's own name and keyword bounds so a draft cannot be rejected on save. */
const MAX_NAME_BYTES = 200;
const MAX_KEYWORDS_BYTES = 1000;

/**
 * Truncates to a UTF-8 byte budget, not a code-unit count.
 *
 * The backend measures these in bytes. Clamping by `String.slice` would let a
 * name of 200 CJK characters — 600 bytes — pass here and come back as the
 * backend's generic "every link needs a name and HTTPS URL", which reads like
 * the URL was wrong. Whole code points only, so truncation never leaves half a
 * character behind.
 */
function clampBytes(value: string, max: number): string {
  const encoder = new TextEncoder();
  if (encoder.encode(value).length <= max) return value;
  let out = "";
  let used = 0;
  for (const point of value) {
    const size = encoder.encode(point).length;
    if (used + size > max) break;
    out += point;
    used += size;
  }
  return out;
}

const EXTENSION_KINDS: ReadonlyArray<readonly [DictationAttachmentKind, readonly string[]]> = [
  ["presentation", ["ppt", "pptx", "key", "odp"]],
  ["spreadsheet", ["xls", "xlsx", "xlsm", "csv", "tsv", "ods", "numbers"]],
  ["pdf", ["pdf"]],
  ["video", ["mp4", "mov", "m4v", "webm", "avi", "mkv"]],
  ["image", ["png", "jpg", "jpeg", "gif", "webp", "svg", "heic", "avif"]],
  ["document", ["doc", "docx", "rtf", "txt", "md", "pages", "odt"]],
];

/**
 * Spoken synonyms seeded into keywords for each kind.
 *
 * A user asking for "my deck" produces the query "pitch deck", which is matched
 * against name and keywords as plain terms. Saving a Google Slides link as
 * kind `presentation` does nothing for that search on its own, so the words
 * people actually say go into the keywords the ranking can see.
 */
const KIND_KEYWORDS: Record<DictationAttachmentKind, string> = {
  presentation: "presentation deck slides",
  spreadsheet: "spreadsheet sheet",
  document: "document doc",
  pdf: "pdf document",
  video: "video recording",
  image: "image picture",
  any: "",
};

function safeDecode(value: string): string {
  try {
    return decodeURIComponent(value);
  } catch {
    // A malformed escape is the provider's problem, not a reason to reject a link.
    return value;
  }
}

/** Filename and slug separators become spaces; the result is a title, not an identifier. */
function tidy(value: string): string {
  const collapsed = value
    .replace(/[_+]+/gu, " ")
    .replace(/-+/gu, " ")
    .replace(/\s+/gu, " ")
    .trim();
  return clampBytes(collapsed, MAX_NAME_BYTES);
}

function extensionOf(segment: string): string {
  const dot = segment.lastIndexOf(".");
  return dot > 0 ? segment.slice(dot + 1).toLowerCase() : "";
}

function kindForExtension(extension: string): DictationAttachmentKind {
  for (const [kind, extensions] of EXTENSION_KINDS) {
    if (extensions.includes(extension)) return kind;
  }
  return "any";
}

/** A filename is a title once its extension and separators are gone. */
function nameFromSegment(segment: string): string {
  const decoded = safeDecode(segment);
  const dot = decoded.lastIndexOf(".");
  return tidy(dot > 0 ? decoded.slice(0, dot) : decoded);
}

function segmentsOf(url: URL): string[] {
  return url.pathname.split("/").filter(Boolean);
}

function hostMatches(host: string, domain: string): boolean {
  return host === domain || host.endsWith(`.${domain}`);
}

type Recognition = { source: string; kind: DictationAttachmentKind; name: string };

const GOOGLE_PRODUCTS: Record<string, { source: string; kind: DictationAttachmentKind }> = {
  document: { source: "Google Docs", kind: "document" },
  spreadsheets: { source: "Google Sheets", kind: "spreadsheet" },
  presentation: { source: "Google Slides", kind: "presentation" },
  forms: { source: "Google Forms", kind: "document" },
  file: { source: "Google Drive", kind: "any" },
  drive: { source: "Google Drive", kind: "any" },
};

/**
 * Notion puts the title in the slug with the page id appended:
 * `Quarterly-Plan-2f1a…` over 32 hexadecimal characters. Only that exact tail is
 * removed, so a title that merely ends in a word stays intact.
 */
function notionName(segments: string[]): string {
  const slug = segments.at(-1) ?? "";
  return tidy(safeDecode(slug).replace(/-?[0-9a-f]{32}$/iu, ""));
}

function recognize(url: URL): Recognition {
  const host = url.hostname.toLowerCase().replace(/^www\./u, "");
  const segments = segmentsOf(url);

  if (hostMatches(host, "google.com")) {
    const product = GOOGLE_PRODUCTS[segments[0] ?? ""] ?? { source: "Google", kind: "any" as const };
    // Deliberately nameless: a Google share URL holds a file id, never the title.
    return { ...product, name: "" };
  }

  if (hostMatches(host, "dropbox.com")) {
    const last = segments.at(-1) ?? "";
    return {
      source: "Dropbox",
      kind: kindForExtension(extensionOf(last)),
      name: nameFromSegment(last),
    };
  }

  if (hostMatches(host, "notion.so") || hostMatches(host, "notion.site")) {
    return { source: "Notion", kind: "document", name: notionName(segments) };
  }

  if (hostMatches(host, "figma.com")) {
    // /file/<key>/<Name> and /design/<key>/<Name> both carry the title third.
    const named = ["file", "design", "proto", "board"].includes(segments[0] ?? "");
    return {
      source: "Figma",
      kind: "image",
      name: named ? tidy(safeDecode(segments[2] ?? "")) : "",
    };
  }

  if (hostMatches(host, "loom.com")) return { source: "Loom", kind: "video", name: "" };
  if (hostMatches(host, "youtube.com") || hostMatches(host, "youtu.be")) {
    return { source: "YouTube", kind: "video", name: "" };
  }
  if (hostMatches(host, "vimeo.com")) return { source: "Vimeo", kind: "video", name: "" };

  const last = segments.at(-1) ?? "";
  const extension = extensionOf(last);
  const kind = kindForExtension(extension);
  return {
    source: kind === "any" ? "Link" : extension.toUpperCase(),
    kind,
    // Only a recognized file extension proves the segment is a filename rather
    // than a routing path, so "/pricing/enterprise" does not become a title.
    name: kind === "any" ? "" : nameFromSegment(last),
  };
}

/** Source and kind words the ranking can match, minus anything the name already says. */
function suggestKeywords(recognition: Recognition): string {
  const spoken = new Set<string>();
  const inName = new Set(recognition.name.toLocaleLowerCase().split(/\s+/u).filter(Boolean));
  for (const word of `${KIND_KEYWORDS[recognition.kind]} ${recognition.source}`.split(/\s+/u)) {
    const term = word.toLocaleLowerCase().trim();
    if (term && term !== "link" && !inName.has(term)) spoken.add(term);
  }
  return clampBytes([...spoken].join(" "), MAX_KEYWORDS_BYTES);
}

/**
 * Parses a pasted link into a draft, or explains why it cannot be saved.
 *
 * A bare `docs.google.com/...` gets an `https://` prefix, because pasting a URL
 * without its scheme is a copy artefact rather than a statement about transport.
 * An explicit `http://` is refused instead of silently upgraded: the user said
 * which protocol they meant, and the backend would reject it regardless.
 */
export function parseSavedLink(input: string): SavedLinkParse {
  const raw = input.trim();
  if (!raw) return { ok: false, reason: "empty" };
  if (/^http:\/\//iu.test(raw)) return { ok: false, reason: "insecure" };
  if (/^[a-z][a-z0-9+.-]*:/iu.test(raw) && !/^https:\/\//iu.test(raw)) {
    return { ok: false, reason: "not-a-url" };
  }

  let url: URL;
  try {
    url = new URL(/^https:\/\//iu.test(raw) ? raw : `https://${raw}`);
  } catch {
    return { ok: false, reason: "not-a-url" };
  }
  if (url.protocol !== "https:" || !url.hostname || !url.hostname.includes(".")) {
    return { ok: false, reason: "not-a-url" };
  }
  // The backend rejects these too; refusing here keeps a credential out of the
  // saved catalog and out of whatever field the link is later pasted into.
  if (url.username || url.password) return { ok: false, reason: "credentials" };

  const recognition = recognize(url);
  return {
    ok: true,
    draft: {
      url: url.toString(),
      name: recognition.name,
      kind: recognition.kind,
      keywords: suggestKeywords(recognition),
      source: recognition.source,
    },
  };
}

export function savedLinkRejectionMessage(reason: SavedLinkRejection): string {
  switch (reason) {
    case "empty":
      return "Paste a link to add.";
    case "insecure":
      return "Only https links can be saved.";
    case "credentials":
      return "That link contains a username or password. Use a plain share link.";
    default:
      return "That does not look like a link. Copy the share URL from the file.";
  }
}

/**
 * Prepares a draft for `/v1/links`, or returns why it is not ready.
 *
 * The name is required rather than defaulted: an unnamed link is unreachable by
 * voice, so saving one silently would only look like it worked.
 */
export function savedLinkFromDraft(
  draft: SavedLinkDraft,
  name: string,
  keywords: string,
): { ok: true; link: SavedLink } | { ok: false; message: string } {
  const trimmed = clampBytes(name.trim(), MAX_NAME_BYTES);
  if (!trimmed) {
    return { ok: false, message: "Give this link a name you would say out loud." };
  }
  const terms = clampBytes(keywords.trim().replace(/\s+/gu, " "), MAX_KEYWORDS_BYTES);
  return { ok: true, link: { name: trimmed, url: draft.url, ...(terms ? { keywords: terms } : {}) } };
}
