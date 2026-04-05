import { Check, ChevronDown, ChevronUp } from "lucide-react";
import {
  type KeyboardEvent,
  type ReactNode,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
} from "react";

import "./select.css";

/**
 * Reusable Select with label, helper text, error state, and size variants.
 * Sizes: sm, md
 * States: default, hover, focused, error, disabled.
 */

export type SelectVariant = "default" | "ghost";

export type SelectSize = "sm" | "md";

export interface SelectOption {
  value: string;
  label: ReactNode;
  triggerLabel?: ReactNode;
}

export interface SelectProperties {
  value: string;
  variant?: SelectVariant;
  options: SelectOption[];
  onChange: (value: string) => void;
  id?: string;
  name?: string;
  size?: SelectSize;
  label?: ReactNode;
  helperText?: ReactNode;
  error?: boolean;
  required?: boolean;
  disabled?: boolean;
  className?: string;
}

export function Select({
  value,
  options,
  onChange,
  id,
  name,
  size = "md",
  variant = "default",
  label,
  helperText,
  error = false,
  required,
  disabled = false,
  className,
}: SelectProperties) {
  const autoId = useId();
  const selectId = id ?? autoId;
  const helperId = `${selectId}-helper`;

  const rootRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const optionRefs = useRef<Array<HTMLButtonElement | null>>([]);

  const selectedIndex = Math.max(
    0,
    options.findIndex((option) => option.value === value),
  );

  // FIXME: This fallback to 0 can cause issues if the value is not in options.
  // See: https://github.com/ironcalc/IronCalc/pull/834#discussion_r3015897822
  const selectedOption = options[selectedIndex] ?? options[0];

  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(selectedIndex);

  useEffect(() => {
    if (!open) {
      setActiveIndex(selectedIndex);
    }
  }, [open, selectedIndex]);

  useEffect(() => {
    function handlePointerDown(event: MouseEvent) {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    document.addEventListener("pointerdown", handlePointerDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
    };
  }, []);

  useEffect(() => {
    if (!open) {
      return;
    }
    optionRefs.current[activeIndex]?.focus();
  }, [open, activeIndex]);

  const listboxId = `${selectId}-listbox`;

  const controlClassName = useMemo(() => {
    return [
      "ic-select-control",
      size,
      variant !== "default" && `variant-${variant}`,
      open && "is-open",
      error && "is-error",
      disabled && "is-disabled",
    ]
      .filter(Boolean)
      .join(" ");
  }, [open, error, disabled, size, variant]);

  function closeMenu() {
    setOpen(false);
    triggerRef.current?.focus();
  }

  function openMenu() {
    if (disabled) {
      return;
    }
    setActiveIndex(selectedIndex);
    setOpen(true);
  }

  function toggleMenu() {
    if (disabled) {
      return;
    }
    setOpen((current) => !current);
  }

  function commit(index: number) {
    const option = options[index];
    if (!option) {
      return;
    }
    onChange(option.value);
    setOpen(false);
    triggerRef.current?.focus();
  }

  function handleTriggerKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    if (disabled || options.length === 0) {
      return;
    }

    switch (event.key) {
      case "ArrowDown":
      case "ArrowUp": {
        event.preventDefault();
        if (!open) {
          openMenu();
        } else {
          const delta = event.key === "ArrowDown" ? 1 : -1;
          const nextIndex =
            (activeIndex + delta + options.length) % options.length;
          setActiveIndex(nextIndex);
        }
        break;
      }
      case "Enter":
      case " ": {
        event.preventDefault();
        toggleMenu();
        break;
      }
      case "Escape": {
        if (open) {
          event.preventDefault();
          closeMenu();
        }
        break;
      }
    }
  }

  function handleOptionKeyDown(
    event: KeyboardEvent<HTMLButtonElement>,
    index: number,
  ) {
    switch (event.key) {
      case "ArrowDown": {
        event.preventDefault();
        setActiveIndex((index + 1) % options.length);
        break;
      }
      case "ArrowUp": {
        event.preventDefault();
        setActiveIndex((index - 1 + options.length) % options.length);
        break;
      }
      case "Home": {
        event.preventDefault();
        setActiveIndex(0);
        break;
      }
      case "End": {
        event.preventDefault();
        setActiveIndex(options.length - 1);
        break;
      }
      case "Enter":
      case " ": {
        event.preventDefault();
        commit(index);
        break;
      }
      case "Escape": {
        event.preventDefault();
        closeMenu();
        break;
      }
      case "Tab": {
        setOpen(false);
        break;
      }
    }
  }

  return (
    <div className={["ic-select", className].filter(Boolean).join(" ")}>
      {label && (
        <label htmlFor={selectId} data-required={required || undefined}>
          {label}
        </label>
      )}

      <div ref={rootRef} className={controlClassName}>
        {name ? <input type="hidden" name={name} value={value} /> : null}

        <button
          ref={triggerRef}
          id={selectId}
          type="button"
          role="combobox"
          aria-autocomplete="none"
          className="ic-select-trigger"
          aria-haspopup="listbox"
          aria-expanded={open ? "true" : "false"}
          aria-controls={listboxId}
          aria-invalid={error || undefined}
          aria-describedby={helperText ? helperId : undefined}
          aria-required={required || undefined}
          disabled={disabled}
          onClick={toggleMenu}
          onKeyDown={handleTriggerKeyDown}
        >
          <span className="ic-select-trigger-value">
            {selectedOption?.triggerLabel ?? selectedOption?.label}
          </span>
          <span className="ic-select-trigger-icon" aria-hidden="true">
            {open ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
          </span>
        </button>

        {open ? (
          <div className="ic-select-menu-wrapper">
            <div id={listboxId} className="ic-select-menu" role="listbox">
              {options.map((option, index) => {
                const isSelected = option.value === value;
                const isActive = index === activeIndex;

                return (
                  <button
                    key={option.value}
                    ref={(element) => {
                      optionRefs.current[index] = element;
                    }}
                    type="button"
                    role="option"
                    aria-selected={isSelected ? "true" : "false"}
                    className={[
                      "ic-select-option",
                      isSelected && "is-selected",
                      isActive && "is-active",
                    ]
                      .filter(Boolean)
                      .join(" ")}
                    onClick={() => commit(index)}
                    onMouseEnter={() => setActiveIndex(index)}
                    onKeyDown={(event) => handleOptionKeyDown(event, index)}
                  >
                    <span className="ic-select-option-check" aria-hidden="true">
                      {isSelected ? <Check size={16} /> : null}
                    </span>
                    <span className="ic-select-option-content">
                      {option.label}
                    </span>
                  </button>
                );
              })}
            </div>
          </div>
        ) : null}
      </div>

      {helperText && <p id={helperId}>{helperText}</p>}
    </div>
  );
}

Select.displayName = "Select";
