import { invokeAccount as invoke } from "../account";

export type DictationEvent = {
  teamId: string | null;
  text: string;
  elapsedMs: number;
  completedAt?: number;
};

export type DictationSnapshot = {
  words: number;
  speakingMs: number;
  days: string[];
  lastText: string;
};

export type RecentDictation = {
  id: string;
  text: string;
  completedAt: number;
  words: number;
};

/** One local calendar day's totals. */
type DayTotals = { words: number; speakingMs: number };

/**
 * The on-disk shape: totals attributed to the day that produced them, plus
 * everything older than the days still cached.
 *
 * Storing per-day rather than one lifetime counter is what makes syncing safe.
 * The backend merges a day by keeping the larger value, so a client can
 * report the same day repeatedly without double-counting and a wiped cache
 * cannot pull the server's history down. A single lifetime number has no such
 * merge — whoever wrote last would win, which is how a local reset used to
 * become permanent.
 *
 * `carried` holds the totals for days that have aged out of `byDay`. It only
 * ever grows, is never reported back to the server, and exists so the lifetime
 * number stays right without caching every day forever.
 */
type PersistedMetrics = { byDay: Record<string, DayTotals>; carried: DayTotals };

const METRICS_PREFIX = "destroy.dictation.metrics";
/**
 * Which signed-in user the cached counters belong to.
 *
 * Persisted so that a restart followed by a *different* rep signing in on this
 * Mac is still recognised as a real account boundary. That is the only case
 * that has to drop these counters, and distinguishing it from "the team id has
 * not resolved yet" is the whole point: treating the second as the first is
 * what used to delete a rep's entire dictation history on an unlucky cold
 * start. See `accountCache.ts`.
 */
export const DICTATION_METRICS_OWNER_KEY = `${METRICS_PREFIX}.owner`;

/// True for the counter cache and its owner marker — the keys that are a local
/// mirror of the rep's server-side usage rather than customer content.
export function isDictationMetricsKey(key: string): boolean {
  return key === METRICS_PREFIX || key.startsWith(`${METRICS_PREFIX}.`);
}
export const DICTATION_UPDATED_EVENT = "destroy-dictation-updated";
const NO_TOTALS: DayTotals = { words: 0, speakingMs: 0 };
/// Enough history for the streak and the dashboard; matches the server's cap.
const MAX_DAYS = 90;
/// Mirrors the backend's per-day ceilings (migration 119) so a value that
/// would be refused is never written locally either.
const MAX_DAY_WORDS = 200_000;
const MAX_DAY_SPEAKING_MS = 86_400_000;
const MAX_RECENT_DICTATIONS = 20;

// Transcribed customer text is deliberately memory-only. Numeric usage metrics
// persist locally as a cache of the rep's server-side counters.
let last = {
  teamId: null as string | null,
  text: "",
};
let lastRecordId: string | null = null;
let recentDictations: Array<RecentDictation & { teamId: string | null }> = [];
let recentDictationSequence = 0;

// Vocabulary lives on the backend, envelope-encrypted and scoped to
// (team, user) — see migration 105. This map is only a read-through cache so
// the dashboard renders without a round trip; it is wiped at the account
// boundary and is never the source of truth.
const personalTermsByTeam = new Map<string, string[]>();

function metricsKey(teamId: string | null): string | null {
  return teamId ? `${METRICS_PREFIX}.${teamId}` : null;
}

