import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  dictationMediaAvailable,
  extractDictationMediaIntent,
  parseDictationMediaVoiceChoice,
  refreshDictationMediaAvailability,
  searchDictationMedia,
  mediaDeliveryRequest,
} from "./giphy";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}));

beforeEach(() => {
  invoke.mockReset();
});

describe("dictation media availability", () => {
  it("only enables provider commands after an authenticated positive status", async () => {
    invoke.mockResolvedValueOnce({ enabled: true });
    await expect(refreshDictationMediaAvailability()).resolves.toBe(true);
    expect(dictationMediaAvailable()).toBe(true);
    expect(invoke).toHaveBeenCalledWith("backend_api", {
      method: "GET",
      path: "/v1/dictation/media/status",
      body: null,
    });
  });

  it("fails closed when status is unavailable", async () => {
    invoke.mockRejectedValueOnce(new Error("signed out"));
    await expect(refreshDictationMediaAvailability()).resolves.toBe(false);
    expect(dictationMediaAvailable()).toBe(false);
  });
});

describe("extractDictationMediaIntent", () => {
  it("extracts a query that appears before the media kind", () => {
    expect(extractDictationMediaIntent("Add a proud reaction GIF.")).toEqual({
      kind: "gif",
      query: "proud reaction",
      leadingText: "",
    });
  });

  it("extracts kind-first sticker commands", () => {
    expect(extractDictationMediaIntent("Insert a sticker of thumbs up")).toEqual({
      kind: "sticker",
      query: "thumbs up",
      leadingText: "",
    });
  });

  it("preserves message text before an explicit command", () => {
    expect(
      extractDictationMediaIntent("That demo was incredible. Add a proud reaction gif"),
    ).toEqual({
      kind: "gif",
      query: "proud reaction",
      leadingText: "That demo was incredible",
    });
  });

  it("supports a terse voice command", () => {
    expect(extractDictationMediaIntent("sticker thank you")).toEqual({
      kind: "sticker",
      query: "thank you",
      leadingText: "",
    });
  });

  it("supports natural send, put, and find-me commands", () => {
    expect(extractDictationMediaIntent("find me a gif of happy dance")).toMatchObject({
      kind: "gif",
      query: "happy dance",
    });
    expect(extractDictationMediaIntent("put a sticker of nice work")).toMatchObject({
      kind: "sticker",
      query: "nice work",
    });
    expect(extractDictationMediaIntent("send a giphy of applause")).toMatchObject({
      kind: "gif",
      query: "applause",
    });
  });

  it("treats a transcribed \"gift\" as a GIF in command positions", () => {
    const expected = { kind: "gif", query: "proud reaction", leadingText: "This demo was incredible" };
    expect(extractDictationMediaIntent("This demo was incredible add a proud reaction gift")).toMatchObject(expected);
    expect(extractDictationMediaIntent("This demo was incredible. Add a proud reaction gift.")).toMatchObject(expected);
    expect(extractDictationMediaIntent("Laughing gift.")).toMatchObject({ kind: "gif", query: "Laughing" });
    expect(extractDictationMediaIntent("Insert a gift of a dancing cat")).toMatchObject({ kind: "gif", query: "a dancing cat" });
    expect(extractDictationMediaIntent("Reply with a thumbs up gift")).toMatchObject({ kind: "gif", query: "thumbs up" });
  });

  it("keeps sentences about real gifts as dictation", () => {
    for (const text of [
      "I bought her a gift",
      "Thanks for the gift",
      "Nice gift!",
      "Birthday gift",
      "Don't forget to add a birthday gift",
      "Add a gift card",
      "Add a small gift",
      "Add my gift",
      "We should send a gift",
      "Get me a gift for Christmas",
      "Can you add a gift receipt",
    ]) {
      expect(extractDictationMediaIntent(text), text).toBeNull();
    }
  });

  it("accepts a short verbless request as the whole utterance", () => {
    expect(extractDictationMediaIntent("Laughing GIF.")).toMatchObject({ kind: "gif", query: "Laughing", leadingText: "" });
    expect(extractDictationMediaIntent("funny cat gif")).toMatchObject({ kind: "gif", query: "funny cat" });
    expect(extractDictationMediaIntent("A thumbs up sticker")).toMatchObject({ kind: "sticker", query: "thumbs up" });
    expect(extractDictationMediaIntent("apple gif")).toMatchObject({ query: "apple" });
  });

  it("keeps short sentences about GIFs as dictation", () => {
    expect(extractDictationMediaIntent("I love GIFs.")).toBeNull();
    expect(extractDictationMediaIntent("We need more stickers")).toBeNull();
    expect(extractDictationMediaIntent("That was a great sticker")).toBeNull();
    expect(extractDictationMediaIntent("Thanks for the nice gift")).toBeNull();
  });

  it("accepts more verbs and punctuation after the media word", () => {
    expect(extractDictationMediaIntent("Get me a laughing gif")).toMatchObject({ kind: "gif", query: "laughing" });
    expect(extractDictationMediaIntent("Grab a GIF of a dancing cat")).toMatchObject({ query: "a dancing cat" });
    expect(extractDictationMediaIntent("Search for a gif of applause")).toMatchObject({ query: "applause" });
    expect(extractDictationMediaIntent("Reply with a thumbs up sticker")).toMatchObject({ kind: "sticker", query: "thumbs up" });
    expect(extractDictationMediaIntent("Add a GIF, laughing.")).toMatchObject({ kind: "gif", query: "laughing" });
    expect(extractDictationMediaIntent("GIF: happy dance")).toMatchObject({ kind: "gif", query: "happy dance" });
  });

  it("accepts common speech-model spellings of gif", () => {
    expect(extractDictationMediaIntent("Add a laughing giff")).toMatchObject({ kind: "gif", query: "laughing" });
    expect(extractDictationMediaIntent("Send me a jiff of a cat")).toMatchObject({ kind: "gif", query: "a cat" });
    expect(extractDictationMediaIntent("I bought her a gift")).toBeNull();
  });

  it("does not intercept ordinary speech that mentions GIFs", () => {
    expect(extractDictationMediaIntent("People communicate with GIFs all the time.")).toBeNull();
  });

  // Speech-to-text may or may not punctuate mid-sentence. The same spoken
  // request must split the same way either way, or the literal command text
  // ends up pasted into the customer's field.
  it("splits a message from its command without any punctuation", () => {
    expect(
      extractDictationMediaIntent("omg that is so funny find me a gif of spongebob laughing"),
    ).toEqual({
      kind: "gif",
      query: "spongebob laughing",
      leadingText: "omg that is so funny",
    });
  });

  it("does not paste a request scaffold addressed to Destroy", () => {
    expect(extractDictationMediaIntent("Hey Amy, can you add a laughing GIF")).toEqual({
      kind: "gif",
      query: "laughing",
      leadingText: "Hey Amy",
    });
    expect(extractDictationMediaIntent("could you insert a thank you sticker")).toEqual({
      kind: "sticker",
      query: "thank you",
      leadingText: "",
    });
  });

  it("splits on a comma boundary", () => {
    expect(
      extractDictationMediaIntent("OMG that is so funny, find me a GIF of SpongeBob laughing"),
    ).toEqual({
      kind: "gif",
      query: "SpongeBob laughing",
      leadingText: "OMG that is so funny",
    });
  });

  it("splits an unpunctuated sticker request", () => {
    expect(extractDictationMediaIntent("haha love it add a laughing sticker")).toEqual({
      kind: "sticker",
      query: "laughing",
      leadingText: "haha love it",
    });
  });

  it("keeps leading text when the message itself contains a command verb", () => {
    expect(
      extractDictationMediaIntent("I'll send you the deck find me a gif of applause"),
    ).toEqual({
      kind: "gif",
      query: "applause",
      leadingText: "I'll send you the deck",
    });
  });

  it("supports leading text before a bare kind command", () => {
    expect(
      extractDictationMediaIntent("that is hilarious. gif of spongebob laughing"),
    ).toEqual({
      kind: "gif",
      query: "spongebob laughing",
      leadingText: "that is hilarious",
    });
  });

  // The bare-kind pattern has no verb to anchor on, so it still requires real
  // punctuation. Without that guard any sentence mentioning a GIF would capture
  // a query and hijack the paste.
  it("does not let a bare kind hijack unpunctuated speech about GIFs", () => {
    expect(extractDictationMediaIntent("People communicate with GIFs all the time")).toBeNull();
    expect(extractDictationMediaIntent("we should use more stickers in email")).toBeNull();
  });

  // Reported from real use: the combined form pasted the whole utterance as
  // literal text while the bare command worked.
  it("splits a movie GIF request, comma or not", () => {
    const expected = { kind: "gif", query: "Scarface", leadingText: "I think Scarface is the coolest movie in the world" };
    expect(
      extractDictationMediaIntent("I think Scarface is the coolest movie in the world, find me a GIF of Scarface"),
    ).toEqual(expected);
    expect(
      extractDictationMediaIntent("I think Scarface is the coolest movie in the world find me a GIF of Scarface"),
    ).toEqual(expected);
    expect(
      extractDictationMediaIntent("I think Scarface is the coolest movie in the world. Find me a GIF of Scarface."),
    ).toEqual(expected);
  });

  it("rejects provider queries longer than the API limit", () => {
    expect(extractDictationMediaIntent(`add ${"x".repeat(51)} gif`)).toBeNull();
  });
});

