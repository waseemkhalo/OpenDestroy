/**
 * Spoken attachment references — "attach my pitch deck", "see attached demo video".
 *
 * This is the retrieval sibling of [`giphy.ts`](./giphy.ts) and [`emoji.ts`](./emoji.ts),
 * and the cost asymmetry is inverted. A GIF is *discovered*: any laughing
 * SpongeBob satisfies the request, so a loose match is generous. A file is
 * *retrieved*: there is exactly one correct deck, and attaching the wrong one
 * to a customer is a mistake Destroy caused. So this module is deliberately
 * stricter than the media patterns, and resolution always ends in a picker.
 */

export type DictationAttachmentKind =
  | "presentation"
  | "document"
  | "spreadsheet"
  | "pdf"
  | "video"
  | "image"
  | "any";

export type DictationAttachmentIntent = {
  /** Spoken description of the wanted file, filler stripped. Sent for ranking. */
  query: string;
  /** Type implied by the spoken noun; narrows and ranks, never hard-excludes. */
  kind: DictationAttachmentKind;
  /**
   * The prose to write into the field.
   *
   * Imperative commands ("attach my deck") are instructions to Destroy and are
   * removed. Referential phrases ("please see attached deck") are the user's own
   * sentence and are kept — pasting "Hey Amy," and silently eating "please see
   * attached deck" would mangle a message the user meant to send.
   */
  messageText: string;
};

type NounRule = { kind: DictationAttachmentKind; terms: string[] };

/**
 * Ordered longest-phrase-first within each rule so "slide deck" wins over the
 * bare "deck", and checked in list order so a compound noun ("demo video")
 * resolves on its head noun rather than on "demo".
 */
const NOUNS: NounRule[] = [
  {
    kind: "presentation",
    terms: ["slide deck", "slidedeck", "pitch deck", "sales deck", "deck", "slides", "presentation", "keynote"],
  },
  { kind: "video", terms: ["demo video", "walkthrough", "screencast", "recording", "video", "demo"] },
  { kind: "spreadsheet", terms: ["spreadsheet", "sheet", "pricing model", "model", "budget", "forecast"] },
  {
    /**
     * Ordinary business paperwork, not only the deck-and-proposal vocabulary of
     * a first demo. A rep sending a resume, an NDA, or an invoice is speaking
     * about a specific file of their own exactly as much as one sending a
     * proposal, and a noun missing from this list fails silently: the utterance
     * falls through to plain dictation and the command is pasted as prose.
     */
    kind: "document",
    terms: [
      "non-disclosure agreement",
      "non disclosure agreement",
      "statement of work",
      "curriculum vitae",
      "scope of work",
      "case study",
      "one pager",
      "one-pager",
      "order form",
      "onepager",
      "agreement",
      "proposal",
      "playbook",
      "contract",
      "document",
      "estimate",
      "template",
      "invoice",
      "resume",
      "report",
      "agenda",
      "letter",
      "memo",
      "brief",
      "quote",
      "doc",
      "nda",
      "sow",
      "cv",
    ],
  },
  { kind: "image", terms: ["screenshot", "diagram", "mockup", "logo", "image", "photo", "picture"] },
  { kind: "pdf", terms: ["pdf"] },
  { kind: "any", terms: ["attachment", "file"] },
];

const NOUN_ALTERNATION = NOUNS.flatMap((rule) => rule.terms)
  .sort((a, b) => b.length - a.length)
  .map((term) => term.replace(/[-\s]/gu, "[-\\s]"))
  .join("|");

/** Same boundary rationale as `giphy.ts`: speech-to-text punctuation is unreliable. */
const LEADING = "(?:(?<leading>.+?)[.,!?;:]?\\s+)?";
const NOUN_TAIL = `(?<query>[^.!?]*?(?:${NOUN_ALTERNATION})[^.!?]*?)`;

