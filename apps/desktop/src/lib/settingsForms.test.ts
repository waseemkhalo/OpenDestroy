import { describe, expect, it } from "vitest";
import {
  parseLinksJson,
  parseVariablesJson,
  serializeLinks,
  serializeVariables,
  validateLinkRows,
  validateVariableRows,
} from "./settingsForms";

describe("Settings structured forms", () => {
  it("round-trips the existing variables object schema", () => {
    const parsed = parseVariablesJson('{"company":"Destroy","icp":"teams"}');
    expect(parsed.error).toBe("");
    expect(serializeVariables(parsed.rows)).toBe('{\n  "company": "Destroy",\n  "icp": "teams"\n}');
  });

  it("round-trips the schema-valid __proto__ variable name", () => {
    const parsed = parseVariablesJson('{"__proto__":"safe"}');
    expect(parsed.error).toBe("");
    const roundTripped = JSON.parse(serializeVariables(parsed.rows)) as Record<string, string>;
    expect(Object.prototype.hasOwnProperty.call(roundTripped, "__proto__")).toBe(true);
    expect(roundTripped["__proto__"]).toBe("safe");
  });

  it("keeps malformed variables visible as an error instead of dropping values", () => {
    const parsed = parseVariablesJson('{"company":42,"private_note":"keep this"}');
    expect(parsed.rows).toEqual([]);
    expect(parsed.error).toContain("company");
  });

  it("round-trips link keywords while exposing only name and URL rows", () => {
    const parsed = parseLinksJson('[{"name":"Docs","url":"https://example.com/docs","keywords":["docs","guide"]}]');
    expect(parsed.error).toBe("");
    expect(serializeLinks(parsed.rows)).toContain('"keywords": [\n      "docs",\n      "guide"\n    ]');
  });

  it("rejects malformed links without silently returning partial rows", () => {
    const parsed = parseLinksJson('[{"name":"Docs","url":"https://example.com"},{"name":"Bad","url":"http://example.com"}]');
    expect(parsed.rows).toEqual([]);
    expect(parsed.error).toContain("HTTPS");
  });

  it("validates structured edits before serialization", () => {
    expect(validateVariableRows([{ id: "1", name: "company", value: "Destroy" }])).toBe("");
    expect(validateVariableRows([{ id: "1", name: "company-name", value: "Destroy" }])).toContain("underscores");
    expect(validateLinkRows([{ id: "1", name: "Docs", url: "https://example.com" }])).toBe("");
    expect(validateLinkRows([{ id: "1", name: "Docs", url: "http://example.com" }])).toContain("HTTPS");
    expect(validateLinkRows([{ id: "1", name: "Docs", url: "https://user:password@example.com" }])).toContain("HTTPS");
  });
});
