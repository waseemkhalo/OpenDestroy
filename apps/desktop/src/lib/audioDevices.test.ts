import { afterEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";

import {
  micAudioConstraints,
  ensureMicrophonePermission,
  openMicrophoneStream,
  voiceFocusProfileForMicrophoneLabel,
  shouldRetryPreferredMicWithDefault,
  matchingWebMicrophoneId,
} from "./audioDevices";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

afterEach(() => {
  vi.unstubAllGlobals();
  vi.mocked(invoke).mockReset();
});

describe("live-call microphone constraints", () => {
  it("does not engage WebKit voice processing that can attenuate other call apps", () => {
    expect(micAudioConstraints(null)).toMatchObject({
      echoCancellation: false,
      noiseSuppression: false,
      autoGainControl: false,
      channelCount: 1,
    });
  });

  it("requires the selected microphone instead of silently falling back", () => {
    expect(micAudioConstraints("chosen-input")).toMatchObject({
      deviceId: { exact: "chosen-input" },
      echoCancellation: false,
      noiseSuppression: false,
      autoGainControl: false,
    });
  });

  it("opens the current system default on every unconfigured dictation", async () => {
    const getUserMedia = vi.fn().mockResolvedValue({});
    vi.stubGlobal("navigator", { mediaDevices: { getUserMedia } });
    await openMicrophoneStream(null);
    await openMicrophoneStream();
    for (const [constraints] of getUserMedia.mock.calls) {
      expect(constraints.audio).not.toHaveProperty("deviceId");
    }
    expect(invoke).not.toHaveBeenCalled();
  });

  it("opens the selected native microphone using its WebKit device ID", async () => {
    const stream = { getAudioTracks: () => [{ label: "MacBook Pro Microphone" }] };
    const getUserMedia = vi.fn()
      .mockRejectedValueOnce({ name: "OverconstrainedError" })
      .mockResolvedValueOnce(stream);
    vi.mocked(invoke).mockResolvedValue([
      { id: "native-mic", name: "MacBook Pro Microphone", is_default: false },
    ]);
    const enumerateDevices = vi.fn().mockResolvedValue([
      { kind: "audioinput", deviceId: "webkit-mic", label: "MacBook Pro Microphone" },
    ]);
    vi.stubGlobal("navigator", { mediaDevices: { getUserMedia, enumerateDevices } });

    await expect(openMicrophoneStream("native-mic")).resolves.toBe(stream);
    expect(getUserMedia).toHaveBeenLastCalledWith({
      audio: expect.objectContaining({ deviceId: { exact: "webkit-mic" } }),
    });
    expect(getUserMedia).toHaveBeenCalledTimes(2);
  });

  it("refuses ambiguous device names and synthetic default aliases", () => {
    const native = [{ id: "native-usb", name: "USB Audio", is_default: false }];
    const web = [{ kind: "audioinput" as const, deviceId: "web-usb", label: "USB Audio" }];
    expect(matchingWebMicrophoneId("native-usb", native, web)).toBe("web-usb");
    expect(matchingWebMicrophoneId("native-usb", native, [
      ...web, { ...web[0], deviceId: "second-usb" },
    ])).toBeNull();
    expect(matchingWebMicrophoneId("native-usb", [
      ...native, { ...native[0], id: "another-native-usb" },
    ], web)).toBeNull();
    expect(matchingWebMicrophoneId("native-usb", native, [
      { ...web[0], deviceId: "default" },
    ])).toBeNull();
    expect(matchingWebMicrophoneId("native-usb", native, [
      { ...web[0], label: "" },
    ])).toBeNull();
  });

  it("recognizes WebKit's invalid native device constraint", () => {
    expect(shouldRetryPreferredMicWithDefault(new TypeError("Invalid constraint"))).toBe(true);
    expect(shouldRetryPreferredMicWithDefault({ name: "OverconstrainedError" })).toBe(true);
    expect(shouldRetryPreferredMicWithDefault({ name: "NotAllowedError" })).toBe(false);
  });

  it("does not silently switch an unavailable selected microphone", async () => {
    const getUserMedia = vi.fn().mockRejectedValue(new TypeError("Invalid constraint"));
    vi.stubGlobal("navigator", { mediaDevices: { getUserMedia } });
    await expect(openMicrophoneStream("native-avfoundation-id")).rejects.toThrow("Destroy has not switched inputs");
    expect(getUserMedia).toHaveBeenCalledTimes(1);
    expect(getUserMedia.mock.calls[0]?.[0]).toMatchObject({
      audio: { deviceId: { exact: "native-avfoundation-id" } },
    });
  });

  it("releases an unprocessed permission probe immediately", async () => {
    const stop = vi.fn();
    const getUserMedia = vi.fn().mockResolvedValue({ getTracks: () => [{ stop }] });
    vi.stubGlobal("navigator", { mediaDevices: { getUserMedia } });
    await expect(ensureMicrophonePermission()).resolves.toBe(true);
    expect(stop).toHaveBeenCalledOnce();
    expect(getUserMedia).toHaveBeenCalledWith({audio: expect.objectContaining({echoCancellation:false,noiseSuppression:false,autoGainControl:false})});
  });

  it("does not hide microphone permission failures", async () => {
    const denied = { name: "NotAllowedError", message: "Permission denied" };
    const getUserMedia = vi.fn().mockRejectedValue(denied);
    vi.stubGlobal("navigator", { mediaDevices: { getUserMedia } });

    await expect(openMicrophoneStream("chosen-input")).rejects.toBe(denied);
    expect(getUserMedia).toHaveBeenCalledTimes(1);
  });

  it("selects Voice Focus only for unambiguous microphone positions", () => {
    expect(voiceFocusProfileForMicrophoneLabel("MacBook Pro Microphone")).toBe("far-field");
    expect(voiceFocusProfileForMicrophoneLabel("AirPods Pro Microphone")).toBe("near-field");
    expect(voiceFocusProfileForMicrophoneLabel("Acme USB Audio Device")).toBeNull();
  });
});
