const SPEECH_THRESHOLD = 0.003;
const NO_SPEECH_SECONDS = 4;
const POST_SPEECH_SILENCE_SECONDS = 1.5;

/**
 * Hands-free endpointing only. Hold-to-dictate belongs to key-up and must never
 * discard a quiet recording based on a local level estimate. Use the audio
 * clock so an interrupted/suspended context cannot be mistaken for silence.
 */
export class DictationEndpointDetector {
  private startedAt: number | null = null;
  private lastSpeechAt: number | null = null;

  constructor(private readonly autoStop: boolean) {}

  shouldFinish(rms: number, audioTimeSeconds: number, contextState: string): boolean {
    if (!this.autoStop || contextState !== "running" || !Number.isFinite(audioTimeSeconds)) return false;
    this.startedAt ??= audioTimeSeconds;
    if (rms > SPEECH_THRESHOLD) {
      this.lastSpeechAt = audioTimeSeconds;
      return false;
    }
    return this.lastSpeechAt === null
      ? audioTimeSeconds - this.startedAt >= NO_SPEECH_SECONDS
      : audioTimeSeconds - this.lastSpeechAt >= POST_SPEECH_SILENCE_SECONDS;
  }
}

/** Resume newly suspended or interrupted WebKit audio without reviving a closed session. */
export function keepAudioContextRunning(context: AudioContext): () => void {
  let disposed = false;
  let resuming = false;
  const resume = () => {
    if (disposed || resuming || context.state === "running" || context.state === "closed") return;
    resuming = true;
    void context.resume().catch(() => {}).finally(() => { resuming = false; });
  };
  context.addEventListener("statechange", resume);
  resume();
  return () => {
    disposed = true;
    context.removeEventListener("statechange", resume);
  };
}

export type DictationTranscription = { text: string; path: "local" | "cloud" };

/** A live socket may return empty text when PCM was interrupted while MediaRecorder kept audio. */
export async function recoverDictationTranscription(
  live: DictationTranscription | null,
  transcribeRecording: () => Promise<DictationTranscription>,
): Promise<DictationTranscription> {
  return live?.text.trim() ? live : transcribeRecording();
}

/** Encoding yields to the event loop; cancellation must win before audio egress. */
export async function transcribeDictationRecording(options: {
  encodeAudio: () => Promise<string>;
  isCancelled: () => boolean;
  transcribe: (audioBase64: string) => Promise<DictationTranscription>;
}): Promise<DictationTranscription> {
  if (options.isCancelled()) throw new Error("Dictation was cancelled.");
  const audioBase64 = await options.encodeAudio();
  if (options.isCancelled()) throw new Error("Dictation was cancelled.");
  return options.transcribe(audioBase64);
}