function localDay(timestamp: number): string {
  const date = new Date(timestamp);
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

function currentLocalWeek(now: number): { start: string; end: string } {
  const date = new Date(now);
  const monday = new Date(date.getFullYear(), date.getMonth(), date.getDate());
  monday.setDate(monday.getDate() - ((monday.getDay() + 6) % 7));
  const sunday = new Date(monday.getFullYear(), monday.getMonth(), monday.getDate());
  sunday.setDate(sunday.getDate() + 6);
  return { start: localDay(monday.getTime()), end: localDay(sunday.getTime()) };
}

function count(value: unknown, max: number): number {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? Math.min(max, Math.max(0, Math.round(parsed))) : 0;
}

function isDay(value: unknown): value is string {
  return typeof value === "string" && /^\d{4}-\d{2}-\d{2}$/u.test(value);
}

/**
 * Reads the v1 shape — one lifetime total plus a bare list of active days.
 *
 * The old totals carry no per-day attribution, so they are credited to the most
 * recent day the rep dictated on. That keeps the lifetime number they have been
 * looking at, keeps every streak day, and lets the whole history sync through
 * the same per-day merge as everything written afterwards. Parking them in
 * `carried` instead would keep the number but never report it, so the server
 * would answer with a smaller total and the count would appear to drop again —
 * exactly the confusion this change exists to end.
 */
function fromLegacyMetrics(value: Record<string, unknown>): PersistedMetrics {
  const days = Array.isArray(value.days) ? value.days.filter(isDay) : [];
  const words = count(value.words, MAX_DAY_WORDS);
  const speakingMs = count(value.speakingMs, MAX_DAY_SPEAKING_MS);
  // No days to attribute the total to: keep the number, lose nothing.
  if (days.length === 0) return { byDay: {}, carried: { words, speakingMs } };
  const byDay: Record<string, DayTotals> = {};
  for (const day of days.slice(-MAX_DAYS)) byDay[day] = { ...NO_TOTALS };
  byDay[days[days.length - 1]] = { words, speakingMs };
  return { byDay, carried: { ...NO_TOTALS } };
}

function readDayTotals(value: unknown): DayTotals {
  if (!value || typeof value !== "object") return { ...NO_TOTALS };
  const { words, speakingMs } = value as Record<string, unknown>;
  return {
    words: count(words, MAX_DAY_WORDS),
    speakingMs: count(speakingMs, MAX_DAY_SPEAKING_MS),
  };
}

function readMetrics(teamId: string | null): PersistedMetrics {
  const key = metricsKey(teamId);
  if (!key || typeof localStorage === "undefined") return structuredEmpty();
  try {
    const value = JSON.parse(localStorage.getItem(key) ?? "null") as Record<string, unknown> | null;
    if (!value || typeof value !== "object") return structuredEmpty();
    if (!value.byDay) return fromLegacyMetrics(value);
    const raw = value.byDay as Record<string, unknown>;
    const byDay: Record<string, DayTotals> = {};
    for (const [day, dayValue] of Object.entries(raw)) {
      if (!isDay(day)) continue;
      byDay[day] = readDayTotals(dayValue);
    }
    return trimMetrics({
      byDay,
      carried: {
        words: count((value.carried as Record<string, unknown> | undefined)?.words, Number.MAX_SAFE_INTEGER),
        speakingMs: count(
          (value.carried as Record<string, unknown> | undefined)?.speakingMs,
          Number.MAX_SAFE_INTEGER,
        ),
      },
    });
  } catch {
    return structuredEmpty();
  }
}

function structuredEmpty(): PersistedMetrics {
  return { byDay: {}, carried: { ...NO_TOTALS } };
}

/// Caps the cached day detail, folding whatever ages out into `carried`.
///
/// Dropping the days outright would quietly shrink the lifetime total — the
/// class of bug this whole change exists to remove — so the counts move rather
/// than disappear. Only the per-day detail (and the streak reach) is lost.
function trimMetrics(metrics: PersistedMetrics): PersistedMetrics {
  const days = Object.keys(metrics.byDay).sort();
  if (days.length <= MAX_DAYS) return metrics;
  const byDay: Record<string, DayTotals> = {};
  const carried = { ...metrics.carried };
  for (const day of days.slice(0, days.length - MAX_DAYS)) {
    carried.words += metrics.byDay[day].words;
    carried.speakingMs += metrics.byDay[day].speakingMs;
  }
  for (const day of days.slice(-MAX_DAYS)) byDay[day] = metrics.byDay[day];
  return { byDay, carried };
}

function sumDays(byDay: Record<string, DayTotals>): DayTotals {
  return Object.values(byDay).reduce(
    (total, day) => ({
      words: total.words + day.words,
      speakingMs: total.speakingMs + day.speakingMs,
    }),
    { ...NO_TOTALS },
  );
}

function totals(metrics: PersistedMetrics): { words: number; speakingMs: number; days: string[] } {
  const days = sumDays(metrics.byDay);
  return {
    words: metrics.carried.words + days.words,
    speakingMs: metrics.carried.speakingMs + days.speakingMs,
    days: Object.keys(metrics.byDay).sort(),
  };
}

/**
 * Routes a metrics no-op to the native no-op diagnostic command.
 *
 * This counter can fail three different silent ways — no team id, no
 * localStorage, a throwing `setItem` — and all three previously looked
 * identical to "you have not dictated yet". The notch webview has no devtools
 * in a release build, so the log file is the only place a missed write can be
 * answered from. Counts only: transcribed text never goes to a log.
 */
function diagnose(message: string): void {
  try {
    void invoke("notch_log", { message: `dictation-metrics: ${message}` }).catch(() => {});
  } catch {
    // Diagnostics must never be the reason a dictation fails.
  }
}

function writeMetrics(teamId: string | null, metrics: PersistedMetrics): void {
  const key = metricsKey(teamId);
  if (!key) {
    diagnose("write skipped — no team id at dictation time");
    return;
  }
  if (typeof localStorage === "undefined") {
    diagnose("write skipped — localStorage unavailable");
    return;
  }
  try {
    localStorage.setItem(key, JSON.stringify(metrics));
    const summary = totals(metrics);
    diagnose(`wrote ${summary.words} words, ${summary.days.length} day(s)`);
  } catch (error) {
    // Dictation still delivers when local metric storage is unavailable.
    diagnose(`write failed — ${error instanceof Error ? error.message : String(error)}`);
  }
}

export function countDictationWords(text: string): number {
  return text.trim() ? text.trim().split(/\s+/u).length : 0;
}

export function recordDictation(event: DictationEvent): DictationSnapshot {
  const metrics = readMetrics(event.teamId);
  const completedAt = event.completedAt ?? Date.now();
  const day = localDay(completedAt);
  const previous = metrics.byDay[day] ?? { ...NO_TOTALS };
  const byDay = {
    ...metrics.byDay,
    [day]: {
      words: Math.min(MAX_DAY_WORDS, previous.words + countDictationWords(event.text)),
      speakingMs: Math.min(
        MAX_DAY_SPEAKING_MS,
        previous.speakingMs + Math.max(0, event.elapsedMs),
      ),
    },
  };
  const next = trimMetrics({ byDay, carried: metrics.carried });
  writeMetrics(event.teamId, next);
  const id = `${completedAt}-${++recentDictationSequence}`;
  recentDictations = [
    { id, teamId: event.teamId, text: event.text, completedAt, words: countDictationWords(event.text) },
    ...recentDictations,
  ].slice(0, MAX_RECENT_DICTATIONS);
  last = { teamId: event.teamId, text: event.text };
  lastRecordId = id;
  if (typeof window !== "undefined") {
    window.dispatchEvent(new CustomEvent(DICTATION_UPDATED_EVENT));
  }
  // The rep's durable copy lives on the backend. Reporting the day's running
  // total (not this one dictation) keeps the write idempotent, so a failure here
  // is recoverable by the next dictation or the next hydrate rather than lost.
  void pushDay(event.teamId, day, next.byDay[day]).catch(() => {});
  return readDictationSnapshot(event.teamId);
}

export function readDictationSnapshot(teamId: string | null): DictationSnapshot {
  const metrics = readMetrics(teamId);
  const sameTeam = Boolean(teamId) && last.teamId === teamId;
  return { ...totals(metrics), lastText: sameTeam ? last.text : "" };
}

export function readWeeklyDictationSnapshot(
  teamId: string | null,
  now = Date.now(),
): DictationSnapshot {
  const metrics = readMetrics(teamId);
  const { start, end } = currentLocalWeek(now);
  const byDay = Object.fromEntries(
    Object.entries(metrics.byDay).filter(([day]) => day >= start && day <= end),
  );
  const week = sumDays(byDay);
  const sameTeam = Boolean(teamId) && last.teamId === teamId;
  return {
    words: week.words,
    speakingMs: week.speakingMs,
    days: Object.keys(byDay).sort(),
    lastText: sameTeam ? last.text : "",
  };
}

export function readRecentDictations(teamId: string | null): RecentDictation[] {
  if (!teamId) return [];
  return recentDictations
    .filter((record) => record.teamId === teamId)
    .map(({ teamId: _teamId, ...record }) => ({ ...record }));
}

type UsageResponse = { words?: unknown; speaking_ms?: unknown; days?: unknown };

async function usageApi(
  method: "GET" | "POST",
  body?: { day: string; words: number; speaking_ms: number },
): Promise<UsageResponse | null> {
  return invoke<UsageResponse | null>("backend_api", {
    method,
    path: "/v1/dictation/usage",
    body: body ?? null,
  });
}

function readServerDays(response: UsageResponse | null): Record<string, DayTotals> {
  const rows = Array.isArray(response?.days) ? response.days : [];
  const byDay: Record<string, DayTotals> = {};
  for (const row of rows) {
    if (!row || typeof row !== "object") continue;
    const { day, words, speaking_ms: speakingMs } = row as Record<string, unknown>;
    if (!isDay(day)) continue;
    byDay[day] = {
      words: count(words, MAX_DAY_WORDS),
      speakingMs: count(speakingMs, MAX_DAY_SPEAKING_MS),
    };
  }
  return byDay;
}

async function pushDay(
  teamId: string | null,
  day: string,
  dayTotals: DayTotals,
): Promise<UsageResponse | null> {
  if (!teamId) return null;
  return usageApi("POST", { day, words: dayTotals.words, speaking_ms: dayTotals.speakingMs });
}

/**
 * Reconciles this Mac's cache with the rep's durable counters.
 *
 * Call it when the dictation surface opens. The exchange is two-way and
 * symmetric: every day is merged by keeping the larger count, then any day this
 * Mac counted higher is reported back. The server merges the same way, so
 * neither side can erase the other — a rep who dictated offline for a week
 * catches up on reconnect, and a cache that was wiped is refilled from the
 * server instead of overwriting it with zeroes.
 */
export async function syncDictationUsage(teamId: string | null): Promise<DictationSnapshot> {
  if (!teamId) return readDictationSnapshot(teamId);
  const local = readMetrics(teamId);
  let response: UsageResponse | null;
  try {
    response = await usageApi("GET");
  } catch (error) {
    // An offline rep keeps dictating against the local cache; the next sync
    // reports whatever accumulated. Nothing is lost by failing quietly here.
    diagnose(`sync skipped — ${error instanceof Error ? error.message : String(error)}`);
    return readDictationSnapshot(teamId);
  }

  const serverDays = readServerDays(response);
  const merged: Record<string, DayTotals> = { ...local.byDay };
  for (const [day, server] of Object.entries(serverDays)) {
    const mine = merged[day] ?? { ...NO_TOTALS };
    merged[day] = {
      words: Math.max(mine.words, server.words),
      speakingMs: Math.max(mine.speakingMs, server.speakingMs),
    };
  }

  // The authoritative lifetime total: what the server has already stored, plus
  // whatever this Mac counted beyond it and is about to report below. Counting
  // the unreported excess now means the rep never watches their total dip while
  // the pushes are still in flight.
  const lifetime = Object.entries(merged).reduce(
    (total, [day, mine]) => {
      const server = serverDays[day] ?? NO_TOTALS;
      return {
        words: total.words + Math.max(0, mine.words - server.words),
        speakingMs: total.speakingMs + Math.max(0, mine.speakingMs - server.speakingMs),
      };
    },
    {
      words: count(response?.words, Number.MAX_SAFE_INTEGER),
      speakingMs: count(response?.speaking_ms, Number.MAX_SAFE_INTEGER),
    },
  );

  // Whatever the lifetime total holds beyond the days this cache keeps. Deriving
  // it by subtraction — rather than accumulating it locally — is what keeps the
  // rendered number equal to the server's however often days age out.
  const kept = trimMetrics({ byDay: merged, carried: { ...NO_TOTALS } }).byDay;
  const keptTotal = sumDays(kept);
  writeMetrics(teamId, {
    byDay: kept,
    carried: {
      words: Math.max(0, lifetime.words - keptTotal.words),
      speakingMs: Math.max(0, lifetime.speakingMs - keptTotal.speakingMs),
    },
  });

  // Report only the days this Mac counted higher; the rest already match.
  for (const [day, mine] of Object.entries(merged)) {
    const server = serverDays[day];
    if (server && server.words >= mine.words && server.speakingMs >= mine.speakingMs) continue;
    try {
      await pushDay(teamId, day, mine);
    } catch {
      // Retried on the next sync; the merge above already kept the local value.
    }
  }

  if (typeof window !== "undefined") {
    window.dispatchEvent(new CustomEvent(DICTATION_UPDATED_EVENT));
  }
  return readDictationSnapshot(teamId);
}

export function clearLastDictation(teamId: string | null): void {
  if (last.teamId !== teamId) return;
  if (lastRecordId) {
    recentDictations = recentDictations.filter((record) => record.id !== lastRecordId);
  }
  last = { teamId: null, text: "" };
  lastRecordId = null;
  if (typeof window !== "undefined") {
    window.dispatchEvent(new CustomEvent(DICTATION_UPDATED_EVENT));
  }
}

/** Cached terms for immediate render. Call `loadPersonalDictationTerms` for truth. */
export function readPersonalDictationTerms(teamId: string | null): string[] {
  return teamId ? [...(personalTermsByTeam.get(teamId) ?? [])] : [];
}

type VocabularyResponse = { terms?: unknown; max_terms?: unknown };

function readTerms(response: VocabularyResponse | null): string[] {
  return Array.isArray(response?.terms)
    ? response.terms.filter((term): term is string => typeof term === "string")
    : [];
}

async function vocabularyApi(
  method: "GET" | "PUT",
  body?: { terms: string[] },
): Promise<VocabularyResponse | null> {
  return invoke<VocabularyResponse | null>("backend_api", {
    method,
    path: "/v1/dictation/vocabulary",
    body: body ?? null,
  });
}

/** Fetches the rep's saved terms and refreshes the render cache. */
export async function loadPersonalDictationTerms(teamId: string | null): Promise<string[]> {
  if (!teamId) return [];
  const terms = readTerms(await vocabularyApi("GET"));
  personalTermsByTeam.set(teamId, terms);
  return [...terms];
}

/**
 * Adds a term and persists the whole list.
 *
 * The server owns validation, so a term it refuses surfaces as a thrown error
 * rather than a silently dropped entry — the rep needs to know their term is
 * not in use.
 */
export async function addPersonalDictationTerm(
  teamId: string | null,
  value: string,
): Promise<string[]> {
  if (!teamId) return [];
  const term = value.trim();
  const current = personalTermsByTeam.get(teamId) ?? [];
  if (!term || current.some((item) => item.toLocaleLowerCase() === term.toLocaleLowerCase())) {
    return [...current];
  }
  const terms = readTerms(await vocabularyApi("PUT", { terms: [...current, term] }));
  personalTermsByTeam.set(teamId, terms);
  return [...terms];
}

export async function removePersonalDictationTerm(
  teamId: string | null,
  value: string,
): Promise<string[]> {
  if (!teamId) return [];
  const current = personalTermsByTeam.get(teamId) ?? [];
  const terms = readTerms(
    await vocabularyApi("PUT", { terms: current.filter((item) => item !== value) }),
  );
  personalTermsByTeam.set(teamId, terms);
  return [...terms];
}

/** Clear only this owner's transient transcripts, without erasing usage or vocabulary. */
export function clearRecentDictations(teamId: string | null): void {
  if (!teamId) return;
  recentDictations = recentDictations.filter((item) => item.teamId !== teamId);
  if (last.teamId === teamId) {
    last = { teamId: null, text: "" };
    lastRecordId = null;
  }
  if (typeof window !== "undefined") window.dispatchEvent(new CustomEvent(DICTATION_UPDATED_EVENT));
}

export function clearDictationMemory(): void {
  last = { teamId: null, text: "" };
  lastRecordId = null;
  recentDictations = [];
  personalTermsByTeam.clear();
  if (typeof window !== "undefined") {
    window.dispatchEvent(new CustomEvent(DICTATION_UPDATED_EVENT));
  }
}

export function dictationAverageWpm(snapshot: DictationSnapshot): number {
  if (snapshot.words === 0 || snapshot.speakingMs < 1_000) return 0;
  return Math.round(snapshot.words / (snapshot.speakingMs / 60_000));
}

export function dictationTimeSavedMs(snapshot: DictationSnapshot): number {
  // Compare against a conservative 40 WPM keyboard baseline.
  return Math.max(0, (snapshot.words / 40) * 60_000 - snapshot.speakingMs);
}

export function dictationDayStreak(snapshot: DictationSnapshot, now = new Date()): number {
  const days = new Set(snapshot.days);
  const cursor = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  // A streak counts days dictated in a row, so it must survive the part of a
  // day before the rep has spoken. Anchoring on today alone reads zero every
  // morning; anchor on yesterday when today is still empty and only break the
  // run once a whole day has passed with nothing dictated.
  if (!days.has(localDay(cursor.getTime()))) {
    cursor.setDate(cursor.getDate() - 1);
  }
  let streak = 0;
  while (days.has(localDay(cursor.getTime()))) {
    streak += 1;
    cursor.setDate(cursor.getDate() - 1);
  }
  return streak;
}
