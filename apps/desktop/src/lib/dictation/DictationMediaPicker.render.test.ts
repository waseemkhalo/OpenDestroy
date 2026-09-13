import { render } from "svelte/server";
import { describe, expect, it } from "vitest";

import DictationMediaPicker from "./DictationMediaPicker.svelte";
import type { DictationMediaResult } from "./giphy";

const RESULTS: DictationMediaResult[] = [
  {
    id: "gif-1",
    title: "Celebrate",
    alt_text: "Celebration GIF",
    preview_url: "https://media.giphy.com/preview-1.gif",
    content_url: "https://media.giphy.com/content-1.gif",
    source_url: "https://giphy.com/gifs/gif-1",
    width: 240,
    height: 160,
    kind: "gif",
  },
  {
    id: "gif-2",
    title: "Applause",
    alt_text: "Applause GIF",
    preview_url: "https://media.giphy.com/preview-2.gif",
    content_url: "https://media.giphy.com/content-2.gif",
    source_url: "https://giphy.com/gifs/gif-2",
    width: 240,
    height: 160,
    kind: "gif",
  },
];

function base() {
  return {
    kind: "gif" as const,
    query: "laughing",
    leadingText: "",
    page: 0,
    results: RESULTS,
    selectedIndex: 0,
    loading: false,
    delivering: false,
    error: "",
    attribution: "Powered by GIPHY",
    onKindChange: () => {},
    onSelect: () => {},
    onFavorite: () => {},
    onRotate: () => {},
    onCancel: () => {},
  };
}

describe("DictationMediaPicker", () => {
  it("offers a way out of three results the rep does not like", () => {
    const { body } = render(DictationMediaPicker, { props: base() });
    expect(body).toContain("Show different results");
    // Cancelling and re-dictating the whole command must not be the only path
    // to a different set, so the shortcut is stated where the rep is looking.
    expect(body).toContain("R more");
  });

  it("does not invite a second rotation while one is in flight", () => {
    const { body } = render(DictationMediaPicker, {
      props: { ...base(), results: [], loading: true },
    });
    expect(body).toContain("disabled");
    expect(body).toContain("Finding three choices");
  });

  it("says a rotation is what it is waiting on", () => {
    const { body } = render(DictationMediaPicker, {
      props: { ...base(), results: [], loading: true, page: 2 },
    });
    expect(body).toContain("Finding three different choices");
  });
});
