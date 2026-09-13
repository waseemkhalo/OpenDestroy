import { invoke } from "@tauri-apps/api/core";

export type MediaDeviceOption = {
  deviceId: string;
  label: string;
  isDefault?: boolean;
};

export type VoiceFocusProfile = "near-field" | "far-field";

export type AudioOutputDeviceOption = {
  id: string;
  name: string;
  is_default: boolean;
};

export type AudioInputDeviceOption = AudioOutputDeviceOption;

export type AudioDevicePreferences = {
  microphone_device_id: string | null;
  speaker_device_id: string | null;
};

/** Notch listens so an active call can hot-swap mic / system-audio tap. */
export const AUDIO_DEVICES_CHANGED_EVENT = "destroy://audio-devices-changed";

export type AudioDeviceChange = {
  kind: "microphone" | "speaker";
  /** The saved selection at the time of the event; null means System default. */
  microphone_device_id?: string | null;
};

const DEFAULT_MIC_CONSTRAINTS: MediaTrackConstraints = {
  // WebKit's voice-processing capture can attenuate the microphone heard by
  // other calling apps. Destroy only needs clean PCM for transcription, so its
  // compatibility path must not opt the shared input into that processing.
  echoCancellation: false,
  noiseSuppression: false,
  autoGainControl: false,
  channelCount: 1,
};

/** Constraints for live-call / dictation capture, honoring a preferred mic when set. */
export function micAudioConstraints(
  deviceId?: string | null,
): boolean | MediaTrackConstraints {
  const trimmed = deviceId?.trim();
  if (!trimmed) {
    return { ...DEFAULT_MIC_CONSTRAINTS };
  }
  return {
    ...DEFAULT_MIC_CONSTRAINTS,
    // Prefer exact so Settings/meeting selection is honored; callers should
    // surface an error via `openMicrophoneStream` when the device is gone.
    deviceId: { exact: trimmed },
  };
}

export async function ensureMicrophonePermission(): Promise<boolean> {
  if (!navigator.mediaDevices?.getUserMedia) return false;
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: micAudioConstraints(null) });
    stream.getTracks().forEach((track) => track.stop());
    return true;
  } catch {
    return false;
  }
}

/**
 * Native AVFoundation device IDs are not guaranteed to be valid WebKit
 * `deviceId` constraints. That matters when native capture is unavailable and
 * Destroy has to use its compatibility microphone path. Do not replace an explicit selection with WebKit's default input.
 * Surface device/constraint failures with a recovery instruction; permission
 * failures retain their original error.
 */
export function shouldRetryPreferredMicWithDefault(cause: unknown): boolean {
  const error = cause as { name?: unknown; message?: unknown } | null;
  const name = typeof error?.name === "string" ? error.name : "";
  const message = typeof error?.message === "string"
    ? error.message
    : typeof cause === "string" ? cause : "";

  return name === "OverconstrainedError"
    || name === "NotFoundError"
    || /invalid\s+constraint|overconstrained|device\s*id/i.test(message);
}

/** Resolve the same physical input across the native and WebKit ID namespaces. */
export function matchingWebMicrophoneId(
  preferredId: string,
  nativeDevices: AudioInputDeviceOption[],
  webDevices: Pick<MediaDeviceInfo, "kind" | "deviceId" | "label">[],
): string | null {
  const inputDevices = webDevices.filter((device) => device.kind === "audioinput");
  if (inputDevices.some((device) => device.deviceId === preferredId)) return preferredId;
  const selected = nativeDevices.find((device) => device.id === preferredId);
  const name = selected?.name.trim().toLocaleLowerCase();
  if (!name) return null;
  // Device names are not guaranteed identifiers. Only bridge an exact,
  // unambiguous name on both sides; duplicate USB inputs remain an error.
  if (nativeDevices.filter((device) => device.name.trim().toLocaleLowerCase() === name).length !== 1) {
    return null;
  }
  const matches = inputDevices.filter((device) =>
    device.deviceId
    && device.deviceId !== "default"
    && device.deviceId !== "communications"
    && device.label.trim().toLocaleLowerCase() === name,
  );
  return matches.length === 1 ? matches[0].deviceId : null;
}

async function preferredWebMicrophoneId(preferredId: string): Promise<string | null> {
  if (!navigator.mediaDevices?.enumerateDevices) return null;
  try {
    const [nativeDevices, webDevices] = await Promise.all([
      invoke<AudioInputDeviceOption[]>("list_audio_input_devices"),
      navigator.mediaDevices.enumerateDevices(),
    ]);
    return matchingWebMicrophoneId(preferredId, nativeDevices, webDevices);
  } catch {
    return null;
  }
}

