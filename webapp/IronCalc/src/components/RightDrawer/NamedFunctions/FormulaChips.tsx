import type { KeyboardEvent } from "react";
import { getFormulaSegments } from "./formulaSegments";

interface FormulaChipsProps {
  formula: string;
  parameters: string[];
  unknownNames?: string[];
}

/** A formula with its arguments shown as chips, for display */
export function FormulaChips({
  formula,
  parameters,
  unknownNames = [],
}: FormulaChipsProps) {
  const segments = getFormulaSegments(formula, parameters, unknownNames);
  return (
    <span className="ic-named-functions-formula">
      {segments.map((segment, index) =>
        segment.kind === "text" ? (
          // biome-ignore lint/suspicious/noArrayIndexKey: segments have no identity
          <span key={index}>{segment.text}</span>
        ) : (
          <span
            // biome-ignore lint/suspicious/noArrayIndexKey: segments have no identity
            key={index}
            className={`ic-named-functions-chip ic-named-functions-chip--${segment.kind}`}
          >
            {segment.text}
          </span>
        ),
      )}
    </span>
  );
}

interface FormulaFieldProps {
  id?: string;
  label: string;
  placeholder: string;
  value: string;
  parameters: string[];
  unknownNames: string[];
  error: boolean;
  helperText?: string;
  onChange: (value: string) => void;
  onKeyDown?: (e: KeyboardEvent) => void;
}

/**
 * An editable formula whose arguments are shown as chips. The chips are
 * drawn on a layer under a textarea with transparent text: both use the same
 * font and wrapping, so the chips sit exactly under the typed names.
 */
export function FormulaField({
  id,
  label,
  placeholder,
  value,
  parameters,
  unknownNames,
  error,
  helperText,
  onChange,
  onKeyDown,
}: FormulaFieldProps) {
  const segments = getFormulaSegments(value, parameters, unknownNames);
  return (
    <div className="ic-named-functions-formula-field">
      <label htmlFor={id}>{label}</label>
      <div
        className={`ic-textarea-control ic-named-functions-formula-control ${error ? "is-error" : ""}`}
      >
        <div className="ic-named-functions-formula-overlay" aria-hidden="true">
          {segments.map((segment, index) =>
            segment.kind === "text" ? (
              // biome-ignore lint/suspicious/noArrayIndexKey: segments have no identity
              <span key={index}>{segment.text}</span>
            ) : (
              <span
                // biome-ignore lint/suspicious/noArrayIndexKey: segments have no identity
                key={index}
                className={`ic-named-functions-chip ic-named-functions-chip--${segment.kind} ic-named-functions-chip--inline`}
              >
                {segment.text}
              </span>
            ),
          )}
          {/* A trailing newline needs a character to take up its line */}
          {value.endsWith("\n") ? <span> </span> : null}
        </div>
        <textarea
          id={id}
          className="ic-named-functions-formula-textarea"
          value={value}
          placeholder={placeholder}
          spellCheck={false}
          rows={3}
          onChange={(e) => onChange(e.target.value)}
          // Like the Textarea component: the workbook's global handlers would
          // otherwise take the focus and the keystrokes
          onKeyDown={(e) => {
            e.stopPropagation();
            onKeyDown?.(e);
          }}
          onClick={(e) => e.stopPropagation()}
          onPaste={(e) => e.stopPropagation()}
          onCopy={(e) => e.stopPropagation()}
          onCut={(e) => e.stopPropagation()}
        />
      </div>
      {helperText ? (
        <p
          className={`ic-named-functions-helper ${error ? "ic-named-functions-helper--error" : ""}`}
        >
          {helperText}
        </p>
      ) : null}
    </div>
  );
}
