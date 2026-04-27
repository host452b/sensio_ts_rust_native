import { useEffect, useMemo, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  commands,
  type HighlightInput,
  type HighlightRecord,
  type HighlightRect,
} from "../bindings";
import { pdfjs } from "../lib/pdf";
import type { TextItem } from "pdfjs-dist/types/src/display/api";

type PdfViewerProps = {
  documentId: string;
  compact?: boolean;
};

type PdfDocument = Awaited<ReturnType<typeof pdfjs.getDocument>["promise"]>;
type TextLayerItem = {
  text: string;
  left: number;
  top: number;
  width: number;
  height: number;
};

export function PdfViewer({ documentId, compact = false }: PdfViewerProps) {
  const queryClient = useQueryClient();
  const [pdfDocument, setPdfDocument] = useState<PdfDocument | null>(null);
  const [scale, setScale] = useState(compact ? 0.82 : 1.18);

  const pdfBytes = useQuery({
    queryKey: ["pdf-bytes", documentId],
    queryFn: async () => {
      const bytes = await commands.readPdfBytes(documentId);
      return new Uint8Array(bytes);
    },
  });

  const highlights = useQuery({
    queryKey: ["highlights", documentId],
    queryFn: () => commands.listHighlights(documentId, null),
  });

  const addHighlight = useMutation({
    mutationFn: (input: HighlightInput) => commands.addHighlight(input),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["highlights", documentId] });
    },
  });

  useEffect(() => {
    if (!pdfBytes.data) {
      setPdfDocument(null);
      return;
    }

    let cancelled = false;
    const loadingTask = pdfjs.getDocument({ data: pdfBytes.data });

    void loadingTask.promise.then((nextDocument) => {
      if (!cancelled) {
        setPdfDocument(nextDocument);
      }
    });

    return () => {
      cancelled = true;
      void loadingTask.destroy();
    };
  }, [pdfBytes.data]);

  const highlightsByPage = useMemo(() => {
    const grouped = new Map<number, HighlightRecord[]>();
    for (const highlight of highlights.data ?? []) {
      const pageHighlights = grouped.get(highlight.page_index) ?? [];
      pageHighlights.push(highlight);
      grouped.set(highlight.page_index, pageHighlights);
    }
    return grouped;
  }, [highlights.data]);

  if (pdfBytes.isPending) {
    return <div className="pdf-state">正在加载 PDF...</div>;
  }

  if (pdfBytes.isError) {
    return <div className="pdf-state error">{String(pdfBytes.error)}</div>;
  }

  if (!pdfDocument) {
    return <div className="pdf-state">正在解析 PDF...</div>;
  }

  const pages = Array.from({ length: pdfDocument.numPages }, (_, index) => index + 1);

  return (
    <div className={compact ? "pdf-viewer compact" : "pdf-viewer"}>
      <div className="pdf-toolbar">
        <button
          type="button"
          aria-label="缩小"
          onClick={() => setScale((value) => Math.max(0.55, value - 0.1))}
        >
          −
        </button>
        <span aria-live="polite">{Math.round(scale * 100)}%</span>
        <button
          type="button"
          aria-label="放大"
          onClick={() => setScale((value) => Math.min(2.2, value + 0.1))}
        >
          +
        </button>
      </div>
      <div className="pdf-pages">
        {pages.map((pageNumber) => (
          <PdfPage
            key={`${documentId}-${pageNumber}-${scale}`}
            pdfDocument={pdfDocument}
            pageNumber={pageNumber}
            scale={scale}
            highlights={highlightsByPage.get(pageNumber - 1) ?? []}
            onHighlight={(rects) =>
              addHighlight.mutate({
                document_id: documentId,
                page_index: pageNumber - 1,
                rects,
                color: "#f6c453",
                note: null,
              })
            }
          />
        ))}
      </div>
    </div>
  );
}

type PdfPageProps = {
  pdfDocument: PdfDocument;
  pageNumber: number;
  scale: number;
  highlights: HighlightRecord[];
  onHighlight: (rects: HighlightRect[]) => void;
};

