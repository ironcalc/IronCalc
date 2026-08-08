import { useTranslation } from "react-i18next";
import TemplatesListItem from "./TemplatesListItem";
import { TEMPLATES } from "./templates";
import "./welcome-dialog.css";

function TemplatesList(props: {
  selectedTemplate: string;
  handleTemplateSelect: (templateId: string) => void;
  categoryFilter?: string;
  limit?: number;
  columns?: number;
  onScroll?: React.UIEventHandler<HTMLDivElement>;
  lastItemRef?: React.Ref<HTMLButtonElement>;
}) {
  const {
    selectedTemplate,
    handleTemplateSelect,
    categoryFilter,
    limit,
    columns,
    onScroll,
    lastItemRef,
  } = props;
  const { t } = useTranslation();

  const filtered =
    !categoryFilter || categoryFilter === "all"
      ? TEMPLATES
      : TEMPLATES.filter((tmpl) => tmpl.categoryId === categoryFilter);

  const visible = limit ? filtered.slice(0, limit) : filtered;

  return (
    <div
      className="app-ic-wd-templates-list"
      onScroll={onScroll}
      style={
        columns !== undefined
          ? { gridTemplateColumns: `repeat(${columns}, 1fr)` }
          : undefined
      }
    >
      {visible.map((tmpl, index) => (
        <TemplatesListItem
          key={tmpl.id}
          ref={index === visible.length - 1 ? lastItemRef : undefined}
          title={t(tmpl.titleKey)}
          category={t(tmpl.categoryKey)}
          active={selectedTemplate === tmpl.id}
          thumbnailUrl={`/templates/${tmpl.id}.png`}
          onClick={() => handleTemplateSelect(tmpl.id)}
        />
      ))}
    </div>
  );
}

export default TemplatesList;
