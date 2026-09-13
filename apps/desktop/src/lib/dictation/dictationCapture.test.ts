import { describe, expect, it, vi } from "vitest";

import {
  DictationEndpointDetector,
  keepAudioContextRunning,
  recoverDictationTranscription,
  transcribeDictationRecording,
} from "./dictationCapture";

describe("dictation capture reliability", () => {
  it("never finishes key-held dictation from a local microphone level guess", () => {
    const detector = new DictationEndpointDetector(false);
    for (const audioTime of [0, 4, 15, 60, 119]) {
      expect(detector.shouldFinish(0.001, audioTime, "running")).toBe(false);
    }
  });

  it("does not count a suspended or interrupted context as no speech", () => {
    const detector = new DictationEndpointDetector(true);
    expect(detector.shouldFinish(0, 0, "suspended")).toBe(false);
    expect(detector.shouldFinish(0, 30, "interrupted")).toBe(false);
    expect(detector.shouldFinish(0, 30, "running")).toBe(false);
    expect(detector.shouldFinish(0, 33.9, "running")).toBe(false);
    expect(detector.shouldFinish(0, 34, "running")).toBe(true);
  });

  it("lets quieter speech continue and finishes hands-free capture after a pause", () => {
    const detector = new DictationEndpointDetector(true);
    expect(detector.shouldFinish(0.006, 0, "running")).toBe(false);
    expect(detector.shouldFinish(0.006, 6, "running")).toBe(false);
    expect(detector.shouldFinish(0.0001, 7.4, "running")).toBe(false);
    expect(detector.shouldFinish(0.0001, 7.5, "running")).toBe(true);
  });

  it("resumes new and interrupted audio and detaches recovery on cancellation", async () => {
    const events = new EventTarget();
    const resume = vi.fn().mockResolvedValue(undefined);
    const context = Object.assign(events, { state: "suspended", resume });
    const dispose = keepAudioContextRunning(context as unknown as AudioContext);
    expect(resume).toHaveBeenCalledTimes(1);
    await new Promise<void>((resolve) => queueMicrotask(resolve));
    context.state = "running";
    events.dispatchEvent(new Event("statechange"));
    context.state = "interrupted";
    events.dispatchEvent(new Event("statechange"));
    expect(resume).toHaveBeenCalledTimes(2);
    dispose();
    await new Promise<void>((resolve) => queueMicrotask(resolve));
    context.state = "suspended";
    events.dispatchEvent(new Event("statechange"));
    expect(resume).toHaveBeenCalledTimes(2);
  });

  it("recovers an empty PCM transcript from the in-memory recording", async () => {
    const recorded = { text: "Keep the words I said quietly", path: "cloud" as const };
    const fallback = vi.fn().mockResolvedValue(recorded);
    await expect(recoverDictationTranscription({ text: "  ", path: "cloud" }, fallback)).resolves.toBe(recorded);
    expect(fallback).toHaveBeenCalledOnce();
  });

  it("avoids a second transcription when the live stream has text", async () => {
    const live = { text: "Hello", path: "cloud" as const };
    const fallback = vi.fn();
    await expect(recoverDictationTranscription(live, fallback)).resolves.toBe(live);
    expect(fallback).not.toHaveBeenCalled();
  });

  it("reports recording transcription failures instead of disguising them as silence", async () => {
    const fallback = vi.fn().mockRejectedValue(new Error("Transcription unavailable"));
    await expect(recoverDictationTranscription(null, fallback)).rejects.toThrow("Transcription unavailable");
  });

  it("never dispatches old audio when cancellation arrives during encoding", async () => {
    let cancelled = false;
    let finishEncoding!: (audio: string) => void;
    const encodeAudio = vi.fn(() => new Promise<string>((resolve) => { finishEncoding = resolve; }));
    const transcribe = vi.fn();
    const result = recoverDictationTranscription({ text: "", path: "cloud" }, () =>
      transcribeDictationRecording({ encodeAudio, isCancelled: () => cancelled, transcribe }),
    );
    cancelled = true; // Key cancellation or an account/team boundary.
    finishEncoding("old-account-audio");
    await expect(result).rejects.toThrow("Dictation was cancelled.");
    expect(transcribe).not.toHaveBeenCalled();
  });

  it("dispatches the recorded fallback after encoding while its session is current", async () => {
    const transcription = { text: "Recovered speech", path: "cloud" as const };
    const transcribe = vi.fn().mockResolvedValue(transcription);
    await expect(transcribeDictationRecording({
      encodeAudio: async () => "current-audio",
      isCancelled: () => false,
      transcribe,
    })).resolves.toBe(transcription);
    expect(transcribe).toHaveBeenCalledWith("current-audio");
  });
});
