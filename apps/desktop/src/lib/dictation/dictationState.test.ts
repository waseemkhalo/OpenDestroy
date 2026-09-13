import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";

import {
  clearLastDictation,
  addPersonalDictationTerm,
  clearDictationMemory,
  countDictationWords,
  dictationAverageWpm,
  dictationDayStreak,
  readDictationSnapshot,
  loadPersonalDictationTerms,
  readPersonalDictationTerms,
  recordDictation,
  removePersonalDictationTerm,
  syncDictationUsage,
} from "./dictationState";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

/// Stands in for the backend: echoes back whatever the client asked to
/// store, which is what the real route does after sanitising.
function serverHolding(terms: string[]) {
  let stored = [...terms];
  mockedInvoke.mockImplementation(async (_command, args) => {
    const request = args as { method: string; body?: { terms: string[] } | null };
    if (request.method === "PUT" && request.body) stored = [...request.body.terms];
    return { terms: stored, max_terms: 100 };
  });
  return () => stored;
}

function storage() {
  const values = new Map<string, string>();
  return {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
  };
}

describe("dictation state", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    vi.stubGlobal("localStorage", storage());
    vi.stubGlobal("window", { dispatchEvent: vi.fn() });
    clearLastDictation("team-a");
    clearDictationMemory();
  });

  it("keeps text in memory while persisting only numeric metrics", () => {
    recordDictation({
      teamId: "team-a",
      text: "Hello Priya Shah",
      elapsedMs: 1_500,
      completedAt: new Date(2026, 7, 23).getTime(),
    });
    const snapshot = readDictationSnapshot("team-a");
    expect(snapshot.words).toBe(3);
    expect(snapshot.lastText).toBe("Hello Priya Shah");
    expect(localStorage.getItem("destroy.dictation.metrics.team-a")).not.toContain("Priya");
    expect(readDictationSnapshot("team-b").lastText).toBe("");
  });

  it("computes speed and a consecutive-day streak", () => {
    for (const day of [21, 22, 23]) {
      recordDictation({
        teamId: "team-a",
        text: "one two three four",
        elapsedMs: 2_000,
        completedAt: new Date(2026, 7, day).getTime(),
      });
    }
    const snapshot = readDictationSnapshot("team-a");
    expect(dictationDayStreak(snapshot, new Date(2026, 7, 23))).toBe(3);
    expect(dictationAverageWpm(snapshot)).toBe(120);
  });

  it("holds the streak through a day the rep has not dictated in yet", () => {
    for (const day of [21, 22, 23]) {
      recordDictation({
        teamId: "team-a",
        text: "one two three four",
        elapsedMs: 2_000,
        completedAt: new Date(2026, 7, day).getTime(),
      });
    }
    const snapshot = readDictationSnapshot("team-a");
    // Morning of the 24th, before the first dictation: the run still stands.
    expect(dictationDayStreak(snapshot, new Date(2026, 7, 24))).toBe(3);
    // A full day passed with nothing spoken, so the run is genuinely over.
    expect(dictationDayStreak(snapshot, new Date(2026, 7, 25))).toBe(0);
  });

  it("counts normalized words", () => {
    expect(countDictationWords("  a   b\n c ")).toBe(3);
    expect(countDictationWords("   ")).toBe(0);
  });

  it("persists personal vocabulary through the server, never localStorage", async () => {
    const stored = serverHolding([]);

    expect(await addPersonalDictationTerm("team-a", "Mehmi Group")).toEqual(["Mehmi Group"]);
    // A case-insensitive repeat is a no-op and never reaches the server again.
    const callsBefore = mockedInvoke.mock.calls.length;
    expect(await addPersonalDictationTerm("team-a", "mehmi group")).toEqual(["Mehmi Group"]);
    expect(mockedInvoke.mock.calls.length).toBe(callsBefore);

    // Vocabulary is customer content: it must not be written to disk here.
    expect(localStorage.getItem("destroy.dictation.vocabulary.team-a")).toBeNull();

    expect(await removePersonalDictationTerm("team-a", "Mehmi Group")).toEqual([]);
    expect(stored()).toEqual([]);
  });

  it("scopes the render cache to one team", async () => {
    serverHolding([]);
    await addPersonalDictationTerm("team-a", "Northstar");
    expect(readPersonalDictationTerms("team-a")).toEqual(["Northstar"]);
    expect(readPersonalDictationTerms("team-b")).toEqual([]);
  });

  it("drops the cache at the account boundary", async () => {
    serverHolding([]);
    await addPersonalDictationTerm("team-a", "Northstar");
    clearDictationMemory();
    expect(readPersonalDictationTerms("team-a")).toEqual([]);
  });

  it("returns nothing without a team rather than calling the server", async () => {
    serverHolding(["Northstar"]);
    expect(await loadPersonalDictationTerms(null)).toEqual([]);
    expect(mockedInvoke).not.toHaveBeenCalled();
  });
});

