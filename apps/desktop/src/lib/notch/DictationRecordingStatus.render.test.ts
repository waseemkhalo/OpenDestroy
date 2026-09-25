import { render } from "svelte/server";
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";

import DictationRecordingStatus from "./DictationRecordingStatus.svelte";

describe("DictationRecordingStatus", () => {
  it("keeps listening and destination content on opposite sides of the camera gutter", () => {
    const { body } = render(DictationRecordingStatus, {
      props: {
        target: {
          canPaste: true,
          appName: "ChatGPT",
          bundleId: "com.openai.chat",
          appKind: "generic",
          appIconDataUrl: "data:image/png;base64,c2t5",
        },
      },
    });

    expect(body).toContain("dictation-flank--input");
    expect(body).toContain("dictation-camera-gutter");
    expect(body).toContain("dictation-flank--destination");
    expect(body).toContain("Dictating into ChatGPT");
    expect(body).toContain("ChatGPT");
    expect(body).toContain('dictation-app-name');
    expect(body).toContain('data-native-app-icon="true"');
    expect(body).toContain("data:image/png;base64,c2t5");
  });

  it("uses equal flanks around a runtime camera gutter", () => {
    const css=readFileSync(new URL("./DictationHud.svelte",import.meta.url),"utf8");
    expect(css).toContain("grid-template-columns:128px var(--camera-width,200px) 128px");
    expect(css).toContain("width:calc(var(--camera-width,200px) + 256px)");
    expect(css).toContain("height:max(46px,var(--camera-height,34px))");
  });

  it("renders recording warnings in the notch status", () => {
    const warning = "About 15 seconds left. Release your shortcut to finish dictation.";
    const { body } = render(DictationRecordingStatus, { props: { warning } });
    expect(body).toContain(`aria-label="${warning}"`);
    expect(body).toContain(`title="${warning}"`);
    expect(body).toContain('data-warning="true"');
  });
});
