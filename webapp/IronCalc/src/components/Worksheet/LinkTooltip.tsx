import type { CellLink } from "@ironcalc/wasm";
import { Copy, Link, Link2Off, PencilLine } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { LinkHoverCell } from "../WorksheetCanvas/cellLinks";
import {
  headerColumnWidth,
  headerRowHeight,
} from "../WorksheetCanvas/constants";

function linkLabel(link: CellLink): string {
  if (link.type === "Internal") {
    return link.location;
  }
  const target = link.target;
  try {
    const url = new URL(target);
    if (url.protocol === "http:" || url.protocol === "https:") {
      return url.hostname.replace(/^www\./, "");
    }
    if (url.protocol === "mailto:") {
      return url.pathname;
    }
  } catch {
    // not a valid URL: fall through and show the raw target
  }
  return target;
}

const LinkTooltip = (props: {
  cell: LinkHoverCell;
  onFollow: (link: CellLink) => void;
  onEdit?: (row: number, column: number) => void;
  onDelete?: (row: number, column: number) => void;
  onHide: () => void;
  onPointerEnter: () => void;
  onPointerLeave: () => void;
}) => {
  const { cell, onFollow, onEdit, onDelete, onHide } = props;
  const { t } = useTranslation();
  const rootElement = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState<number | null>(null);

  // The tooltip is kept invisible on its first paint so it can be measured
  // and positioned above (or below) the cell before it is shown.
  useLayoutEffect(() => {
    setHeight(rootElement.current?.offsetHeight || 26);
  }, []);

  const link = cell.link;

  // Position the tooltip above the cell (or below if there is no room),
  // flush with the cell edge: any gap in between would make the pointer
  // cross a neighboring cell on its way to the tooltip, hiding it (or
  // showing the neighbor's tooltip) before it can be reached.
  let top = cell.y - (height ?? 26) + 1;
  if (top < headerRowHeight) {
    top = cell.y + cell.height - 1;
  }
  const left = Math.max(headerColumnWidth + 2, cell.x);

  return (
    <div
      ref={rootElement}
      className="ic-worksheet-link-tooltip"
      style={{ left, top, visibility: height === null ? "hidden" : "visible" }}
      onPointerEnter={props.onPointerEnter}
      onPointerLeave={props.onPointerLeave}
      // Interacting with the tooltip must not select the cells underneath
      onPointerDown={(event) => event.stopPropagation()}
    >
      <span className="ic-worksheet-link-tooltip-icon">
        <Link size={12} />
      </span>
      <button
        type="button"
        className="ic-worksheet-link-tooltip-label"
        title={link.tooltip || ""}
        onClick={() => {
          onHide();
          onFollow(link);
        }}
      >
        {linkLabel(link)}
      </button>
      <button
        type="button"
        className="ic-worksheet-link-tooltip-button"
        title={t("link_tooltip.copy")}
        onClick={() => {
          const text = link.type === "External" ? link.target : link.location;
          navigator.clipboard?.writeText(text).catch(() => {
            // clipboard access denied or unavailable: nothing to do
          });
        }}
      >
        <Copy size={12} />
      </button>
      {/* Dynamic links (created by formulas like HYPERLINK) cannot be edited
          or deleted: only the formula itself can change them. */}
      {onEdit && !link.dynamic && (
        <button
          type="button"
          className="ic-worksheet-link-tooltip-button"
          title={t("link_tooltip.edit")}
          onClick={() => {
            onHide();
            onEdit(cell.row, cell.column);
          }}
        >
          <PencilLine size={12} />
        </button>
      )}
      {onDelete && !link.dynamic && (
        <button
          type="button"
          className="ic-worksheet-link-tooltip-button"
          title={t("link_tooltip.break")}
          onClick={() => {
            onHide();
            onDelete(cell.row, cell.column);
          }}
        >
          <Link2Off size={12} />
        </button>
      )}
    </div>
  );
};

export default LinkTooltip;