function PdfPage({ pdfDocument, pageNumber, scale, highlights, onHighlight }: PdfPageProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const pageRef = useRef<HTMLDivElement | null>(null);
  const [size, setSize] = useState({ width: 0, height: 0 });
  const [textItems, setTextItems] = useState<TextLayerItem[]>([]);

  useEffect(() => {
    let cancelled = false;

    async function renderPage() {
      const page = await pdfDocument.getPage(pageNumber);
      const viewport = page.getViewport({ scale });
      const canvas = canvasRef.current;

      if (!canvas || cancelled) {
        return;
      }

      const context = canvas.getContext("2d");
      const outputScale = window.devicePixelRatio || 1;
      canvas.width = Math.floor(viewport.width * outputScale);
      canvas.height = Math.floor(viewport.height * outputScale);
      canvas.style.width = `${viewport.width}px`;
      canvas.style.height = `${viewport.height}px`;
      setSize({ width: viewport.width, height: viewport.height });

      if (!context) {
        return;
      }

      await page.render({
        canvas: null,
        canvasContext: context,
        viewport,
        transform: outputScale === 1 ? undefined : [outputScale, 0, 0, outputScale, 0, 0],
      }).promise;

      const textContent = await page.getTextContent();
      if (cancelled) {
        return;
      }
      setTextItems(
        textContent.items
          .filter(isTextItem)
          .map((item) => {
            const transform = pdfjs.Util.transform(viewport.transform, item.transform);
            const height = Math.hypot(transform[2], transform[3]);
            return {
              text: item.str,
              left: transform[4],
              top: transform[5] - height,
              width: item.width * scale,
              height,
            };
          }),
      );
    }

    void renderPage();

    return () => {
      cancelled = true;
    };
  }, [pdfDocument, pageNumber, scale]);

  const captureSelection = () => {
    const pageElement = pageRef.current;
    const selection = window.getSelection();
    if (!pageElement || !selection || selection.isCollapsed || selection.rangeCount === 0) {
      return;
    }

    const pageBounds = pageElement.getBoundingClientRect();
    const rects = Array.from(selection.getRangeAt(0).getClientRects())
      .map((rect) => intersectRect(rect, pageBounds))
      .filter((rect): rect is DOMRect => Boolean(rect))
      .map((rect) => ({
        left: (rect.left - pageBounds.left) / pageBounds.width,
        top: (rect.top - pageBounds.top) / pageBounds.height,
        width: rect.width / pageBounds.width,
        height: rect.height / pageBounds.height,
      }))
      .filter((rect) => rect.width > 0.002 && rect.height > 0.002);

    if (rects.length > 0) {
      onHighlight(rects);
      selection.removeAllRanges();
    }
  };

  return (
    <div className="pdf-page-shell">
      <div className="page-number">第 {pageNumber} 页</div>
      <div
        ref={pageRef}
        className="pdf-page"
        style={{ width: size.width, height: size.height }}
        onMouseUp={captureSelection}
      >
        <canvas ref={canvasRef} />
        <div className="text-layer" aria-hidden="true">
          {textItems.map((item, index) => (
            <span
              key={`${item.left}-${item.top}-${index}`}
              style={{
                left: item.left,
                top: item.top,
                width: item.width,
                height: item.height,
                fontSize: item.height,
              }}
            >
              {item.text}
            </span>
          ))}
        </div>
        <div className="highlight-layer" aria-hidden="true">
          {highlights.flatMap((highlight) =>
            highlight.rects.map((rect, index) => (
              <span
                key={`${highlight.id}-${index}`}
                className="highlight-rect"
                style={{
                  left: `${rect.left * 100}%`,
                  top: `${rect.top * 100}%`,
                  width: `${rect.width * 100}%`,
                  height: `${rect.height * 100}%`,
                  backgroundColor: highlight.color,
                }}
              />
            )),
          )}
        </div>
      </div>
    </div>
  );
}

function isTextItem(item: unknown): item is TextItem {
  return typeof item === "object" && item !== null && "str" in item;
}

function intersectRect(rect: DOMRect, bounds: DOMRect): DOMRect | null {
  const left = Math.max(rect.left, bounds.left);
  const top = Math.max(rect.top, bounds.top);
  const right = Math.min(rect.right, bounds.right);
  const bottom = Math.min(rect.bottom, bounds.bottom);

  if (right <= left || bottom <= top) {
    return null;
  }

  return new DOMRect(left, top, right - left, bottom - top);
}
