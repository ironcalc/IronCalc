// A minimal, presentational "formula helper" popup for IronCalc.
//
// It renders a `Completion` (see formulaCompletion.ts) as one of two cards:
//
//   * A list of functions matching the partial name (e.g. "=SU" -> SUM, SUMIF…)
//   * The signature / docs of the function whose arguments you are filling in
//     (e.g. "=SUM(" -> SUM ( number1 , [number2] , … ) + description/examples)
//
// Both cards also cover what the user defined in the workbook: named functions
// (with their description and the description of their parameters) and defined
// names (with their comment). Those carry a tag telling which of the two they
// are.
//
// The selected list row is controlled by the parent so it can be driven from
// either the mouse (hover) or the keyboard (arrow keys). See the editor
// integration in Editor.tsx and the standalone demo in FormulaHelper.stories.

import type { NamedFunction } from "@ironcalc/wasm";
import { ChevronUp } from "lucide-react";
import {
  Fragment,
  type MouseEvent as ReactMouseEvent,
  type PointerEvent as ReactPointerEvent,
  useCallback,
  useState,
} from "react";
import { IconButton } from "../Button/IconButton";
import {
  type Completion,
  type CompletionItem,
  type CompletionItemKind,
  displayArgName,
  docsUrl,
  lookup,
} from "./formulaCompletion";
import "./formulaHelper.css";

// The popup is rendered (via a portal) over the workbook while a cell is being
// edited. Without this, clicking inside it blurs the editor textarea — which
// commits the edit — and the click also bubbles to the workbook's onClick,
// moving the selection. preventDefault on pointer-down keeps focus on the
// textarea so editing continues; stopPropagation keeps the click from reaching
// the workbook. Buttons/links still receive their own click events.
function swallowPointerDown(event: ReactPointerEvent) {
  event.preventDefault();
  event.stopPropagation();
}

function swallowClick(event: ReactMouseEvent) {
  event.stopPropagation();
}

// The tag shown next to what the user defined in the workbook, telling a named
// function from a defined name. Built-in functions have none.
function KindTag({ kind }: { kind: CompletionItemKind }) {
  if (kind === "builtin") {
    return null;
  }
  return (
    <span className={`ic-fh-tag ic-fh-tag-${kind}`}>
      {kind === "function" ? "Function" : "Name"}
    </span>
  );
}

interface FormulaHelperProps {
  completion: Completion;
  // Index of the highlighted row in list mode (controlled by the parent).
  selected: number;
  // Called when a row is hovered, so the parent can update `selected`.
  onSelect: (index: number) => void;
  // Called when a row is clicked, to accept that row's completion.
  onAccept: (index: number) => void;
}

export function FormulaHelper({
  completion,
  selected,
  onSelect,
  onAccept,
}: FormulaHelperProps) {
  if (!completion) {
    return null;
  }
  if (completion.kind === "list") {
    return (
      <FunctionList
        matches={completion.matches}
        prefix={completion.prefix}
        selected={selected}
        onSelect={onSelect}
        onAccept={onAccept}
      />
    );
  }
  if (completion.namedFunction) {
    return (
      <NamedFunctionDetail
        namedFunction={completion.namedFunction}
        activeIndex={completion.activeIndex}
      />
    );
  }
  return (
    <FunctionDetail
      name={completion.name}
      activeIndex={completion.activeIndex}
    />
  );
}

function FunctionList({
  matches,
  prefix,
  selected,
  onSelect,
  onAccept,
}: {
  matches: CompletionItem[];
  prefix: string;
  selected: number;
  onSelect: (index: number) => void;
  onAccept: (index: number) => void;
}) {
  const clamped = Math.min(Math.max(selected, 0), matches.length - 1);

  // Fires whenever the selected button changes, scrolling it into view.
  const scrollSelectedIntoView = useCallback((el: HTMLButtonElement | null) => {
    el?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, []);

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: only swallows events so editing isn't interrupted
    // biome-ignore lint/a11y/useKeyWithClickEvents: not user-interactive; stops propagation only
    <div
      className="ic-fh ic-fh-list"
      onPointerDown={swallowPointerDown}
      onClick={swallowClick}
    >
      {matches.map((item, index) => {
        const { name, kind, description } = item;
        const isSelected = index === clamped;
        return (
          <button
            type="button"
            key={`${kind}:${name}`}
            ref={isSelected ? scrollSelectedIntoView : undefined}
            className={`ic-fh-list-item${isSelected ? " ic-fh-selected" : ""}`}
            onMouseEnter={() => onSelect(index)}
            onClick={() => onAccept(index)}
          >
            <span className="ic-fh-list-row">
              <span className="ic-fh-name">
                {/* The name keeps its own casing, whatever the user typed. */}
                <span className="ic-fh-name-match">
                  {name.slice(0, prefix.length)}
                </span>
                {name.slice(prefix.length)}
              </span>
              <KindTag kind={kind} />
            </span>
            {isSelected && description ? (
              <span className="ic-fh-list-desc">{description}</span>
            ) : null}
          </button>
        );
      })}
    </div>
  );
}