/**
 * Verbs that name the attach action outright, so any determiner is safe.
 *
 * The split is by what the verb *means*, not by how common it is. "Attach the
 * NDA", "add the proposal", "pull up that case study" are all instructions to
 * Destroy no matter which determiner follows, so requiring "my"/"our" only made the
 * feature fail on the natural way to ask for a shared team document.
 */
const IMPERATIVE_EXPLICIT = new RegExp(
  `^${LEADING}(?:please\\s+)?(?:attach|include|add|grab|bring\\s+up|pull\\s+up)\\s+(?:me\\s+)?(?:a|an|the|my|our|that)?\\s*${NOUN_TAIL}[.!?]*$`,
  "iu",
);
/**
 * The looser verbs are ordinary sales vocabulary — "I'll send the proposal over
 * tomorrow" must not open a picker mid-sentence — so they additionally require a
 * first-person possessive to prove the user means a specific file of their own.
 */
const IMPERATIVE_POSSESSIVE = new RegExp(
  `^${LEADING}(?:please\\s+)?(?:send|share|find|use)\\s+(?:me\\s+)?(?:my|our)\\s+${NOUN_TAIL}[.!?]*$`,
  "iu",
);
/**
 * "Please see attached demo video" is prose, not a command. It is recognised so
 * the file still resolves, but the sentence stays in the message.
 */
const REFERENTIAL = new RegExp(
  `^(?<leading>.*\\battach(?:ed|ing)\\b\\s+(?:is\\s+|are\\s+)?(?:a|an|the|my|our)?\\s*${NOUN_TAIL})[.!?]*$`,
  "iu",
);

const FILLER = /^(?:a|an|the|my|our|that|this|some)\s+/iu;

/**
 * Request scaffolding spoken *to Destroy*, sitting at the tail of the leading half
 * of the split.
 *
 * "Hey Amy, can you attach my pitch deck" splits into "Hey Amy, can you" and the
 * command. The command is removed as an instruction, so leaving "can you" behind
 * writes a fragment into the customer's field that the user never meant to send.
 * Only the tail is stripped — "Hey Amy" is the user's own sentence and stays.
 */
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

function cleanQuery(value: string): string {
  let query = value.trim().replace(/^[“”"']+|[“”"']+$/gu, "").trim();
  while (FILLER.test(query)) query = query.replace(FILLER, "");
  return query.replace(/\s+/gu, " ").trim();
}

/** The most specific noun present decides the type; unmatched text stays in the query. */
export function attachmentKindFor(query: string): DictationAttachmentKind {
  const haystack = query.toLocaleLowerCase();
  for (const rule of NOUNS) {
    for (const term of rule.terms) {
      const pattern = new RegExp(`\\b${term.replace(/[-\s]/gu, "[-\\s]")}\\b`, "iu");
      if (pattern.test(haystack)) return rule.kind;
    }
  }
  return "any";
}

/**
 * Recognises only explicit file references. The captured phrase is ranked
 * server-side against the user's saved file links; Destroy never guesses a file
 * from context alone, and never attaches without the picker.
 */
export function extractDictationAttachmentIntent(
  transcript: string,
): DictationAttachmentIntent | null {
  const input = transcript.trim();
  if (!input) return null;

  const referential = REFERENTIAL.exec(input);
  if (referential?.groups) {
    const query = cleanQuery(referential.groups.query ?? "");
    if (!query || [...query].length > 80) return null;
    return {
      query,
      kind: attachmentKindFor(query),
      // The whole utterance is the user's sentence; nothing is stripped.
      messageText: input.replace(/\s+/gu, " ").trim(),
    };
  }

  const match = IMPERATIVE_EXPLICIT.exec(input) ?? IMPERATIVE_POSSESSIVE.exec(input);
  if (!match?.groups) return null;
  const query = cleanQuery(match.groups.query ?? "");
  if (!query || [...query].length > 80) return null;
  return {
    query,
    kind: attachmentKindFor(query),
    messageText: stripRequestScaffolding(match.groups.leading ?? ""),
  };
}