describe("metrics write diagnostics", () => {
  it("reports the reason a write was skipped instead of failing silently", () => {
    // No team id is the failure mode that looks exactly like "never dictated".
    recordDictation({ teamId: null, text: "hello there", elapsedMs: 1000 });
    const messages = mockedInvoke.mock.calls
      .filter(([command]) => command === "notch_log")
      .map(([, args]) => (args as { message: string }).message);
    expect(messages).toContain("dictation-metrics: write skipped — no team id at dictation time");
  });

  it("confirms a successful write so a working counter is provable too", () => {
    recordDictation({ teamId: "team-a", text: "one two three", elapsedMs: 1000 });
    const messages = mockedInvoke.mock.calls
      .filter(([command]) => command === "notch_log")
      .map(([, args]) => (args as { message: string }).message);
    expect(messages.some((m) => m.startsWith("dictation-metrics: wrote 3 words"))).toBe(true);
  });

  it("never logs transcribed text", () => {
    recordDictation({ teamId: "team-a", text: "Priya Shah at Northstar", elapsedMs: 1000 });
    const logged = JSON.stringify(mockedInvoke.mock.calls);
    expect(logged).not.toContain("Priya");
    expect(logged).not.toContain("Northstar");
  });
});

/// Stands in for the backend's `/v1/dictation/usage`: keeps the larger value
/// per day, exactly as migration 119's upsert does.
function usageServer(seed: Record<string, { words: number; speaking_ms: number }> = {}) {
  const stored = { ...seed };
  mockedInvoke.mockImplementation(async (command, args) => {
    if (command !== "backend_api") return undefined;
    const request = args as {
      method: string;
      path: string;
      body?: { day: string; words: number; speaking_ms: number } | null;
    };
    if (request.path !== "/v1/dictation/usage") return null;
    if (request.method === "POST" && request.body) {
      const { day, words, speaking_ms: speakingMs } = request.body;
      const previous = stored[day] ?? { words: 0, speaking_ms: 0 };
      stored[day] = {
        words: Math.max(previous.words, words),
        speaking_ms: Math.max(previous.speaking_ms, speakingMs),
      };
    }
    const days = Object.keys(stored).sort().reverse();
    return {
      words: days.reduce((sum, day) => sum + stored[day].words, 0),
      speaking_ms: days.reduce((sum, day) => sum + stored[day].speaking_ms, 0),
      days: days.map((day) => ({ day, ...stored[day] })),
    };
  });
  return () => stored;
}

