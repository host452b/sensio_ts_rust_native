import { useQuery } from "@tanstack/react-query";
import { commands } from "../bindings";
import { PdfViewer } from "./PdfViewer";

type ReaderPageProps = {
  documentId: string;
};

export function ReaderPage({ documentId }: ReaderPageProps) {
  const documents = useQuery({
    queryKey: ["documents", "reader", documentId],
    queryFn: () =>
      commands.listDocuments({
        search: null,
        tag_ids: [],
        sort_key: "recent",
        offset: 0,
        limit: 500,
      }),
  });

  const document = documents.data?.items.find((item) => item.id === documentId);

  return (
    <main className="reader-page">
      <header className="reader-header">
        <button className="text-button" type="button" onClick={() => (window.location.hash = "#/")}>
          资料库
        </button>
        <div className="reader-header-title">
          <p>阅读器</p>
          <h1>{document?.title ?? "PDF 阅读器"}</h1>
          <span>{document?.file_name ?? documentId}</span>
        </div>
      </header>
      <PdfViewer documentId={documentId} />
    </main>
  );
}
