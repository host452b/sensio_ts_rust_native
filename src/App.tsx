import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { commands } from "./bindings";
import { LibraryPage } from "./components/LibraryPage";
import { ReaderPage } from "./components/ReaderPage";
import { parseHashRoute } from "./lib/hashRoute";
import { normalizeLayoutMode, normalizeSortKey } from "./lib/preferences";
import { useLibraryStore } from "./state/libraryStore";

export function App() {
  const queryClient = useQueryClient();
  const setLibraryPath = useLibraryStore((state) => state.setLibraryPath);
  const setLayoutMode = useLibraryStore((state) => state.setLayoutMode);
  const setSortKey = useLibraryStore((state) => state.setSortKey);
  const [hash, setHash] = useState(window.location.hash);

  const route = useMemo(() => parseHashRoute(hash), [hash]);
  const bootstrap = useQuery({
    queryKey: ["bootstrap"],
    queryFn: commands.initApp,
  });

  useEffect(() => {
    if (bootstrap.data) {
      setLibraryPath(bootstrap.data.library_path);
    }
  }, [bootstrap.data, setLibraryPath]);

  useEffect(() => {
    if (!bootstrap.data) {
      return;
    }

    let cancelled = false;

    async function loadPersistedSettings() {
      const [layoutModeSetting, sortKeySetting] = await Promise.all([
        commands.getSetting("layoutMode"),
        commands.getSetting("sortKey"),
      ]);

      if (cancelled) {
        return;
      }

      setLayoutMode(normalizeLayoutMode(layoutModeSetting?.value));
      setSortKey(normalizeSortKey(sortKeySetting?.value));
    }

    void loadPersistedSettings();

    return () => {
      cancelled = true;
    };
  }, [bootstrap.data, setLayoutMode, setSortKey]);

  useEffect(() => {
    const onHashChange = () => setHash(window.location.hash);
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  useEffect(() => {
    const unlisten = Promise.all([
      listen("library_updated", () => {
        void queryClient.invalidateQueries({ queryKey: ["documents"] });
        void queryClient.invalidateQueries({ queryKey: ["search"] });
      }),
      listen("settings_updated", () => {
        void queryClient.invalidateQueries({ queryKey: ["settings"] });
      }),
    ]);

    return () => {
      void unlisten.then((callbacks) => callbacks.forEach((callback) => callback()));
    };
  }, [queryClient]);

  if (bootstrap.isPending) {
    return <div className="app-loading">正在初始化资料库...</div>;
  }

  if (bootstrap.isError) {
    return (
      <div className="app-error">
        <h1>初始化失败</h1>
        <pre>{String(bootstrap.error)}</pre>
      </div>
    );
  }

  return route.name === "reader" ? (
    <ReaderPage documentId={route.documentId} />
  ) : (
    <LibraryPage />
  );
}