/** Opens a low-processing mic stream, honoring the preferred device when possible. */
export async function openMicrophoneStream(
  preferredDeviceId?: string | null,
): Promise<MediaStream> {
  if (!navigator.mediaDevices?.getUserMedia) {
    throw new Error("Microphone capture is not available in this window.");
  }
  const preferred = preferredDeviceId?.trim() || null;
  try {
    return await navigator.mediaDevices.getUserMedia({
      audio: micAudioConstraints(preferred),
    });
  } catch (cause) {
    if (!preferred || !shouldRetryPreferredMicWithDefault(cause)) throw cause;
    const webDeviceId = await preferredWebMicrophoneId(preferred);
    if (webDeviceId && webDeviceId !== preferred) {
      // This retry still requires the user-selected physical microphone. It
      // never opens the default input as a permission or discovery probe.
      return navigator.mediaDevices.getUserMedia({ audio: micAudioConstraints(webDeviceId) });
    }
    throw new Error(
      "The selected microphone cannot be opened. Choose an available microphone or System default in Destroy Settings; Destroy has not switched inputs.",
      { cause },
    );
  }
}

/** Human label for the track actually opened (may differ from the preference). */
export function activeMicrophoneLabel(stream: MediaStream): string {
  const track = stream.getAudioTracks()[0];
  const label = track?.label?.trim();
  return label || "Microphone";
}

/** Lists microphones. Labels require prior mic permission. */
export async function listMicrophoneDevices(): Promise<MediaDeviceOption[]> {
  try {
    const devices = await invoke<AudioInputDeviceOption[]>("list_audio_input_devices");
    return devices.map((device) => ({
      deviceId: device.id,
      label: device.name,
      isDefault: device.is_default,
    }));
  } catch {
    // Compatibility only for non-macOS/dev builds. Production macOS calls use
    // native device IDs so selection and capture resolve the same exact input.
  }
  if (!navigator.mediaDevices?.enumerateDevices) return [];
  await ensureMicrophonePermission();
  const devices = await navigator.mediaDevices.enumerateDevices();
  const seen = new Set<string>();
  const mics: MediaDeviceOption[] = [];
  for (const device of devices) {
    if (device.kind !== "audioinput") continue;
    if (!device.deviceId || seen.has(device.deviceId)) continue;
    // Skip the synthetic "default" / "communications" aliases — we expose
    // "System default" as an explicit empty selection instead.
    if (device.deviceId === "default" || device.deviceId === "communications") continue;
    seen.add(device.deviceId);
    mics.push({
      deviceId: device.deviceId,
      label: device.label?.trim() || `Microphone ${mics.length + 1}`,
    });
  }
  return mics;
}

export async function listSpeakerDevices(): Promise<AudioOutputDeviceOption[]> {
  try {
    return await invoke<AudioOutputDeviceOption[]>("list_audio_output_devices");
  } catch {
    return [];
  }
}

export async function loadAudioDevicePreferences(): Promise<AudioDevicePreferences> {
  try {
    const prefs = await invoke<AudioDevicePreferences & Record<string, unknown>>(
      "get_app_preferences",
    );
    return {
      microphone_device_id: prefs.microphone_device_id ?? null,
      speaker_device_id: prefs.speaker_device_id ?? null,
    };
  } catch {
    return { microphone_device_id: null, speaker_device_id: null };
  }
}

export async function saveMicrophoneDeviceId(
  deviceId: string | null,
): Promise<AudioDevicePreferences> {
  const prefs = await invoke<AudioDevicePreferences>("set_microphone_device_id", {
    deviceId: deviceId?.trim() || null,
  });
  return {
    microphone_device_id: prefs.microphone_device_id ?? null,
    speaker_device_id: prefs.speaker_device_id ?? null,
  };
}

export async function saveSpeakerDeviceId(
  deviceId: string | null,
): Promise<AudioDevicePreferences> {
  const prefs = await invoke<AudioDevicePreferences>("set_speaker_device_id", {
    deviceId: deviceId?.trim() || null,
  });
  return {
    microphone_device_id: prefs.microphone_device_id ?? null,
    speaker_device_id: prefs.speaker_device_id ?? null,
  };
}

export async function preferredMicrophoneDeviceId(): Promise<string | null> {
  const prefs = await loadAudioDevicePreferences();
  return prefs.microphone_device_id;
}

/**
 * Voice Focus is intentionally opt-in only when the device name describes a
 * clear acoustic setup. Unknown USB/virtual microphones stay unprocessed; a
 * wrong near/far model can remove the rep's own voice.
 */
export function voiceFocusProfileForMicrophoneLabel(
  label: string | null | undefined,
): VoiceFocusProfile | null {
  const normalized = label?.trim().toLocaleLowerCase() ?? "";
  if (!normalized) return null;
  if (
    /airpods|earpods|earbuds|headset|headphones|handset|lavalier|lapel|boom mic|modmic|jabra|plantronics|poly voyager|shokz/.test(
      normalized,
    )
  ) {
    return "near-field";
  }
  if (
    /macbook|built-in|internal microphone|studio display|conference|room mic|webcam|camera microphone|continuity camera/.test(
      normalized,
    )
  ) {
    return "far-field";
  }
  return null;
}

/** Resolves the selected or current default native mic before STT connects. */
export async function recommendedVoiceFocusForMicrophone(
  preferredDeviceId: string | null,
): Promise<VoiceFocusProfile | null> {
  const devices = await listMicrophoneDevices();
  const preferred = preferredDeviceId?.trim();
  const resolved = preferred
    ? devices.find((device) => device.deviceId === preferred)
    : devices.find((device) => device.isDefault);
  return voiceFocusProfileForMicrophoneLabel(resolved?.label);
}
