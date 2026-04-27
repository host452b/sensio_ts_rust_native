import { invoke } from "@tauri-apps/api/core";

export type AppBootstrap = {
  library_path: string;
  database_path: string;
};

export type AppSetting = {
  key: string;
  value: string;
  updated_at: string;
};

export type DocumentRecord = {
  id: string;
  title: string;
  original_path: string;
  storage_path: string;
  file_name: string;
  file_size: number;
  imported_at: string;
  updated_at: string;
  thumbnail_path: string | null;
};

export type DocumentQuery = {
  search: string | null;
  tag_ids: string[];
  sort_key: SortKey;
  offset: number;
  limit: number;
};

export type DocumentList = {
  items: DocumentRecord[];
  total: number;
};

export type SearchHit = {
  document_id: string;
  title: string;
  snippet: string;
  score: number;
};

export type HighlightRect = {
  left: number;
  top: number;
  width: number;
  height: number;
};

export type HighlightRecord = {
  id: string;
  document_id: string;
  page_index: number;
  rects: HighlightRect[];
  color: string;
  note: string | null;
  created_at: string;
};

export type HighlightInput = {
  document_id: string;
  page_index: number;
  rects: HighlightRect[];
  color: string;
  note: string | null;
};

export type SortKey = "recent" | "title" | "size";

export const commands = {
  initApp: () => invoke<AppBootstrap>("init_app"),
  listDocuments: (query: DocumentQuery) => invoke<DocumentList>("list_documents", { query }),
  importDocument: (sourcePath: string) =>
    invoke<DocumentRecord>("import_document", { sourcePath }),
  readPdfBytes: (documentId: string) =>
    invoke<number[]>("read_pdf_bytes", { documentId }),
  searchDocuments: (query: string, limit: number) =>
    invoke<SearchHit[]>("search_documents", { query, limit }),
  listHighlights: (documentId: string, pageIndex: number | null) =>
    invoke<HighlightRecord[]>("list_highlights", { documentId, pageIndex }),
  addHighlight: (input: HighlightInput) =>
    invoke<HighlightRecord>("add_highlight", { input }),
  deleteHighlight: (highlightId: string) =>
    invoke<void>("delete_highlight", { highlightId }),
  getSetting: (key: string) => invoke<AppSetting | null>("get_setting", { key }),
  setSetting: (key: string, value: string) =>
    invoke<AppSetting>("set_setting", { key, value }),
  exportPdfCopy: (documentId: string, destinationPath: string) =>
    invoke<string>("export_pdf_copy", { documentId, destinationPath }),
  openDocumentWindow: (documentId: string) =>
    invoke<void>("open_document_window", { documentId }),
  pickPdfFile: () => invoke<string | null>("pick_pdf_file"),
  pickExportPath: (suggestedFileName: string | null) =>
    invoke<string | null>("pick_export_path", { suggestedFileName }),
};
