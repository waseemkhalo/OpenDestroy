import { describe, expect, it } from "vitest";
import { canRemoveModel, emptyPublicLocalModels, formatBytes, freeSpaceMessage, modelProgress, type PublicLocalModel } from "./localModels";

const model: PublicLocalModel = { id: "whisper-base-en", name: "Whisper Base · English", engine: "Whisper", language: "English", downloadBytes: 1000, installed: true, supported: true };

describe("local model helpers", () => {
  it("formats exact download sizes without pretending they are parameter counts", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1024 * 1024 * 1.5)).toBe("1.5 MiB");
  });

  it("derives progress only for the actively downloading catalog entry", () => {
    const state = { ...emptyPublicLocalModels(), downloadingModelId: model.id, downloadedBytes: 25, totalBytes: 100 };
    expect(modelProgress(state, model)).toBe(25);
    expect(modelProgress({ ...state, downloadingModelId: "other" }, model)).toBeNull();
  });

  it("allows repair removal when idle but blocks model changes during capture", () => {
    const state = { ...emptyPublicLocalModels(), selectedModelId: model.id };
    expect(canRemoveModel(state, model, false)).toBe(true);
    expect(canRemoveModel({ ...state, selectedModelId: "other" }, model, true)).toBe(false);
    expect(canRemoveModel({ ...state, selectedModelId: "other" }, model, false)).toBe(true);
  });

  it("explains insufficient storage using the real catalog size", () => {
    const state = { ...emptyPublicLocalModels(), availableBytes: 500 };
    expect(freeSpaceMessage(state, model)).toContain("500 B more");
    expect(freeSpaceMessage({ ...state, availableBytes: null }, model)).toContain("unavailable");
  });
});