describe("parseDictationMediaVoiceChoice", () => {
  it("selects numbered results", () => {
    expect(parseDictationMediaVoiceChoice("two")).toEqual({ type: "select", index: 1 });
    expect(parseDictationMediaVoiceChoice("pick third")).toEqual({ type: "select", index: 2 });
  });

  it("switches libraries or cancels", () => {
    expect(parseDictationMediaVoiceChoice("stickers")).toEqual({ type: "kind", kind: "sticker" });
    expect(parseDictationMediaVoiceChoice("never mind")).toEqual({ type: "cancel" });
  });

  it("rotates the window when the rep rejects what is shown", () => {
    for (const phrase of [
      "more",
      "show me more",
      "different",
      "different ones",
      "another",
      "shuffle",
      "next",
      "none of these",
      "no good",
      "try again",
    ]) {
      expect(parseDictationMediaVoiceChoice(phrase)).toEqual({ type: "rotate" });
    }
  });

  it("rejects unrelated speech", () => {
    expect(parseDictationMediaVoiceChoice("send it")).toBeNull();
    // A rotation phrase must not swallow speech that carries its own query.
    expect(parseDictationMediaVoiceChoice("more coffee gifs")).toBeNull();
  });
});

describe("searchDictationMedia", () => {
  it("asks the gateway for the requested rotation offset", async () => {
    invoke.mockResolvedValueOnce({ query: "laughing", kind: "gif", provider: "giphy", attribution: "", results: [] });
    await searchDictationMedia("laughing", "gif", 6);
    expect(invoke).toHaveBeenCalledWith("backend_api", {
      method: "POST",
      path: "/v1/dictation/media/search",
      body: { query: "laughing", kind: "gif", offset: 6 },
    });
  });

  it("defaults to the first window", async () => {
    invoke.mockResolvedValueOnce({ query: "laughing", kind: "gif", provider: "giphy", attribution: "", results: [] });
    await searchDictationMedia("laughing", "gif");
    expect(invoke).toHaveBeenCalledWith("backend_api", {
      method: "POST",
      path: "/v1/dictation/media/search",
      body: { query: "laughing", kind: "gif", offset: 0 },
    });
  });

  it("rejects unbounded provider requests before IPC", async () => {
    await expect(searchDictationMedia("x".repeat(51), "gif")).rejects.toThrow(
      "Enter a GIPHY search query",
    );
    await expect(searchDictationMedia("laughing", "gif", -1)).rejects.toThrow(
      "GIPHY search offset is out of range",
    );
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("media delivery", () => {
  it("uses native binary delivery for personal and backend provider results", () => {
    const result = {
      id: "gif-1",
      title: "A GIF",
      alt_text: "A waving GIF",
      preview_url: "https://media.giphy.com/preview.gif",
      content_url: "https://media.giphy.com/content.gif",
      source_url: "https://giphy.com/gifs/gif-1",
      width: 320,
      height: 180,
      kind: "gif" as const,
    };
    expect(mediaDeliveryRequest(result, "Hi Amy")).toEqual({
      leadingText: "Hi Amy",
      contentUrl: result.content_url,
      sourceUrl: result.source_url,
      altText: result.alt_text,
    });
  });
});
