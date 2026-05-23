import type { CellStyle, FmtSettings, NamedStyle } from "@ironcalc/wasm";
import { ArrowLeft, Plus, X } from "lucide-react";
import type { CSSProperties } from "react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import { Tooltip } from "../../Tooltip/Tooltip";
import EditNamedStyle, { type NamedStyleSavePayload } from "./EditNamedStyle";
import "./named-styles.css";

type CategoryType = "grid" | "themed";

interface Category {
  label: string;
  type: CategoryType;
  names: string[];
}

const BUILTIN_CATEGORIES: Category[] = [
  {
    label: "Good, Bad and Neutral",
    type: "grid",
    names: ["normal", "bad", "good", "neutral"],
  },
  {
    label: "Data and Model",
    type: "grid",
    names: [
      "calculation",
      "check cell",
      "explanatory text",
      "input",
      "linked cell",
      "note",
      "output",
      "warning text",
    ],
  },
  {
    label: "Titles and Headings",
    type: "grid",
    names: [
      "heading 1",
      "heading 2",
      "heading 3",
      "heading 4",
      "title",
      "total",
    ],
  },
  {
    label: "Themed Cell Styles",
    type: "themed",
    // Each group of 4 = one accent (20% → 40% → 60% → 100%)
    names: [
      "20% - accent1",
      "40% - accent1",
      "60% - accent1",
      "80% - accent1",
      "accent1",
      "20% - accent2",
      "40% - accent2",
      "60% - accent2",
      "80% - accent2",
      "accent2",
      "20% - accent3",
      "40% - accent3",
      "60% - accent3",
      "80% - accent3",
      "accent3",
      "20% - accent4",
      "40% - accent4",
      "60% - accent4",
      "80% - accent4",
      "accent4",
      "20% - accent5",
      "40% - accent5",
      "60% - accent5",
      "80% - accent5",
      "accent5",
      "20% - accent6",
      "40% - accent6",
      "60% - accent6",
      "80% - accent6",
      "accent6",
    ],
  },
  {
    label: "Number Format",
    type: "grid",
    names: ["comma", "comma [0]", "currency", "currency [0]", "percent"],
  },
];

interface StyleGroup {
  label: string;
  type: CategoryType;
  styles: NamedStyle[];
}

function groupStyles(
  customStyles: NamedStyle[],
  builtinStyles: NamedStyle[],
): StyleGroup[] {
  const byLowerName = new Map(
    builtinStyles.map((s) => [s.name.toLowerCase(), s]),
  );
  const result: StyleGroup[] = [];
  if (customStyles.length > 0) {
    result.push({ label: "Custom", type: "grid", styles: customStyles });
  }
  for (const cat of BUILTIN_CATEGORIES) {
    const styles = cat.names
      .map((n) => byLowerName.get(n))
      .filter((s): s is NamedStyle => s !== undefined);
    if (styles.length > 0) {
      result.push({ label: cat.label, type: cat.type, styles });
    }
  }
  return result;
}

const BORDER_WIDTH: Record<string, string> = {
  thin: "1px",
  medium: "2px",
  thick: "3px",
  double: "3px",
  dotted: "1px",
  slantdashdot: "1px",
  mediumdashed: "2px",
  mediumdashdotdot: "2px",
  mediumdashdot: "2px",
};

const BORDER_CSS_STYLE: Record<string, string> = {
  thin: "solid",
  medium: "solid",
  thick: "solid",
  double: "double",
  dotted: "dotted",
  slantdashdot: "dashed",
  mediumdashed: "dashed",
  mediumdashdotdot: "dashed",
  mediumdashdot: "dashed",
};

interface BorderItem {
  style: string;
  color: string;
}

function getBorderValue(item: BorderItem | undefined): string | undefined {
  if (!item?.style) {
    return undefined;
  }
  const width = BORDER_WIDTH[item.style] ?? "1px";
  const cssStyle = BORDER_CSS_STYLE[item.style] ?? "solid";
  return `${width} ${cssStyle} ${item.color || "currentColor"}`;
}

function getTileStyle(style: CellStyle): CSSProperties {
  const decorations: string[] = [];
  if (style.font.u) {
    decorations.push("underline");
  }
  if (style.font.strike) {
    decorations.push("line-through");
  }
  return {
    backgroundColor: style.fill.fg_color || undefined,
    color: style.font.color || undefined,
    fontWeight: style.font.b ? "bold" : undefined,
    fontStyle: style.font.i ? "italic" : undefined,
    textDecoration: decorations.length > 0 ? decorations.join(" ") : undefined,
    borderTop: getBorderValue(style.border?.top),
    borderRight: getBorderValue(style.border?.right),
    borderBottom: getBorderValue(style.border?.bottom),
    borderLeft: getBorderValue(style.border?.left),
  };
}

