export type Route =
  | { name: "library" }
  | { name: "reader"; documentId: string };

export function parseHashRoute(hash: string): Route {
  const normalized = hash.replace(/^#\/?/, "");
  const [first, second] = normalized.split("/");

  if (first === "reader" && second) {
    return { name: "reader", documentId: decodeURIComponent(second) };
  }

  return { name: "library" };
}

export function readerHash(documentId: string): string {
  return `#/reader/${encodeURIComponent(documentId)}`;
}
