import { describe, expect, it } from "vitest";
import { isBasicDictationReady, isSpeechProviderReady, onboardingContinueIssue, type PublicSpeechStatus } from "./publicSetup";

const shape: PublicSpeechStatus = {
  provider: "local", ready: true, modelReady: true, modelDownloading: false,
  openaiKeyPresent: false, geminiKeyPresent: false, user_id: "local-user",
  backend_url: "https://local.destroy.invalid",
};

describe("public setup readiness", () => {
  it("treats cloud and backend readiness independently of local model state", () => {
    for (const provider of ["openai", "gemini", "backend"] as const) {
      const status = { ...shape, provider, modelReady: false };
      expect(isSpeechProviderReady(status)).toBe(true);
      expect(isBasicDictationReady(status, true)).toBe(true);
    }
  });

  it("requires the local model and microphone for local dictation", () => {
    expect(isBasicDictationReady(shape, true)).toBe(true);
    expect(isBasicDictationReady({ ...shape, modelReady: false }, true)).toBe(false);
    expect(isBasicDictationReady({ ...shape, modelDownloading: true, modelReady: false }, true)).toBe(false);
    expect(isBasicDictationReady(shape, false)).toBe(false);
  });

  it("does not trust a cached ready flag while the local model is downloading", () => {
    expect(isSpeechProviderReady({ ...shape, modelDownloading: true })).toBe(false);
    expect(isSpeechProviderReady({ ...shape, provider: "openai", modelDownloading: true })).toBe(true);
  });

  it("explains why Continue cannot advance local setup", () => {
    expect(onboardingContinueIssue(0, { ...shape, ready: false, modelReady: false }, false))
      .toBe("Choose and install a local speech model before continuing.");
    expect(onboardingContinueIssue(0, { ...shape, ready: false, modelReady: false, modelDownloading: true }, false))
      .toBe("Wait for the local model to finish downloading.");
    expect(onboardingContinueIssue(0, { ...shape, ready: false, modelReady: false, error: "Model validation failed." }, false))
      .toBe("Model validation failed.");
  });

  it("requires the microphone on the Mac step but not Accessibility", () => {
    expect(onboardingContinueIssue(1, shape, false)).toBe("Allow microphone access before continuing.");
    expect(onboardingContinueIssue(1, shape, true)).toBeNull();
  });
});
