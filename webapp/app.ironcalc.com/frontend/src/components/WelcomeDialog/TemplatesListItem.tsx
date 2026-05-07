import type { CSSProperties, ReactNode } from "react";
import "./welcome-dialog.css";

interface TemplatesListItemProps {
  title: string;
  description: string;
  icon: ReactNode;
  iconColor: string;
  active: boolean;
  onClick: () => void;
}

function TemplatesListItem({
  title,
  description,
  icon,
  iconColor,
  active,
  onClick,
}: TemplatesListItemProps) {
  return (
    <button
      type="button"
      className={`app-ic-wd-list-item${active ? " app-ic-wd-list-item--active" : ""}`}
      style={
        {
          "--item-color": iconColor,
          "--item-color-alpha": `${iconColor}24`,
        } as CSSProperties
      }
      aria-pressed={active}
      onClick={onClick}
    >
      <div className="app-ic-wd-list-item-icon">{icon}</div>
      <div className="app-ic-wd-list-item-body">
        <div className="app-ic-wd-list-item-title">{title}</div>
        <div className="app-ic-wd-list-item-description">{description}</div>
      </div>
      <div
        className={`app-ic-wd-radio${active ? " app-ic-wd-radio--active" : ""}`}
      >
        <div className="app-ic-wd-radio-dot" />
      </div>
    </button>
  );
}

export default TemplatesListItem;