const ACCENT_LABELS = [
  "Accent 1",
  "Accent 2",
  "Accent 3",
  "Accent 4",
  "Accent 5",
  "Accent 6",
];

interface NamedStylesPanelProps {
  customStyles: NamedStyle[];
  builtinStyles: NamedStyle[];
  formatOptions: FmtSettings;
  onApplyNamedStyle: (name: string) => void;
  onAddNamedStyle: (payload: NamedStyleSavePayload) => void;
  onClose: () => void;
}

const NamedStylesPanel = ({
  customStyles,
  builtinStyles,
  formatOptions,
  onApplyNamedStyle,
  onAddNamedStyle,
  onClose,
}: NamedStylesPanelProps) => {
  const { t } = useTranslation();
  const [isCreatingNew, setIsCreatingNew] = useState(false);

  const groups = groupStyles(customStyles, builtinStyles);

  const handleNewClick = () => setIsCreatingNew(true);
  const handleClose = () => setIsCreatingNew(false);

  const renderTile = ({ name, style }: NamedStyle, label?: string) => (
    <button
      key={name}
      type="button"
      className="ic-named-styles-tile"
      style={getTileStyle(style)}
      onClick={() => onApplyNamedStyle(name)}
    >
      <span className="ic-named-styles-tile-text">{label ?? name}</span>
    </button>
  );

  const renderGroupContent = (group: StyleGroup) => {
    if (group.type === "themed") {
      return (
        <div className="ic-named-styles-rows">
          {ACCENT_LABELS.map((accentLabel, i) => {
            const accentSuffix = `accent${i + 1}`;
            const row = group.styles.filter((s) =>
              s.name.toLowerCase().endsWith(accentSuffix),
            );
            if (row.length === 0) return null;
            return (
              <div key={accentLabel} className="ic-named-styles-row">
                {row.map((s) => {
                  const pct = s.name.match(/^(\d+%)/);
                  return renderTile(s, pct ? pct[1] : accentLabel);
                })}
              </div>
            );
          })}
        </div>
      );
    }

    return (
      <div className="ic-named-styles-grid">
        {group.styles.map((s) => renderTile(s))}
      </div>
    );
  };

  if (isCreatingNew) {
    return (
      <div className="ic-named-styles-container">
        <div className="ic-named-styles-edit-header">
          <Tooltip title={t("named_styles.back_to_list")}>
            <IconButton
              icon={<ArrowLeft />}
              onClick={handleClose}
              aria-label={t("named_styles.back_to_list")}
            />
          </Tooltip>
          <div className="ic-named-styles-edit-header-title">
            {t("named_styles.add_new_style")}
          </div>
          <Tooltip title={t("right_drawer.close")}>
            <IconButton
              icon={<X />}
              onClick={onClose}
              aria-label={t("right_drawer.close")}
            />
          </Tooltip>
        </div>
        <div className="ic-named-styles-content">
          <EditNamedStyle
            name=""
            style={
              builtinStyles.find((s) => s.name.toLowerCase() === "normal")
                ?.style ?? builtinStyles[0]?.style
            }
            formatOptions={formatOptions}
            existingStyleNames={[
              ...customStyles.map((s) => s.name),
              ...builtinStyles.map((s) => s.name),
            ]}
            onSave={(payload) => {
              onAddNamedStyle(payload);
              return { nameError: "" };
            }}
            onClose={handleClose}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="ic-named-styles-container">
      <div className="ic-named-styles-header">
        <div className="ic-named-styles-header-title">
          {t("named_styles.panel_title")}
        </div>
        <Tooltip title={t("right_drawer.close")}>
          <IconButton
            icon={<X />}
            onClick={onClose}
            aria-label={t("right_drawer.close")}
          />
        </Tooltip>
      </div>
      <div className="ic-named-styles-content">
        {groups.map((group) => (
          <div key={group.label} className="ic-named-styles-section">
            <div className="ic-named-styles-section-title">{group.label}</div>
            {renderGroupContent(group)}
          </div>
        ))}
      </div>
      <div className="ic-named-styles-footer">
        <Button startIcon={<Plus />} onClick={handleNewClick}>
          Add new
        </Button>
      </div>
    </div>
  );
};

export default NamedStylesPanel;
