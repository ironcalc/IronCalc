import type { CellStyle, NamedStyle } from "@ironcalc/wasm";
import {
  type CSSProperties,
  type RefObject,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import "./named-styles.css";

// Known Excel built-in style name→category mapping (case-insensitive)
const BUILTIN_CATEGORIES: { label: string; names: string[] }[] = [
  {
    label: "Good, Bad and Neutral",
    names: ["normal", "bad", "good", "neutral"],
  },
  {
    label: "Data and Model",
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
    names: [
      "20% - accent1",
      "20% - accent2",
      "20% - accent3",
      "20% - accent4",
      "20% - accent5",
      "20% - accent6",
      "40% - accent1",
      "40% - accent2",
      "40% - accent3",
      "40% - accent4",
      "40% - accent5",
      "40% - accent6",
      "60% - accent1",
      "60% - accent2",
      "60% - accent3",
      "60% - accent4",
      "60% - accent5",
      "60% - accent6",
      "accent1",
      "accent2",
      "accent3",
      "accent4",
      "accent5",
      "accent6",
    ],
  },
  {
    label: "Number Format",
    names: ["comma", "comma [0]", "currency", "currency [0]", "percent"],
  },
];

function groupStyles(
  namedStyles: NamedStyle[],
): { label: string; styles: NamedStyle[] }[] {
  const byLowerName = new Map(
    namedStyles.map((s) => [s.name.toLowerCase(), s]),
  );
  const assigned = new Set<string>();
  const builtinGroups: { label: string; styles: NamedStyle[] }[] = [];

  for (const cat of BUILTIN_CATEGORIES) {
    const catStyles = cat.names
      .map((n) => byLowerName.get(n))
      .filter((s): s is NamedStyle => s !== undefined);
    if (catStyles.length > 0) {
      builtinGroups.push({ label: cat.label, styles: catStyles });
      for (const s of catStyles) assigned.add(s.name.toLowerCase());
    }
  }

  const custom = namedStyles.filter((s) => !assigned.has(s.name.toLowerCase()));

  const result: { label: string; styles: NamedStyle[] }[] = [];
  if (custom.length > 0) {
    result.push({ label: "Custom", styles: custom });
  }
  result.push(...builtinGroups);
  return result;
}

function getTileStyle(style: CellStyle): CSSProperties {
  const decorations: string[] = [];
  if (style.font.u) decorations.push("underline");
  if (style.font.strike) decorations.push("line-through");
  return {
    backgroundColor: style.fill.fg_color || undefined,
    color: style.font.color || undefined,
    fontWeight: style.font.b ? "bold" : undefined,
    fontStyle: style.font.i ? "italic" : undefined,
    textDecoration: decorations.length > 0 ? decorations.join(" ") : undefined,
  };
}

interface NamedStylesPanelProps {
  open: boolean;
  namedStyles: NamedStyle[];
  onApplyNamedStyle: (style: CellStyle) => void;
  onClose: () => void;
  anchorEl: RefObject<HTMLElement | null>;
}

const NamedStylesPanel = ({
  open,
  namedStyles,
  onApplyNamedStyle,
  onClose,
  anchorEl,
}: NamedStylesPanelProps) => {
  const panelRef = useRef<HTMLDivElement | null>(null);
  const [position, setPosition] = useState({ top: 0, left: 0 });
  const { t } = useTranslation();

  useLayoutEffect(() => {
    if (!open) return;

    const anchor = anchorEl.current;
    const panel = panelRef.current;
    if (!anchor || !panel) return;

    const updatePosition = () => {
      const anchorRect = anchor.getBoundingClientRect();
      const panelWidth = panel.offsetWidth;
      const panelHeight = panel.offsetHeight;
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;
      const margin = 8;
      const offset = 4;

      let left = anchorRect.left;
      let top = anchorRect.bottom + offset;

      if (left + panelWidth > viewportWidth - margin) {
        left = viewportWidth - panelWidth - margin;
      }
      if (left < margin) left = margin;
      if (top + panelHeight > viewportHeight - margin) {
        top = anchorRect.top - panelHeight - offset;
      }
      if (top < margin) top = margin;

      setPosition({ top, left });
    };

    updatePosition();
    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);
    return () => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [open, anchorEl]);

  if (!open) return null;

  const groups = groupStyles(namedStyles);

  return createPortal(
    <div className="ic-menu-layer">
      <div
        className="ic-menu-backdrop"
        onPointerDown={onClose}
        aria-hidden="true"
      />
      {/* biome-ignore lint/a11y/useKeyWithClickEvents: panel intercepts click only to prevent workbook focus steal */}
      <div
        ref={panelRef}
        className="ic-named-styles-panel"
        role="dialog"
        aria-modal="true"
        aria-label={t("toolbar.named_styles")}
        style={
          {
            top: `${position.top}px`,
            left: `${position.left}px`,
          } as CSSProperties
        }
        onPointerDown={(e) => e.stopPropagation()}
        onClick={(e) => e.stopPropagation()}
      >
        {groups.map((group) => (
          <div key={group.label} className="ic-named-styles-category">
            <div className="ic-named-styles-category-label">{group.label}</div>
            <div className="ic-named-styles-grid">
              {group.styles.map(({ name, style }) => (
                <button
                  key={name}
                  type="button"
                  className="ic-named-styles-tile"
                  style={getTileStyle(style)}
                  title={name}
                  onClick={() => {
                    onApplyNamedStyle(style);
                    onClose();
                  }}
                >
                  {name}
                </button>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>,
    document.body,
  );
};

export default NamedStylesPanel;
