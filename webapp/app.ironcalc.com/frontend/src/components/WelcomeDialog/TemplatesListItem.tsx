import type { ReactNode } from "react";

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
      className={`template-item${active ? " template-item--active" : ""}`}
      style={{ "--template-color": iconColor } as React.CSSProperties}
      onClick={onClick}
    >
      <span className="template-item-icon">{icon}</span>
      <span className="template-item-info">
        <span className="template-item-title">{title}</span>
        <span className="template-item-description">{description}</span>
      </span>
      <span
        className={`template-item-radio${active ? " template-item-radio--active" : ""}`}
      >
        <span className="template-item-radio-dot" />
      </span>
    </button>
  );
}

export default TemplatesListItem;
