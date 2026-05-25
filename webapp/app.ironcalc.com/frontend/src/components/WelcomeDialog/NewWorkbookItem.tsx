import type { ReactNode } from "react";
import "./welcome-dialog.css";

interface NewWorkbookItemProps {
  title: string;
  description: string;
  icon: ReactNode;
  active: boolean;
  onClick: () => void;
}

function NewWorkbookItem({
  title,
  description,
  icon,
  active,
  onClick,
}: NewWorkbookItemProps) {
  return (
    <button
      type="button"
      className={`app-ic-wd-new-item${active ? " app-ic-wd-new-item--active" : ""}`}
      aria-pressed={active}
      onClick={onClick}
    >
      <div className="app-ic-wd-new-item-icon">{icon}</div>
      <div className="app-ic-wd-new-item-text">
        <div className="app-ic-wd-list-item-title">{title}</div>
        <div className="app-ic-wd-list-item-description">{description}</div>
      </div>
    </button>
  );
}

export default NewWorkbookItem;