// Details card unmounts whenever the helper closes so
// user's choice (collapsed/uncollapsed) is remembered.
let lastCollapsed = false;

// Shared by both detail cards: the collapsed state and its toggle button.
function useCollapsed() {
  const [collapsed, setCollapsed] = useState(lastCollapsed);
  const toggleCollapsed = () => {
    setCollapsed((value) => {
      lastCollapsed = !value;
      return lastCollapsed;
    });
  };
  const button = (
    <IconButton
      icon={
        <ChevronUp
          size={16}
          style={{ transform: collapsed ? "rotate(180deg)" : undefined }}
        />
      }
      aria-label={collapsed ? "Expand" : "Collapse"}
      title={collapsed ? "Expand" : "Collapse"}
      size="xs"
      className="ic-fh-collapse-btn"
      onClick={toggleCollapsed}
    />
  );
  return { collapsed, button };
}

// The card of a named function: its signature, the description the user gave
// it and the description of each parameter that has one.
function NamedFunctionDetail({
  namedFunction,
  activeIndex,
}: {
  namedFunction: NamedFunction;
  activeIndex: number;
}) {
  const { collapsed, button } = useCollapsed();
  const { name, description, parameters } = namedFunction;
  const documented = parameters.some((parameter) => parameter.description);

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: only swallows events so editing isn't interrupted
    // biome-ignore lint/a11y/useKeyWithClickEvents: not user-interactive; stops propagation only
    <div
      className="ic-fh"
      onPointerDown={swallowPointerDown}
      onClick={swallowClick}
    >
      <div className="ic-fh-header">
        <div className="ic-fh-header-main">
          <div className="ic-fh-signature">
            <span className="ic-fh-fn-name">{name}</span>
            {" ( "}
            {parameters.map((parameter, index) => (
              <Fragment key={parameter.name}>
                {index > 0 ? <span className="ic-fh-comma">, </span> : null}
                <span
                  className={`ic-fh-arg${
                    index === activeIndex ? " ic-fh-arg-active" : ""
                  }`}
                >
                  {parameter.optional ? `[${parameter.name}]` : parameter.name}
                </span>
              </Fragment>
            ))}
            {" )"}
          </div>
          {collapsed || !description ? null : (
            <div className="ic-fh-desc">{description}</div>
          )}
        </div>
        <KindTag kind="function" />
        {button}
      </div>

      {collapsed || !documented ? null : (
        <div className="ic-fh-body">
          {parameters.map((parameter, index) =>
            parameter.description ? (
              <div
                key={parameter.name}
                className={`ic-fh-param${
                  index === activeIndex ? " ic-fh-param-active" : ""
                }`}
              >
                <span className="ic-fh-param-name">{parameter.name}</span>
                {parameter.description}
              </div>
            ) : null,
          )}
        </div>
      )}
    </div>
  );
}

function FunctionDetail({
  name,
  activeIndex,
}: {
  name: string;
  activeIndex: number;
}) {
  const { collapsed, button } = useCollapsed();
  const info = lookup(name);
  if (!info) {
    return null;
  }
  const args = info.args;
  const url = docsUrl(name, info.category);
  // Extra commas can push the index past the listed args; keep it on the last
  // one (typically the repeating "…" argument).
  const clampedIndex = Math.min(activeIndex, args.length - 1);

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: only swallows events so editing isn't interrupted
    // biome-ignore lint/a11y/useKeyWithClickEvents: not user-interactive; stops propagation only
    <div
      className="ic-fh"
      onPointerDown={swallowPointerDown}
      onClick={swallowClick}
    >
      <div className="ic-fh-header">
        <div className="ic-fh-header-main">
          <div className="ic-fh-signature">
            <span className="ic-fh-fn-name">{name.toUpperCase()}</span>
            {" ( "}
            {args.map((arg, index) => (
              <Fragment key={arg[0]}>
                {index > 0 ? <span className="ic-fh-comma">, </span> : null}
                <span
                  className={`ic-fh-arg${
                    index === clampedIndex ? " ic-fh-arg-active" : ""
                  }`}
                >
                  {displayArgName(arg[0])}
                </span>
              </Fragment>
            ))}
            {" )"}
          </div>
          {collapsed ? null : (
            <div className="ic-fh-desc">
              {info.description}
              {url ? (
                <a
                  className="ic-fh-link"
                  href={url}
                  target="_blank"
                  rel="noreferrer"
                >
                  Read more
                </a>
              ) : null}
            </div>
          )}
        </div>
        {button}
      </div>

      {collapsed || info.examples.length === 0 ? null : (
        <div className="ic-fh-body">
          <div className="ic-fh-examples-label">Examples</div>
          {info.examples.map((example) => (
            <div key={example} className="ic-fh-example">
              {example}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default FormulaHelper;
