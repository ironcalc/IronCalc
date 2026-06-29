import type {
  FmtSettings,
  IronCalcTheme,
  Model,
  NamedStyle,
} from "@ironcalc/wasm";
import { ArrowLeft, Plus, Settings2, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";
import { IconButton } from "../../Button/IconButton";
import { Tooltip } from "../../Tooltip/Tooltip";
import EditNamedStyle, {
  type NamedStyleSavePayload,
  type SaveError,
} from "./EditNamedStyle";
import ManageCustomStyles from "./ManageCustomStyles";
import "./named-styles.css";
import { getTileStyle } from "./named-styles-utils";

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

function getBuiltinGroups(builtinStyles: NamedStyle[]): StyleGroup[] {
  const byLowerName = new Map(
    builtinStyles.map((s) => [s.name.toLowerCase(), s]),
  );
  return BUILTIN_CATEGORIES.map((cat) => ({
    label: cat.label,
    type: cat.type,
    styles: cat.names
      .map((n) => byLowerName.get(n))
      .filter((s): s is NamedStyle => s !== undefined),
  })).filter((g) => g.styles.length > 0);
}

const ACCENT_LABELS = [
  "Accent 1",
  "Accent 2",
  "Accent 3",
  "Accent 4",
  "Accent 5",
  "Accent 6",
];

type PanelView =
  | { mode: "list" }
  | { mode: "create" }
  | { mode: "manage" }
  | { mode: "editing"; style: NamedStyle };

interface NamedStylesPanelProps {
  model: Model;
  customStyles: NamedStyle[];
  builtinStyles: NamedStyle[];
  formatOptions: FmtSettings;
  currentTheme: IronCalcTheme;
  onApplyNamedStyle: (name: string) => void;
  onAddNamedStyle: (payload: NamedStyleSavePayload) => SaveError;
  onUpdateNamedStyle: (
    originalName: string,
    payload: NamedStyleSavePayload,
  ) => SaveError;
  onDeleteNamedStyle: (name: string) => void;
  onClose: () => void;
}

const NamedStylesPanel = ({
  model,
  customStyles,
  builtinStyles,
  formatOptions,
  currentTheme,
  onApplyNamedStyle,
  onAddNamedStyle,
  onUpdateNamedStyle,
  onDeleteNamedStyle,
  onClose,
}: NamedStylesPanelProps) => {
  const { t } = useTranslation();
  const [view, setView] = useState<PanelView>({ mode: "list" });

  const normalStyle = builtinStyles.find(
    (s) => s.name.toLowerCase() === "normal",
  );
  const builtinGroups = getBuiltinGroups(builtinStyles);
  const allStyleNames = [
    ...customStyles.map((s) => s.name),
    ...builtinStyles.map((s) => s.name),
  ];

  const renderTile = ({ name, style }: NamedStyle, label?: string) => (
    <button
      key={name}
      type="button"
      className="ic-named-styles-tile"
      style={getTileStyle(model, style)}
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
            if (row.length === 0) {
              return null;
            }
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

  const renderSubHeader = (title: string, onBack?: () => void) => (
    <div className="ic-named-styles-edit-header">
      <Tooltip title={t("named_styles.back_to_list")}>
        <IconButton
          icon={<ArrowLeft />}
          onClick={onBack ?? (() => setView({ mode: "list" }))}
          aria-label={t("named_styles.back_to_list")}
        />
      </Tooltip>
      <div className="ic-named-styles-edit-header-title">{title}</div>
      <Tooltip title={t("right_drawer.close")}>
        <IconButton
          icon={<X />}
          onClick={onClose}
          aria-label={t("right_drawer.close")}
        />
      </Tooltip>
    </div>
  );

  if (view.mode === "create" && normalStyle) {
    return (
      <div className="ic-named-styles-container">
        {renderSubHeader(t("named_styles.add_new_style"))}
        <div className="ic-named-styles-content">
          <EditNamedStyle
            model={model}
            name=""
            style={normalStyle.style}
            formatOptions={formatOptions}
            existingStyleNames={allStyleNames}
            currentTheme={currentTheme}
            onSave={(payload) => {
              if (
                allStyleNames.some(
                  (n) => n.toLowerCase() === payload.name.toLowerCase(),
                )
              ) {
                return { nameError: t("named_styles.name_already_exists") };
              }
              return onAddNamedStyle(payload);
            }}
            onClose={() => setView({ mode: "list" })}
          />
        </div>
      </div>
    );
  }

  if (view.mode === "manage") {
    return (
      <div className="ic-named-styles-container">
        {renderSubHeader(t("named_styles.manage_styles"))}
        <div className="ic-named-styles-content">
          <ManageCustomStyles
            model={model}
            customStyles={customStyles}
            onEdit={(style) => setView({ mode: "editing", style })}
            onDelete={(style) => onDeleteNamedStyle(style.name)}
          />
        </div>
        <div className="ic-named-styles-footer">
          <Button
            startIcon={<Plus />}
            onClick={() => setView({ mode: "create" })}
          >
            {t("named_styles.add_new_style")}
          </Button>
        </div>
      </div>
    );
  }

  if (view.mode === "editing") {
    const { style: editingStyle } = view;
    const otherStyleNames = allStyleNames.filter(
      (n) => n.toLowerCase() !== editingStyle.name.toLowerCase(),
    );
    return (
      <div className="ic-named-styles-container">
        {renderSubHeader(t("named_styles.update_style"), () =>
          setView({ mode: "manage" }),
        )}
        <div className="ic-named-styles-content">
          <EditNamedStyle
            model={model}
            name={editingStyle.name}
            style={editingStyle.style}
            formatOptions={formatOptions}
            existingStyleNames={allStyleNames}
            currentTheme={currentTheme}
            onSave={(payload) => {
              if (
                otherStyleNames.some(
                  (n) => n.toLowerCase() === payload.name.toLowerCase(),
                )
              ) {
                return { nameError: t("named_styles.name_already_exists") };
              }
              return onUpdateNamedStyle(editingStyle.name, payload);
            }}
            onClose={() => setView({ mode: "manage" })}
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
        {customStyles.length > 0 && (
          <div className="ic-named-styles-section">
            <div
              className="ic-named-styles-section-title"
              style={{ height: "17px" }}
            >
              Custom
              <div className="ic-named-styles-section-title-actions">
                <Tooltip title={t("named_styles.manage_styles")}>
                  <IconButton
                    icon={<Settings2 size={14} />}
                    onClick={() => setView({ mode: "manage" })}
                    aria-label={t("named_styles.manage_styles")}
                  />
                </Tooltip>
              </div>
            </div>
            <div className="ic-named-styles-grid">
              {customStyles.map((s) => renderTile(s))}
            </div>
          </div>
        )}
        <div className="ic-named-styles-section">
          <div className="ic-named-styles-section-title">Theme</div>
          {builtinGroups.map((group) => (
            <div key={group.label} className="ic-named-styles-category">
              <div className="ic-named-styles-category-label">
                {group.label}
              </div>
              {renderGroupContent(group)}
            </div>
          ))}
        </div>
      </div>
      <div className="ic-named-styles-footer">
        <Button
          startIcon={<Plus />}
          onClick={() => setView({ mode: "create" })}
        >
          {t("named_styles.add_new_style")}
        </Button>
      </div>
    </div>
  );
};

export default NamedStylesPanel;
