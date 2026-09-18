import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  dictationMediaAvailability,
  dictationMediaAvailable,
  dictationMediaUnavailableMessage,
  extractDictationMediaIntent,
  parseDictationMediaVoiceChoice,
  refreshDictationMediaAvailability,
  searchDictationMedia,
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

  it("names the settings an operator still has to set", async () => {
    invoke.mockResolvedValueOnce({
      enabled: false,
      missing: ["GIPHY_PROXY_APPROVED", "GIPHY_API_KEY"],
    });
    await expect(refreshDictationMediaAvailability()).resolves.toBe(false);
    expect(dictationMediaAvailability()).toEqual({
      state: "disabled",
      missing: ["GIPHY_PROXY_APPROVED", "GIPHY_API_KEY"],
    });
    const message = dictationMediaUnavailableMessage();
    expect(message).toContain("GIPHY_PROXY_APPROVED");
    expect(message).toContain("GIPHY_API_KEY");
    expect(message).toContain("restart");
  });

  it("separates an unreachable backend from a disabled integration", async () => {
    invoke.mockRejectedValueOnce(new Error("signed out"));
    await refreshDictationMediaAvailability();
    expect(dictationMediaAvailability()).toEqual({ state: "unreachable" });
    // Sending an operator to .env for what is a connection problem is the
    // failure this whole distinction exists to prevent.
    expect(dictationMediaUnavailableMessage()).not.toContain(".env");
    expect(dictationMediaUnavailableMessage()).toContain("Connection");
  });

  it("degrades to a generic message for a backend that reports no list", async () => {
    invoke.mockResolvedValueOnce({ enabled: false });
    await refreshDictationMediaAvailability();
    expect(dictationMediaAvailability()).toEqual({ state: "disabled", missing: [] });
    expect(dictationMediaUnavailableMessage()).toContain("SELF_HOSTING");
  });

  it("ignores a malformed missing list rather than rendering it", async () => {
    invoke.mockResolvedValueOnce({ enabled: false, missing: [42, "GIPHY_API_KEY", null] });
    await refreshDictationMediaAvailability();
    expect(dictationMediaAvailability()).toEqual({
      state: "disabled",
      missing: ["GIPHY_API_KEY"],
    });
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
});
