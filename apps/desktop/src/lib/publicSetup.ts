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

export function usesNativeBatch(provider: PublicSpeechProvider | null): boolean {
  return provider !== "backend";
}
