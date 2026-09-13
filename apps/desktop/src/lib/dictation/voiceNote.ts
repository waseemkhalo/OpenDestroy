export function isVoiceNoteCommand(value: string): boolean {
  return /^(?:please\s+)?(?:(?:start|record|add|insert|send)\s+)?(?:a\s+)?voice\s+note[.!?]*$/iu.test(
    value.trim(),
  );
}
