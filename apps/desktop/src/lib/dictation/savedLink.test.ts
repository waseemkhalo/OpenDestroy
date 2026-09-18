import { describe, expect, it } from "vitest";

import { parseSavedLink, savedLinkFromDraft, savedLinkRejectionMessage } from "./savedLink";

function draft(input: string) {
  const parsed = parseSavedLink(input);
  if (!parsed.ok) throw new Error(`expected a draft, got ${parsed.reason}`);
  return parsed.draft;
}

describe("parseSavedLink recognition", () => {
  it("reads the product from a Google share URL without inventing a title", () => {
    const slides = draft("https://docs.google.com/presentation/d/1AbCdEf/edit?usp=sharing");
    expect(slides.source).toBe("Google Slides");
    expect(slides.kind).toBe("presentation");
    // The title is simply not in the URL. Guessing one puts a wrong name in
    // front of a customer, so the form asks instead.
    expect(slides.name).toBe("");
    expect(slides.keywords).toContain("deck");

    expect(draft("https://docs.google.com/spreadsheets/d/1x/edit").kind).toBe("spreadsheet");
    expect(draft("https://docs.google.com/document/d/1x/edit").kind).toBe("document");
    expect(draft("https://drive.google.com/file/d/1x/view").source).toBe("Google Drive");
  });

  it("takes the title from providers that put one in the path", () => {
    expect(draft("https://www.dropbox.com/s/abc/Q3%20Pitch%20Deck.pdf?dl=0")).toMatchObject({
      source: "Dropbox",
      kind: "pdf",
      name: "Q3 Pitch Deck",
    });
    expect(draft("https://www.notion.so/Quarterly-Plan-2f1a3b4c5d6e7f8a9b0c1d2e3f4a5b6c")).toMatchObject({
      source: "Notion",
      name: "Quarterly Plan",
    });
    expect(draft("https://www.figma.com/design/k3y/Onboarding-Flow?node-id=1").name).toBe(
      "Onboarding Flow",
    );
  });

  it("names a plain file URL from its filename and kind from its extension", () => {
    expect(draft("https://example.com/files/demo_video.mp4")).toMatchObject({
      kind: "video",
      name: "demo video",
    });
  });

  it("leaves a routing path unnamed rather than titling it from the last segment", () => {
    // "/pricing/enterprise" is a page, not a file; calling it "enterprise"
    // would put a confident wrong name in the catalog.
    const page = draft("https://example.com/pricing/enterprise");
    expect(page.name).toBe("");
    expect(page.kind).toBe("any");
    expect(page.source).toBe("Link");
  });

  it("recognizes video hosts that carry no title", () => {
    expect(draft("https://www.loom.com/share/abc123")).toMatchObject({ kind: "video", name: "" });
    expect(draft("https://youtu.be/abc123").kind).toBe("video");
  });
});

describe("parseSavedLink validation", () => {
  it("adds a missing scheme to a pasted host", () => {
    expect(draft("docs.google.com/document/d/1x/edit").url).toBe(
      "https://docs.google.com/document/d/1x/edit",
    );
  });

  it("refuses an explicit http link instead of silently upgrading it", () => {
    const parsed = parseSavedLink("http://example.com/deck.pdf");
    expect(parsed).toEqual({ ok: false, reason: "insecure" });
    expect(savedLinkRejectionMessage("insecure")).toContain("https");
  });

  it("refuses a credential-bearing URL, as the backend does", () => {
    expect(parseSavedLink("https://user:secret@example.com/deck.pdf")).toEqual({
      ok: false,
      reason: "credentials",
    });
  });

  it("refuses other schemes and non-URLs", () => {
    expect(parseSavedLink("file:///deck.pdf").ok).toBe(false);
    expect(parseSavedLink("javascript:alert(1)").ok).toBe(false);
    expect(parseSavedLink("just some words").ok).toBe(false);
    expect(parseSavedLink("   ")).toEqual({ ok: false, reason: "empty" });
  });

  it("survives a malformed percent escape", () => {
    expect(parseSavedLink("https://example.com/%E0%A4%A.pdf").ok).toBe(true);
  });
});

describe("savedLinkFromDraft", () => {
  it("requires a name, because an unnamed link is unreachable by voice", () => {
    const result = savedLinkFromDraft(draft("https://docs.google.com/document/d/1x/edit"), "  ", "");
    expect(result).toEqual({ ok: false, message: expect.stringContaining("name") });
  });

  it("omits keywords rather than saving an empty string", () => {
    const result = savedLinkFromDraft(draft("https://example.com/a.pdf"), "Pricing", "   ");
    expect(result).toEqual({ ok: true, link: { name: "Pricing", url: "https://example.com/a.pdf" } });
  });

  it("keeps the name and keywords inside the backend's limits", () => {
    const result = savedLinkFromDraft(
      draft("https://example.com/a.pdf"),
      "n".repeat(500),
      `${"k".repeat(2000)}`,
    );
    if (!result.ok) throw new Error("expected a link");
    expect(result.link.name).toHaveLength(200);
    expect(result.link.keywords).toHaveLength(1000);
  });

  it("measures those limits in bytes, as the backend does", () => {
    const bytes = (value: string) => new TextEncoder().encode(value).length;
    // 200 three-byte characters is 600 bytes: within a code-unit limit, well
    // past the backend's, which would reject the whole link.
    const result = savedLinkFromDraft(draft("https://example.com/a.pdf"), "\u6587".repeat(200), "");
    if (!result.ok) throw new Error("expected a link");
    expect(bytes(result.link.name)).toBeLessThanOrEqual(200);

    // Truncation lands on a code point, never inside one.
    const emoji = savedLinkFromDraft(draft("https://example.com/a.pdf"), "\u{1F600}".repeat(80), "");
    if (!emoji.ok) throw new Error("expected a link");
    expect(bytes(emoji.link.name)).toBeLessThanOrEqual(200);
    expect(emoji.link.name).not.toMatch(/[\uD800-\uDFFF]$/u);
  });
});
