import assert from "node:assert/strict";
import { test } from "node:test";
import { normalizeLayoutMode, normalizeSortKey } from "../tmp/preferences-test/lib/preferences.js";

test("normalizes persisted layout mode", () => {
  assert.equal(normalizeLayoutMode("list"), "list");
  assert.equal(normalizeLayoutMode("grid"), "grid");
  assert.equal(normalizeLayoutMode("removed-list"), "grid");
  assert.equal(normalizeLayoutMode(null), "grid");
});

test("normalizes persisted sort key", () => {
  assert.equal(normalizeSortKey("title"), "title");
  assert.equal(normalizeSortKey("size"), "size");
  assert.equal(normalizeSortKey("recent"), "recent");
  assert.equal(normalizeSortKey("updated_at"), "recent");
  assert.equal(normalizeSortKey(null), "recent");
});
