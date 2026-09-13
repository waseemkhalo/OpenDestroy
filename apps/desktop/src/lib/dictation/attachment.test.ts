import { describe, expect, it } from "vitest";

import {
  attachmentKindFor,
  extractDictationAttachmentIntent,
} from "./attachment";

describe("spoken attachment intent", () => {
  it("splits an imperative command from the message", () => {
    const intent = extractDictationAttachmentIntent("Hey Amy, attach my pitch deck");
    expect(intent).toEqual({
      query: "pitch deck",
      kind: "presentation",
      // The boundary comma is consumed with the split, matching the shipped
      // media path in `giphy.ts`; the composer re-punctuates the result.
      messageText: "Hey Amy",
    });
  });

  it("keeps a referential phrase in the message the rep is writing", () => {
    const intent = extractDictationAttachmentIntent(
      "Hey Amy, please see attached demo video of my app Destroy.",
    );
    // The sentence is prose the rep wants sent, so it survives intact — only
    // the file resolution is added.
    expect(intent?.messageText).toBe(
      "Hey Amy, please see attached demo video of my app Destroy.",
    );
    expect(intent?.kind).toBe("video");
    expect(intent?.query).toBe("demo video of my app Destroy");
  });

  it("requires a possessive for verbs that are ordinary sales vocabulary", () => {
    // "send"/"share" appear constantly in real dictation. Without "my"/"our"
    // they must not hijack the utterance and open a picker.
    expect(extractDictationAttachmentIntent("I'll send the proposal over tomorrow")).toBeNull();
    expect(extractDictationAttachmentIntent("let me share the deck with legal first")).toBeNull();
    expect(extractDictationAttachmentIntent("send my deck")).not.toBeNull();
    expect(extractDictationAttachmentIntent("share our one pager")).not.toBeNull();
  });

  it("accepts any determiner behind an unambiguous attach verb", () => {
    expect(extractDictationAttachmentIntent("attach the proposal")?.query).toBe("proposal");
    expect(extractDictationAttachmentIntent("include that case study")?.query).toBe("case study");
    // "add"/"grab"/"pull up" name the action as plainly as "attach" does, so a
    // shared team document ("the NDA") must resolve without a possessive.
    expect(extractDictationAttachmentIntent("add the NDA")?.query).toBe("NDA");
    expect(extractDictationAttachmentIntent("pull up the statement of work")?.query).toBe(
      "statement of work",
    );
  });

  it("recognises ordinary business paperwork, not only deck vocabulary", () => {
    // Each of these previously fell through to plain dictation, which pasted
    // the command into the field the rep was writing in.
    for (const utterance of [
      "add my resume",
      "attach my CV",
      "add my sales resume doc",
      "attach my invoice",
      "send my quote",
      "attach my Q3 report",
      "add my order form",
    ]) {
      expect(extractDictationAttachmentIntent(utterance), utterance).not.toBeNull();
    }
    expect(attachmentKindFor("sales resume doc")).toBe("document");
    expect(attachmentKindFor("mutual NDA")).toBe("document");
  });

  it("drops request scaffolding aimed at Destroy from the message", () => {
    // "can you" is spoken to Destroy, not to Amy; the command half is removed, so
    // leaving the preamble behind would paste a fragment the rep never wrote.
    expect(
      extractDictationAttachmentIntent("Hey Amy, can you attach my pitch deck")?.messageText,
    ).toBe("Hey Amy");
    expect(extractDictationAttachmentIntent("can you add my resume")?.messageText).toBe("");
  });

  it("ignores utterances with no file noun at all", () => {
    expect(extractDictationAttachmentIntent("attach this to the deal record")).toBeNull();
    expect(extractDictationAttachmentIntent("add a laughing gif")).toBeNull();
    expect(extractDictationAttachmentIntent("thanks so much for your time today")).toBeNull();
  });

  it("resolves the type from the most specific noun", () => {
    expect(attachmentKindFor("slide deck")).toBe("presentation");
    expect(attachmentKindFor("demo video")).toBe("video");
    expect(attachmentKindFor("q3 pricing model")).toBe("spreadsheet");
    expect(attachmentKindFor("signed contract")).toBe("document");
    expect(attachmentKindFor("screenshot of the dashboard")).toBe("image");
    expect(attachmentKindFor("the file")).toBe("any");
  });

  it("carries qualifiers into the query so ranking can use them", () => {
    const intent = extractDictationAttachmentIntent("attach my Q3 pricing sheet");
    expect(intent?.query).toBe("Q3 pricing sheet");
    expect(intent?.kind).toBe("spreadsheet");
  });

  it("refuses a query long enough to be a runaway transcript", () => {
    expect(
      extractDictationAttachmentIntent(`attach my deck ${"x".repeat(200)}`),
    ).toBeNull();
  });
});
