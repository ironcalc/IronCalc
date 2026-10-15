import { Button } from "@ironcalc/workbook";
import { Info } from "lucide-react";

import "./notice-bar.css";

// Full-width strip under the file bar with a short message and one action.
export function NoticeBar(properties: {
  message: string;
  actionLabel: string;
  onAction: () => void;
}) {
  return (
    <div className="app-ic-notice-bar" role="status">
      <Info />
      <span>{properties.message}</span>
      <Button size="sm" onClick={properties.onAction}>
        {properties.actionLabel}
      </Button>
    </div>
  );
}
