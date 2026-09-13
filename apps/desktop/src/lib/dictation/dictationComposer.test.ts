import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  currentDictationPreferences,
  extractTerminalRewriteCommand,
  hasLoadedDictationPreferences,
  isPreviousDictationCorrection,
  isUndoDictationCommand,
  loadDictationPreferences,
  resetDictationComposerCache,
} from "./dictationComposer";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

beforeEach(() => {
  resetDictationComposerCache();
  vi.mocked(invoke).mockReset();
});

describe("dictation correction commands", () => {
  it("keeps undo explicit", () => {
    expect(isUndoDictationCommand("undo that")).toBe(true);
    expect(isUndoDictationCommand("scratch that")).toBe(true);
    expect(isUndoDictationCommand("undo is useful")).toBe(false);
  });

  it("recognizes a correction without capturing ordinary prose", () => {
    expect(isPreviousDictationCorrection("change Tuesday to Wednesday")).toBe(true);
    expect(isPreviousDictationCorrection("in that, replace 2 PM with 3 PM")).toBe(true);
    expect(isPreviousDictationCorrection("Change is hard")).toBe(false);
  });
});

describe("explicit rewrite commands", () => {
  it("extracts a terminal rewrite command without pasting the command", () => {
    expect(extractTerminalRewriteCommand("Here are all the details I want to keep, rewrite"))
      .toBe("Here are all the details I want to keep");
    expect(extractTerminalRewriteCommand("Keep the context and make this sound smoother please rewrite."))
      .toBe("Keep the context and make this sound smoother");
  });

  it("does not mistake ordinary uses of rewrite for a command", () => {
    expect(extractTerminalRewriteCommand("This is a rewrite")).toBeNull();
    expect(extractTerminalRewriteCommand("We discussed the rewrite")).toBeNull();
    expect(extractTerminalRewriteCommand("I would like to rewrite")).toBeNull();
    expect(extractTerminalRewriteCommand("Rewrite this paragraph tomorrow")).toBeNull();
  });
});

describe("team-scoped dictation preferences", () => {
  it("never exposes one team's writing style through another team's cache", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({
        self_correction: true,
        remove_fillers: true,
        app_formatting: true,
        selected_text_editing: true,
        language: "en",
        style_note: "Team A private style",
      })
      .mockResolvedValueOnce({
        self_correction: false,
        remove_fillers: false,
        app_formatting: false,
        selected_text_editing: false,
        language: "fr",
        style_note: "Team B private style",
      });

    await loadDictationPreferences("team-a");
    expect(currentDictationPreferences("team-b").style_note).toBe("");
    expect(hasLoadedDictationPreferences("team-b")).toBe(false);

    await loadDictationPreferences("team-b");
    expect(currentDictationPreferences("team-a").style_note).toBe("Team A private style");
    expect(currentDictationPreferences("team-b").style_note).toBe("Team B private style");
  });
});
