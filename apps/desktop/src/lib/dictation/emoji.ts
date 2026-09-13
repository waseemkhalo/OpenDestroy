export type DictationEmojiIntent = {
  query: string;
  leadingText: string;
};

export type DictationEmojiChoice = {
  emoji: string;
  label: string;
};

// The boundary between the rep's message and the command cannot require a
// sentence terminator: speech-to-text punctuates inconsistently, and a miss
// pastes the literal spoken command into the customer's field. Both patterns
// are anchored on a command verb, so a whitespace boundary stays specific.
// See the matching rationale in `giphy.ts`.
const INTENTS = [
  /^(?:(?<leading>.+?)[.,!?;:]?\s+)?(?:please\s+)?(?:add|insert|use|send|drop|give|show)\s+(?:me\s+)?(?:a|an|some)?\s*(?<query>.+?)\s+(?:emoji|emojis|reaction)[.!?]*$/iu,
  /^(?:(?<leading>.+?)[.,!?;:]?\s+)?(?:please\s+)?(?:add|insert|use|send|drop|give|show)\s+(?:me\s+)?(?:a|an|some)?\s*(?:emoji|emojis|reaction)(?:\s+(?:of|for|that\s+means|showing))?\s+(?<query>.+?)[.!?]*$/iu,
];

export function extractDictationEmojiIntent(value: string): DictationEmojiIntent | null {
  const input = value.trim();
  const match = INTENTS.map((pattern) => pattern.exec(input)).find(Boolean);
  const query = match?.groups?.query?.trim().replace(/^[“”"']+|[“”"']+$/gu, "");
  if (!query || [...query].length > 60) return null;
  return { query, leadingText: match?.groups?.leading?.trim() ?? "" };
}

const SETS: Array<{ terms: RegExp; choices: DictationEmojiChoice[] }> = [
  { terms: /laugh|funny|lol|lmao|dying/iu, choices: [["😂", "Laughing"], ["🤣", "Rolling"], ["😭", "Dying"]].map(([emoji, label]) => ({ emoji, label })) },
  { terms: /celebrat|congrat|excited|win|party/iu, choices: [["🎉", "Celebrate"], ["🙌", "Big win"], ["🥳", "Party"]].map(([emoji, label]) => ({ emoji, label })) },
  { terms: /love|heart|care|appreciat/iu, choices: [["❤️", "Love"], ["🥰", "Warm"], ["🫶", "Appreciate"]].map(([emoji, label]) => ({ emoji, label })) },
  { terms: /agree|yes|good|perfect|done|approve/iu, choices: [["👍", "Agree"], ["✅", "Done"], ["👌", "Perfect"]].map(([emoji, label]) => ({ emoji, label })) },
  { terms: /think|hmm|unsure|question|curious/iu, choices: [["🤔", "Thinking"], ["🧐", "Looking"], ["💭", "Considering"]].map(([emoji, label]) => ({ emoji, label })) },
  { terms: /sorry|sad|disappoint|bad news/iu, choices: [["🙏", "Sorry"], ["😔", "Disappointed"], ["💙", "Support"]].map(([emoji, label]) => ({ emoji, label })) },
];

export function emojiChoices(query: string): DictationEmojiChoice[] {
  return SETS.find((set) => set.terms.test(query))?.choices
    ?? [{ emoji: "🙂", label: "Warm" }, { emoji: "✨", label: "Positive" }, { emoji: "👍", label: "Agree" }];
}
