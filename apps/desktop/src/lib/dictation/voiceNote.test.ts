import { describe, expect, it } from "vitest";
import { isVoiceNoteCommand } from "./voiceNote";

describe("voice note command", () => {
  it("requires a standalone mode request", () => {
    expect(isVoiceNoteCommand("record a voice note")).toBe(true);
    expect(isVoiceNoteCommand("I received a voice note yesterday")).toBe(false);
  });
});
