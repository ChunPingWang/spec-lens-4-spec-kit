import { marked } from "marked";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { usePhaseStore } from "@/stores";
import type { Uuid } from "@/types/ipc";

interface DocumentPreviewProps {
  projectId: Uuid;
  relativePath: string | null;
}

/**
 * Renders the currently selected document as Markdown.
 * Uses `marked` for parsing; highlight.js is loaded lazily for code blocks.
 */
export function DocumentPreview({ projectId, relativePath }: DocumentPreviewProps) {
  const { t } = useTranslation();
  const readDocument = usePhaseStore((s) => s.readDocument);
  const cached = usePhaseStore((s) =>
    relativePath ? s.documentCache[relativePath] : undefined,
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!relativePath) return;
    if (cached) return;
    let cancelled = false;
    setLoading(true);
    setError(null);
    void (async () => {
      try {
        await readDocument(projectId, relativePath);
      } catch (err) {
        if (!cancelled) setError(err instanceof Error ? err.message : String(err));
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [projectId, relativePath, readDocument, cached]);

  const html = useMemo(() => {
    if (!cached?.content) return "";
    return marked.parse(cached.content, { async: false }) as string;
  }, [cached?.content]);

  if (!relativePath) {
    return (
      <p className="p-4 text-sm text-muted-foreground">
        {t("phase.selectDocumentHint", {
          defaultValue: "Select a document on the left to preview it.",
        })}
      </p>
    );
  }

  if (loading) {
    return (
      <p className="p-4 text-sm text-muted-foreground">
        {t("phase.loadingDocument", { defaultValue: "Loading document…" })}
      </p>
    );
  }

  if (error) {
    return <p className="p-4 text-sm text-rose-500">{error}</p>;
  }

  return (
    <article className="prose prose-sm max-w-none px-4 py-3 dark:prose-invert">
      {cached?.truncated && (
        <div className="mb-2 rounded-sm border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs text-amber-600">
          {t("phase.truncatedNotice", {
            defaultValue: "Preview truncated — open the file in your editor for the full content.",
          })}
        </div>
      )}
      <div dangerouslySetInnerHTML={{ __html: html }} />
    </article>
  );
}
