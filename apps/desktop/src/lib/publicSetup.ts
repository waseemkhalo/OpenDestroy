export type PublicSpeechProvider = "local" | "openai" | "gemini" | "backend";

export type PublicSpeechStatus = {
  provider: PublicSpeechProvider | null;
  ready: boolean;
  modelReady: boolean;
  modelDownloading?: boolean;
  localModelId?: string | null;
  openaiKeyPresent: boolean;
  geminiKeyPresent: boolean;
  user_id: string | null;
  backend_url: string;
  error?: string;
};

/** Basic dictation is available only after the worker has a usable identity,
 * model/provider, and a real microphone grant. Accessibility is deliberately
 * not required: the native delivery path can copy for the user to paste. */
export function isBasicDictationReady(status: PublicSpeechStatus, microphoneAllowed: boolean): boolean {
  return isSpeechProviderReady(status) && Boolean(status.user_id) && microphoneAllowed;
}

/** `modelReady` is meaningful only for the local provider. Cloud and backend
 * workers report it as false while still being ready to transcribe. */
export function isSpeechProviderReady(status: PublicSpeechStatus): boolean {
  return status.ready &&
    (status.provider !== "local" || (status.modelReady && status.modelDownloading !== true));
}

/** Explain why the current onboarding step cannot advance after native status
 * has been refreshed. Accessibility is intentionally optional because users
 * can paste copied text when direct insertion is unavailable. */
export function onboardingContinueIssue(
  step: number,
  status: PublicSpeechStatus,
  microphoneAllowed: boolean,
): string | null {
  if (step === 0 && !isSpeechProviderReady(status)) {
    if (status.error) return status.error;
    if (status.provider === "local") {
      return status.modelDownloading
        ? "Wait for the local model to finish downloading."
        : "Choose and install a local speech model before continuing.";
    }
    if (status.provider === "openai" || status.provider === "gemini") {
      return "Save your speech provider API key before continuing.";
    }
    return "Choose a speech provider and finish its setup before continuing.";
  }
  if (step === 1 && !status.ready) {
    return "Finish speech setup before continuing.";
  }
  if (step === 1 && !microphoneAllowed) {
    return "Allow microphone access before continuing.";
  }
  return null;
}

export function usesNativeBatch(provider: PublicSpeechProvider | null): boolean {
  return provider !== "backend";
}
