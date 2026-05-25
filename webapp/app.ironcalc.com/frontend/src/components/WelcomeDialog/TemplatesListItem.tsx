import "./welcome-dialog.css";

interface TemplatesListItemProps {
  title: string;
  category: string;
  active: boolean;
  thumbnailUrl?: string;
  onClick: () => void;
}

function TemplatesListItem({
  title,
  category,
  active,
  thumbnailUrl,
  onClick,
}: TemplatesListItemProps) {
  return (
    <button
      type="button"
      className={`app-ic-wd-list-item${active ? " app-ic-wd-list-item--active" : ""}`}
      aria-pressed={active}
      onClick={onClick}
    >
      <div className="app-ic-wd-list-item-thumbnail">
        {thumbnailUrl && (
          <img
            src={thumbnailUrl}
            alt={title}
            className="app-ic-wd-list-item-thumbnail-img"
          />
        )}
      </div>
      <div className="app-ic-wd-list-item-text">
        <div className="app-ic-wd-list-item-title">{title}</div>
        <div className="app-ic-wd-list-item-description">{category}</div>
      </div>
    </button>
  );
}

export default TemplatesListItem;