describe("durable usage", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    vi.stubGlobal("localStorage", storage());
    vi.stubGlobal("window", { dispatchEvent: vi.fn() });
    clearDictationMemory();
  });

  it("restores a wiped local cache from the rep's server counters", async () => {
    // Exactly the failure this table exists for: the local key was deleted, so
    // the Mac believes the rep has never dictated.
    usageServer({
      "2026-08-21": { words: 900, speaking_ms: 300_000 },
      "2026-08-22": { words: 1_587, speaking_ms: 400_000 },
    });
    expect(readDictationSnapshot("team-a").words).toBe(0);

    const synced = await syncDictationUsage("team-a");

    expect(synced.words).toBe(2_487);
    expect(synced.days).toEqual(["2026-08-21", "2026-08-22"]);
    expect(dictationDayStreak(synced, new Date(2026, 7, 22))).toBe(2);
    // And the cache now holds it, so the next render needs no round trip.
    expect(readDictationSnapshot("team-a").words).toBe(2_487);
  });

  it("reports days dictated offline instead of losing them", async () => {
    const stored = usageServer({ "2026-08-21": { words: 900, speaking_ms: 300_000 } });
    recordDictation({
      teamId: "team-a",
      text: "one two three four",
      elapsedMs: 2_000,
      completedAt: new Date(2026, 7, 22).getTime(),
    });

    const synced = await syncDictationUsage("team-a");

    expect(stored()["2026-08-22"]).toEqual({ words: 4, speaking_ms: 2_000 });
    expect(synced.words).toBe(904);
  });

  it("never lets a stale client pull the server's history down", async () => {
    // This Mac only knows about a fraction of the day; the server keeps its own.
    const stored = usageServer({ "2026-08-22": { words: 1_587, speaking_ms: 400_000 } });
    recordDictation({
      teamId: "team-a",
      text: "hello",
      elapsedMs: 1_000,
      completedAt: new Date(2026, 7, 22).getTime(),
    });

    const synced = await syncDictationUsage("team-a");

    expect(stored()["2026-08-22"].words).toBe(1_587);
    expect(synced.words).toBe(1_587);
  });

  it("keeps the lifetime total when days age out of the cached window", async () => {
    // 120 days of history: more than the 90 this cache keeps.
    const seed: Record<string, { words: number; speaking_ms: number }> = {};
    for (let index = 0; index < 120; index += 1) {
      const day = new Date(2026, 0, 1 + index);
      const key = `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, "0")}-${String(day.getDate()).padStart(2, "0")}`;
      seed[key] = { words: 10, speaking_ms: 1_000 };
    }
    usageServer(seed);

    const synced = await syncDictationUsage("team-a");

    // The window is trimmed, the total is not.
    expect(synced.days.length).toBe(90);
    expect(synced.words).toBe(1_200);
    expect(readDictationSnapshot("team-a").words).toBe(1_200);
  });

  it("carries a pre-sync local history into the server on first contact", async () => {
    // The v1 on-disk shape, written by a build that had no server copy.
    localStorage.setItem(
      "destroy.dictation.metrics.team-a",
      JSON.stringify({ words: 2_487, speakingMs: 900_000, days: ["2026-08-21", "2026-08-22"] }),
    );
    const stored = usageServer();

    const synced = await syncDictationUsage("team-a");

    expect(synced.words).toBe(2_487);
    expect(synced.days).toEqual(["2026-08-21", "2026-08-22"]);
    // Both days reach the server, so the streak survives the next local wipe.
    expect(Object.keys(stored()).sort()).toEqual(["2026-08-21", "2026-08-22"]);
  });

  it("keeps showing cached counters when the server cannot be reached", async () => {
    mockedInvoke.mockImplementation(async (command) => {
      if (command === "backend_api") throw new Error("offline");
      return undefined;
    });
    recordDictation({
      teamId: "team-a",
      text: "one two three",
      elapsedMs: 1_000,
      completedAt: new Date(2026, 7, 22).getTime(),
    });

    const synced = await syncDictationUsage("team-a");

    expect(synced.words).toBe(3);
  });

  it("sends counts and never transcribed text to the server", async () => {
    usageServer();
    recordDictation({ teamId: "team-a", text: "Priya Shah at Northstar", elapsedMs: 1_000 });
    await syncDictationUsage("team-a");
    const sent = JSON.stringify(mockedInvoke.mock.calls);
    expect(sent).not.toContain("Priya");
    expect(sent).not.toContain("Northstar");
    expect(sent).toContain("/v1/dictation/usage");
  });
});
