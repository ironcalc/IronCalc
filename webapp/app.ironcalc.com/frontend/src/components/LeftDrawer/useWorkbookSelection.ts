import { useEffect, useState } from "react";

interface Options {
  onDelete: (uuid: string) => void;
}

export function useWorkbookSelection({ onDelete }: Options) {
  const [checkedUuids, setCheckedUuids] = useState<Set<string>>(new Set());
  const [lastCheckedUuid, setLastCheckedUuid] = useState<string | null>(null);

  const hasAnyChecked = checkedUuids.size > 0;

  useEffect(() => {
    if (checkedUuids.size === 0) setLastCheckedUuid(null);
  }, [checkedUuids.size]);

  const handleCheckboxClick = (
    uuid: string,
    shiftKey: boolean,
    orderedUuids: string[],
  ) => {
    if (shiftKey && lastCheckedUuid) {
      const fromIdx = orderedUuids.indexOf(lastCheckedUuid);
      const toIdx = orderedUuids.indexOf(uuid);
      if (fromIdx !== -1 && toIdx !== -1) {
        const [start, end] =
          fromIdx < toIdx ? [fromIdx, toIdx] : [toIdx, fromIdx];
        setCheckedUuids((prev) => {
          const next = new Set(prev);
          for (const id of orderedUuids.slice(start, end + 1)) next.add(id);
          return next;
        });
        return;
      }
    }
    setCheckedUuids((prev) => {
      const next = new Set(prev);
      if (next.has(uuid)) next.delete(uuid);
      else next.add(uuid);
      return next;
    });
    setLastCheckedUuid(uuid);
  };

  const handleDeleteChecked = () => {
    for (const uuid of checkedUuids) {
      onDelete(uuid);
    }
    setCheckedUuids(new Set());
  };

  const handleCancelChecked = () => {
    setCheckedUuids(new Set());
  };

  return {
    checkedUuids,
    hasAnyChecked,
    handleCheckboxClick,
    handleDeleteChecked,
    handleCancelChecked,
  };
}
