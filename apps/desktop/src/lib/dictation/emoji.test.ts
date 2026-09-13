import { describe, expect, it } from "vitest";
import { emojiChoices, extractDictationEmojiIntent } from "./emoji";

describe("spoken emoji intent", () => {
  it("keeps leading text and extracts the expression", () => {
    expect(extractDictationEmojiIntent("That is hilarious. Add a laughing emoji")).toEqual({
      leadingText: "That is hilarious",
      query: "laughing",
    });
  });

  // Speech-to-text punctuates inconsistently; the same spoken request must
  // split the same way either way. See the rationale in `giphy.ts`.
  it("splits a message from its emoji command without punctuation", () => {
    expect(extractDictationEmojiIntent("that is so funny add a laughing emoji")).toEqual({
      leadingText: "that is so funny",
      query: "laughing",
    });
  });

  it("does not intercept ordinary speech mentioning emoji", () => {
    expect(extractDictationEmojiIntent("emoji are everywhere these days")).toBeNull();
  });

  it("offers semantic choices instead of a literal character search", () => {
    expect(emojiChoices("laughing").map((choice) => choice.emoji)).toEqual(["😂", "🤣", "😭"]);
  });
});
