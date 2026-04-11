import { useTranslation } from "react-i18next";

import { DocumentItem } from "./DocumentItem";
import type { PhaseDocument } from "@/types/ipc";

interface DocumentListProps {
  documents: PhaseDocument[];
  activePath: string | null;
  onSelect: (relativePath: string) => void;
}

/**
 * Document list for the current step. Shown inside the Documents
 * sub-page of `PhaseTabs`.
 */
export function DocumentList({ documents, activePath, onSelect }: DocumentListProps) {
  const { t } = useTranslation();
  if (documents.length === 0) {
    return (
      <p className="px-2 py-3 text-sm text-muted-foreground">
        {t("phase.noDocuments", { defaultValue: "This step has no tracked documents yet." })}
      </p>
    );
  }
  return (
    <ul className="space-y-1">
      {documents.map((doc) => (
        <li key={doc.relativePath}>
          <DocumentItem
            document={doc}
            active={doc.relativePath === activePath}
            onSelect={onSelect}
          />
        </li>
      ))}
    </ul>
  );
}
