import { describe, expect, it } from "vitest";

import { dictationDeliveryFeedback } from "./accessibilityFeedback";

describe("dictationDeliveryFeedback", () => {
  it("shows the Accessibility action once when focused-field insertion needs it", () => {
    expect(dictationDeliveryFeedback("needs_accessibility", false)).toEqual({
      hint: "accessibility",
      timeoutMs: 9_000,
      accessibilityGuidanceShown: true,
    });
  });

  it("uses the honest clipboard result instead of repeating the permission CTA", () => {
    expect(dictationDeliveryFeedback("needs_accessibility", true)).toEqual({
      hint: "clipboard",
      timeoutMs: 2_400,
      accessibilityGuidanceShown: true,
    });
  });

  it("does not consume the one-time guidance for unrelated clipboard fallbacks", () => {
    expect(dictationDeliveryFeedback("no_focused_field", false)).toEqual({
      hint: "clipboard",
      timeoutMs: 2_400,
      accessibilityGuidanceShown: false,
    });
  });
});
