import "./welcome-dialog.css";

interface TemplatesListItemProps {
  title: string;
  category: string;
  active: boolean;
  thumbnailUrl?: string;
  onClick: () => void;
  ref?: React.Ref<HTMLButtonElement>;
}

function TemplatesListItem({
  title,
  category,
  active,
  thumbnailUrl,
  onClick,
  ref,
}: TemplatesListItemProps) {
  return (
    <button
      ref={ref}
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
            loading="lazy"
            decoding="async"
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
