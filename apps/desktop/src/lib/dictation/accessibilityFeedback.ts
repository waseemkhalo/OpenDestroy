export type DictationDeliveryFeedback = {
  hint: "accessibility" | "clipboard";
  timeoutMs: number;
  accessibilityGuidanceShown: boolean;
};

/**
 * Explain a missing Accessibility grant once per app process. Every later
 * dictation truthfully reports the clipboard fallback without creating a
 * permission-loop experience.
 */
export function dictationDeliveryFeedback(
  reason: string | null,
  accessibilityGuidanceShown: boolean,
): DictationDeliveryFeedback {
  if (reason === "needs_accessibility" && !accessibilityGuidanceShown) {
    return {
      hint: "accessibility",
      timeoutMs: 9_000,
      accessibilityGuidanceShown: true,
    };
  }

  return {
    hint: "clipboard",
    timeoutMs: 2_400,
    accessibilityGuidanceShown,
  };
}
