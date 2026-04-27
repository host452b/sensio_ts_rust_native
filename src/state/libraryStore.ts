import { create } from "zustand";
import type { SortKey } from "../bindings";
import type { LayoutMode } from "../lib/uiTypes";

type LibraryState = {
  libraryPath: string | null;
  search: string;
  activeTagIds: string[];
  sortKey: SortKey;
  layoutMode: LayoutMode;
  selectedDocumentId: string | null;
  importPath: string;
  exportPath: string;
  setLibraryPath: (libraryPath: string | null) => void;
  setSearch: (search: string) => void;
  setActiveTagIds: (tagIds: string[]) => void;
  setSortKey: (sortKey: SortKey) => void;
  setLayoutMode: (layoutMode: LayoutMode) => void;
  setSelectedDocumentId: (documentId: string | null) => void;
  setImportPath: (importPath: string) => void;
  setExportPath: (exportPath: string) => void;
};

export const useLibraryStore = create<LibraryState>((set) => ({
  libraryPath: null,
  search: "",
  activeTagIds: [],
  sortKey: "recent",
  layoutMode: "grid",
  selectedDocumentId: null,
  importPath: "",
  exportPath: "",
  setLibraryPath: (libraryPath) => set({ libraryPath }),
  setSearch: (search) => set({ search }),
  setActiveTagIds: (activeTagIds) => set({ activeTagIds }),
  setSortKey: (sortKey) => set({ sortKey }),
  setLayoutMode: (layoutMode) => set({ layoutMode }),
  setSelectedDocumentId: (selectedDocumentId) => set({ selectedDocumentId }),
  setImportPath: (importPath) => set({ importPath }),
  setExportPath: (exportPath) => set({ exportPath }),
}));
