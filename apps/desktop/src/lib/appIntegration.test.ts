import { describe, expect, it, vi } from "vitest";
import { readFileSync } from "node:fs";
import {
  clearScopedLocalStorage,
  isCurrentFencedWork,
  localResetSucceeded,
  practiceUsesLiveStream,
  type ConnectionOwner,
} from "./appIntegration";

const owner: ConnectionOwner = {
  expectedUserId: "local-user",
  expectedBackendUrl: "https://local.destroy.invalid",
  revision: 4,
};

describe("App integration fences", () => {
  it("fits only Home to its available height while detail tabs remain scrollable", () => {
    const source = readFileSync(new URL("./HomeScreen.svelte", import.meta.url), "utf8");
    expect(source).toContain("class:home-fit={view==='home'}");
    expect(source).toContain("container:home-shell / size");
    expect(source).toContain(".home-fit{overflow:clip;min-height:0}");
    expect(source).toContain("grid-template-rows:minmax(0,1fr) auto");
    expect(source).toContain("@container home-shell (max-height:650px)");
    expect(source).toMatch(/\.content\{[^}]*overflow:auto/);
    expect(source).toContain(".detail-page[hidden]{display:none}");
  });

  it("shows each transcript once, without a duplicate generated title", () => {
    const source = readFileSync(new URL("./HomeScreen.svelte", import.meta.url), "utf8");
    for (const loop of ["recent.slice(0,2)", "filtered"]) {
      const entry = source.split(`{#each ${loop} as item (item.id)}`)[1]?.split("</article>")[0];
      expect(entry).toBeDefined();
      expect(entry?.match(/\{item\.text\}/g)).toHaveLength(1);
      expect(entry).not.toContain("item.text.split");
      expect(entry).toContain("copy(item.text)");
      expect(entry).toContain("dateLabel(item.completedAt)");
    }
  });

  it("rejects a late practice result after cancellation or reconnection", () => {
    expect(isCurrentFencedWork(owner, owner, 8, 8, false)).toBe(true);
    expect(isCurrentFencedWork(owner, owner, 8, 9, false)).toBe(false);
    expect(isCurrentFencedWork(owner, { ...owner, revision: 5 }, 8, 8, false)).toBe(false);
    expect(isCurrentFencedWork(owner, owner, 8, 8, true)).toBe(false);
  });

  it("uses backend streaming only for backend speech", () => {
    expect(practiceUsesLiveStream("backend")).toBe(true);
    expect(practiceUsesLiveStream("local")).toBe(false);
    expect(practiceUsesLiveStream(null)).toBe(false);
  });

  it("clears Destroy cache only after a successful local reset", () => {
    const values = new Map([
      ["destroy.microphone", "mic"],
      ["destroy.practice", "draft"],
      ["unrelated.preference", "keep"],
    ]);
    const storage = {
      get length() { return values.size; },
      key: (index: number) => [...values.keys()][index] ?? null,
      removeItem: (key: string) => values.delete(key),
    } as Pick<Storage, "length" | "key" | "removeItem">;
    const remove = vi.fn(() => clearScopedLocalStorage(storage));

    expect(localResetSucceeded({ reset: false })).toBe(false);
    expect(values.has("destroy.microphone")).toBe(true);
    if (localResetSucceeded({ reset: true })) remove();
    expect(remove).toHaveBeenCalledOnce();
    expect(values.has("destroy.microphone")).toBe(false);
    expect(values.has("unrelated.preference")).toBe(true);
  });
});
