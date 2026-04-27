import type { SortKey } from "../bindings";
import type { LayoutMode } from "./uiTypes";

export function normalizeLayoutMode(value: string | null | undefined): LayoutMode {
  return value === "list" ? "list" : "grid";
}

export function normalizeSortKey(value: string | null | undefined): SortKey {
  if (value === "title" || value === "size") {
    return value;
  }

  return "recent";
}
