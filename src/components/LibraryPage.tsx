import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { commands, type DocumentRecord, type SortKey } from "../bindings";
import { readerHash } from "../lib/hashRoute";
import type { LayoutMode } from "../lib/uiTypes";
import { useLibraryStore } from "../state/libraryStore";
import { PdfViewer } from "./PdfViewer";

const sortOptions: Array<{ value: SortKey; label: string }> = [
  { value: "recent", label: "最近导入" },
  { value: "title", label: "标题" },
  { value: "size", label: "文件大小" },
];

export function LibraryPage() {
  const queryClient = useQueryClient();
  const [dialogError, setDialogError] = useState<string | null>(null);
  const {
    libraryPath,
    search,
    activeTagIds,
    sortKey,
    layoutMode,
    selectedDocumentId,
    importPath,
    exportPath,
    setSearch,
    setSortKey,
    setLayoutMode,
    setSelectedDocumentId,
    setImportPath,
    setExportPath,
  } = useLibraryStore();

  const documents = useQuery({
    queryKey: ["documents", search, activeTagIds, sortKey],
    queryFn: () =>
      commands.listDocuments({
        search: search.trim() || null,
        tag_ids: activeTagIds,
        sort_key: sortKey,
        offset: 0,
        limit: 80,
      }),
  });

  const selectedDocument =
    documents.data?.items.find((document) => document.id === selectedDocumentId) ??
    documents.data?.items[0] ??
    null;
  const visibleDocuments = documents.data?.items ?? [];
  const visibleFileSize = visibleDocuments.reduce(
    (total, document) => total + document.file_size,
    0,
  );
  const hasSearch = search.trim().length > 0;

  const importMutation = useMutation({
    mutationFn: () => commands.importDocument(importPath.trim()),
    onSuccess: (document) => {
      setImportPath("");
      setSelectedDocumentId(document.id);
      void queryClient.invalidateQueries({ queryKey: ["documents"] });
    },
  });

  const exportMutation = useMutation({
    mutationFn: () => {
      if (!selectedDocument) {
        throw new Error("请先选择文档");
      }
      return commands.exportPdfCopy(selectedDocument.id, exportPath.trim());
    },
  });

  const updateLayout = (nextLayoutMode: LayoutMode) => {
    setLayoutMode(nextLayoutMode);
    void commands.setSetting("layoutMode", nextLayoutMode);
  };

  const updateSort = (nextSortKey: SortKey) => {
    setSortKey(nextSortKey);
    void commands.setSetting("sortKey", nextSortKey);
  };

  const selectImportPath = async () => {
    setDialogError(null);
    try {
      const path = await commands.pickPdfFile();
      if (path) {
        setImportPath(path);
      }
    } catch (error) {
      setDialogError(String(error));
    }
  };

  const selectExportPath = async () => {
    setDialogError(null);
    try {
      const path = await commands.pickExportPath(selectedDocument?.file_name ?? null);
      if (path) {
        setExportPath(path);
      }
    } catch (error) {
      setDialogError(String(error));
    }
  };

  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label="资料库控制">
        <div className="brand-block">
          <div className="brand-mark" aria-hidden="true">
            <span>S</span>
            <small>PDF</small>
          </div>
          <div>
            <div className="brand-title">Sensio</div>
            <div className="brand-subtitle">私人 PDF 资料库</div>
          </div>
        </div>

        <section className="control-group">
          <div className="label-row">
            <label htmlFor="library-search">搜索</label>
            {hasSearch ? (
              <button className="text-button" type="button" onClick={() => setSearch("")}>
                清空
              </button>
            ) : null}
          </div>
          <input
            id="library-search"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="标题、文件名、路径"
          />
        </section>

        <section className="control-group">
          <div className="control-heading">
            <label htmlFor="import-path">导入 PDF</label>
            <span>复制到本地资料库，原文件保持不变。</span>
          </div>
          <textarea
            id="import-path"
            value={importPath}
            onChange={(event) => setImportPath(event.target.value)}
            placeholder="/Users/.../paper.pdf"
            rows={3}
          />
          <div className="button-row">
            <button type="button" onClick={selectImportPath}>
              选择 PDF
            </button>
            <button
              className="primary-button"
              type="button"
              disabled={!importPath.trim() || importMutation.isPending}
              onClick={() => importMutation.mutate()}
            >
              {importMutation.isPending ? "导入中..." : "导入"}
            </button>
          </div>
          {importMutation.isError ? (
            <p className="inline-error">{String(importMutation.error)}</p>
          ) : null}
        </section>

        <section className="control-group">
          <div className="control-heading">
            <label htmlFor="export-path">导出副本</label>
            <span>从当前选中文档导出一份独立副本。</span>
          </div>
          <textarea
            id="export-path"
            value={exportPath}
            onChange={(event) => setExportPath(event.target.value)}
            placeholder="/Users/.../copy.pdf"
            rows={3}
          />
          <div className="button-row">
            <button type="button" disabled={!selectedDocument} onClick={selectExportPath}>
              选择位置
            </button>
            <button
              type="button"
              disabled={!selectedDocument || !exportPath.trim() || exportMutation.isPending}
              onClick={() => exportMutation.mutate()}
            >
              {exportMutation.isPending ? "导出中..." : "导出"}
            </button>
          </div>
          {exportMutation.isSuccess ? (
            <p className="inline-success">已导出：{exportMutation.data}</p>
          ) : null}
          {exportMutation.isError ? (
            <p className="inline-error">{String(exportMutation.error)}</p>
          ) : null}
        </section>

        {dialogError ? <p className="inline-error">{dialogError}</p> : null}

        <div className="library-path" title={libraryPath ?? ""}>
          <span>资料库</span>
          <strong>{libraryPath ?? "路径未加载"}</strong>
        </div>
      </aside>

      <section className="library-panel">
        <header className="toolbar">
          <div className="toolbar-title">
            <p>私人资料库</p>
            <h1>文档陈列</h1>
            <div className="metric-row" aria-label="资料库统计">
              <span>{documents.data?.total ?? 0} 个文件</span>
              <span>{formatBytes(visibleFileSize)}</span>
            </div>
          </div>
          <div className="toolbar-actions">
            <select
              value={sortKey}
              onChange={(event) => updateSort(event.target.value as SortKey)}
              aria-label="排序"
            >
              {sortOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            <div className="segmented" aria-label="视图">
              <button
                type="button"
                className={layoutMode === "grid" ? "selected" : ""}
                aria-pressed={layoutMode === "grid"}
                onClick={() => updateLayout("grid")}
              >
                网格
              </button>
              <button
                type="button"
                className={layoutMode === "list" ? "selected" : ""}
                aria-pressed={layoutMode === "list"}
                onClick={() => updateLayout("list")}
              >
                列表
              </button>
            </div>
          </div>
        </header>

        {documents.isPending ? <div className="panel-state">正在读取文档...</div> : null}
        {documents.isError ? (
          <div className="panel-state error">{String(documents.error)}</div>
        ) : null}
        {documents.data ? (
          <DocumentCollection
            documents={documents.data.items}
            layoutMode={layoutMode}
            selectedDocumentId={selectedDocument?.id ?? null}
            onSelect={setSelectedDocumentId}
            hasSearch={hasSearch}
          />
        ) : null}
      </section>

      <section className="reader-preview">
        {selectedDocument ? (
          <>
            <header className="reader-preview-header">
              <div>
                <p>预览</p>
                <h2>{selectedDocument.title}</h2>
                <span>{selectedDocument.file_name}</span>
              </div>
              <button
                className="primary-button"
                type="button"
                onClick={() => {
                  void commands.openDocumentWindow(selectedDocument.id);
                  window.location.hash = readerHash(selectedDocument.id);
                }}
              >
                进入阅读
              </button>
            </header>
            <PdfViewer documentId={selectedDocument.id} compact />
          </>
        ) : (
          <div className="empty-reader">导入或选择一个 PDF</div>
        )}
      </section>
    </main>
  );
}

