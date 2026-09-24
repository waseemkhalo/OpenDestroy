export type SettingsVariableRow = {
  id: string;
  name: string;
  value: string;
};

export type SettingsLinkRow = {
  id: string;
  name: string;
  url: string;
  keywords?: string | string[];
};

export type ParsedSettingsForm<T> = {
  rows: T[];
  error: string;
};

const VARIABLE_NAME = /^[A-Za-z0-9_]+$/u;
const MAX_VARIABLES = 50;
const MAX_VARIABLE_NAME = 40;
const MAX_VARIABLE_VALUE = 4000;
const MAX_LINKS = 100;
const MAX_LINK_NAME = 200;
const MAX_LINK_KEYWORDS = 1000;

function rowId(prefix: string, index: number): string {
  return `${prefix}-${index}`;
}

function parseJson(input: string, label: string): unknown {
  try {
    return JSON.parse(input);
  } catch {
    throw new Error(`${label} JSON is invalid. Fix it in Advanced JSON before saving.`);
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function validHttpsUrl(value: string): boolean {
  try {
    const url = new URL(value);
    return url.protocol === "https:" && Boolean(url.hostname) && url.username === "" && url.password === "";
  } catch {
    return false;
  }
}

function validKeywords(value: unknown): value is string | string[] {
  if (typeof value === "string") return [...value].length <= MAX_LINK_KEYWORDS;
  return (
    Array.isArray(value) &&
    value.every((keyword) => typeof keyword === "string") &&
    value.reduce((total, keyword) => total + [...keyword].length, 0) <= MAX_LINK_KEYWORDS
  );
}

export function parseVariablesJson(input: string): ParsedSettingsForm<SettingsVariableRow> {
  try {
    const parsed = parseJson(input, "Variables");
    if (!isRecord(parsed)) throw new Error("Variables must be a JSON object.");
    const entries = Object.entries(parsed);
    if (entries.length > MAX_VARIABLES) throw new Error("Variables allow up to 50 entries.");
    const rows = entries.map(([name, value], index) => {
      if (!name || name.length > MAX_VARIABLE_NAME || !VARIABLE_NAME.test(name)) {
        throw new Error("Variable names use letters, numbers, and underscores, up to 40 characters.");
      }
      if (typeof value !== "string" || [...value].length > MAX_VARIABLE_VALUE) {
        throw new Error(`Variable “${name}” must have a text value up to 4,000 characters.`);
      }
      return { id: rowId("variable", index), name, value };
    });
    return { rows, error: "" };
  } catch (error) {
    return { rows: [], error: error instanceof Error ? error.message : "Variables could not be read." };
  }
}

export function parseLinksJson(input: string): ParsedSettingsForm<SettingsLinkRow> {
  try {
    const parsed = parseJson(input, "Saved links");
    if (!Array.isArray(parsed)) throw new Error("Saved links must be a JSON array.");
    if (parsed.length > MAX_LINKS) throw new Error("Saved links allow up to 100 entries.");
    const rows = parsed.map((value, index) => {
      if (!isRecord(value)) throw new Error(`Saved link ${index + 1} must be an object.`);
      const unsupported = Object.keys(value).filter((key) => !["name", "url", "keywords"].includes(key));
      if (unsupported.length) throw new Error(`Saved link ${index + 1} contains unsupported fields: ${unsupported.join(", ")}.`);
      const name = value.name;
      const url = value.url;
      if (typeof name !== "string" || !name.trim() || [...name].length > MAX_LINK_NAME) {
        throw new Error(`Saved link ${index + 1} needs a name up to 200 characters.`);
      }
      if (typeof url !== "string" || !validHttpsUrl(url)) {
        throw new Error(`Saved link ${index + 1} needs a valid HTTPS URL.`);
      }
      const keywords = value.keywords;
      if (keywords !== undefined && !validKeywords(keywords)) {
        throw new Error(`Saved link ${index + 1} has unsupported keywords. Use text or a list of text values.`);
      }
      return {
        id: rowId("link", index),
        name,
        url,
        ...(keywords === undefined ? {} : { keywords }),
      };
    });
    return { rows, error: "" };
  } catch (error) {
    return { rows: [], error: error instanceof Error ? error.message : "Saved links could not be read." };
  }
}

export function validateVariableRows(rows: SettingsVariableRow[]): string {
  if (rows.length > MAX_VARIABLES) return "Variables allow up to 50 entries.";
  const names = new Set<string>();
  for (const row of rows) {
    const name = row.name.trim();
    if (!name || name.length > MAX_VARIABLE_NAME || !VARIABLE_NAME.test(name)) {
      return "Variable names use letters, numbers, and underscores, up to 40 characters.";
    }
    if (names.has(name)) return `Variable name “${name}” is used more than once.`;
    names.add(name);
    if ([...row.value].length > MAX_VARIABLE_VALUE) return `Variable “${name}” is too long.`;
  }
  return "";
}

export function validateLinkRows(rows: SettingsLinkRow[]): string {
  if (rows.length > MAX_LINKS) return "Saved links allow up to 100 entries.";
  for (const [index, row] of rows.entries()) {
    if (!row.name.trim() || [...row.name].length > MAX_LINK_NAME) return `Saved link ${index + 1} needs a name up to 200 characters.`;
    if (!validHttpsUrl(row.url.trim())) return `Saved link ${index + 1} needs a valid HTTPS URL.`;
    if (row.keywords !== undefined && !validKeywords(row.keywords)) return `Saved link ${index + 1} has unsupported keywords.`;
  }
  return "";
}

export function serializeVariables(rows: SettingsVariableRow[]): string {
  return JSON.stringify(
    Object.fromEntries(rows.map((row) => [row.name.trim(), row.value])),
    null,
    2,
  );
}

export function serializeLinks(rows: SettingsLinkRow[]): string {
  return JSON.stringify(
    rows.map(({ name, url, keywords }) => ({
      name: name.trim(),
      url: url.trim(),
      ...(keywords === undefined ? {} : { keywords }),
    })),
    null,
    2,
  );
}