type DocumentCollectionProps = {
  documents: DocumentRecord[];
  layoutMode: LayoutMode;
  selectedDocumentId: string | null;
  onSelect: (documentId: string) => void;
  hasSearch: boolean;
};

function DocumentCollection({
  documents,
  layoutMode,
  selectedDocumentId,
  onSelect,
  hasSearch,
}: DocumentCollectionProps) {
  if (documents.length === 0) {
    return (
      <div className="panel-state empty-state">
        <div className="empty-state-mark" aria-hidden="true">
          PDF
        </div>
        <h2>{hasSearch ? "没有匹配文档" : "还没有文档"}</h2>
        <p>{hasSearch ? "调整搜索条件后再试。" : "导入 PDF 后，它会安静地陈列在这里。"}</p>
      </div>
    );
  }

  return (
    <div className={layoutMode === "grid" ? "document-grid" : "document-list"}>
      {documents.map((document) => (
        <button
          key={document.id}
          className={document.id === selectedDocumentId ? "document-card selected" : "document-card"}
          type="button"
          aria-pressed={document.id === selectedDocumentId}
          onClick={() => onSelect(document.id)}
          title={document.title}
        >
          <div className="document-thumb" aria-hidden="true">
            {getDocumentInitial(document.title)}
          </div>
          <div className="document-meta">
            <strong title={document.title}>{document.title}</strong>
            <span title={document.file_name}>{document.file_name}</span>
            <div className="document-facts">
              <small>
                <span>大小</span>
                {formatBytes(document.file_size)}
              </small>
              <small>
                <span>入库</span>
                {formatDate(document.imported_at)}
              </small>
            </div>
          </div>
        </button>
      ))}
    </div>
  );
}

function formatBytes(size: number) {
  if (size < 1024) {
    return `${size} B`;
  }
  if (size < 1024 * 1024) {
    return `${(size / 1024).toFixed(1)} KB`;
  }
  return `${(size / 1024 / 1024).toFixed(1)} MB`;
}

function formatDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value.slice(0, 10);
  }

  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

function getDocumentInitial(title: string) {
  return Array.from(title.trim())[0]?.toUpperCase() ?? "P";
}
