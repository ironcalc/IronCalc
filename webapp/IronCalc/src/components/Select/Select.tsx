import { Check, ChevronDown, ChevronUp } from "lucide-react";
import {
  type CSSProperties,
  type KeyboardEvent,
  type ReactNode,
  useEffect,
  useId,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { createPortal } from "react-dom";

import "./select.css";
import "./dropdown-menu.css";

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

function getMenuPosition(trigger: HTMLElement, menu: HTMLElement) {
  const triggerRect = trigger.getBoundingClientRect();
  const menuWidth = Math.max(menu.offsetWidth, triggerRect.width); // Menu is always at least as wide as the trigger
  const menuHeight = menu.offsetHeight;
  const viewportWidth = window.innerWidth; // Uses window width to determine menu positioning
  const viewportHeight = window.innerHeight;

  const offset = 4; // Distance between trigger and menu
  const margin = 8; // Safety margin when menu os too close to the window's edge

  // Menu will stay below but falls back to above when there's not enough space
  const spaceBelow = viewportHeight - triggerRect.bottom - margin;
  const spaceAbove = triggerRect.top - margin;
  const openBelow =
    spaceBelow >= menuHeight + offset || spaceBelow >= spaceAbove;

  // Menu is left-align by default but right-align when the menu would overflow the right edge
  const leftAligned = triggerRect.left;
  const overflowsRight = leftAligned + menuWidth > viewportWidth - margin;

  let left = overflowsRight ? triggerRect.right - menuWidth : leftAligned;

  let top = openBelow
    ? triggerRect.bottom + offset
    : triggerRect.top - menuHeight - offset;

  if (left + menuWidth > viewportWidth - margin) {
    left = viewportWidth - menuWidth - margin;
  }

  if (left < margin) {
    left = margin;
  }

  if (top + menuHeight > viewportHeight - margin) {
    top = viewportHeight - menuHeight - margin;
  }

  if (top < margin) {
    top = margin;
  }

  return { top, left, minWidth: triggerRect.width };
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
  const labelId = `${selectId}-label`;
  const valueId = `${selectId}-value`;
  const listboxId = `${selectId}-listbox`;

  const rootRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const optionRefs = useRef<Array<HTMLButtonElement | null>>([]);

  const selectedIndex = options.findIndex((option) => option.value === value);
  const selectedOption = options[selectedIndex];

  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(selectedIndex);
  const [menuPosition, setMenuPosition] = useState<CSSProperties>({});

  useEffect(() => {
    if (!open) {
      setActiveIndex(Math.max(0, selectedIndex));
    }
  }, [open, selectedIndex]);

  useEffect(() => {
    function handlePointerDown(event: MouseEvent) {
      const target = event.target as Node | null;
      const root = rootRef.current;
      const menu = menuRef.current;

      if (!target || !root) {
        return;
      }

      const isInside =
        root.contains(target) || (menu ? menu.contains(target) : false);

      if (!isInside) {
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

  useLayoutEffect(() => {
    if (!open) {
      return;
    }

    function updateMenuPosition() {
      const trigger = triggerRef.current;
      const menu = menuRef.current;
      if (!trigger || !menu) {
        return;
      }

      const { top, left, minWidth } = getMenuPosition(trigger, menu);

      setMenuPosition({ top, left, minWidth });
    }

    updateMenuPosition();

    window.addEventListener("resize", updateMenuPosition);
    window.addEventListener("scroll", updateMenuPosition, true);

    return () => {
      window.removeEventListener("resize", updateMenuPosition);
      window.removeEventListener("scroll", updateMenuPosition, true);
    };
  }, [open]);

  const controlClassName = useMemo(() => {
    return [
      "ic-select-control",
      size,
      variant !== "default" && `variant-${variant}`,
      open && "is-open",
      error && "is-error",
      disabled && "disabled",
    ]
      .filter(Boolean)
      .join(" ");
  }, [disabled, error, open, size, variant]);

  function closeMenu() {
    setOpen(false);
    triggerRef.current?.focus();
  }

  function toggleMenu() {
    if (disabled || options.length === 0) {
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
    if (disabled || options.length === 0 || open) {
      return;
    }

    switch (event.key) {
      case "ArrowDown":
      case "ArrowUp":
      case "Enter":
      case " ": {
        event.preventDefault();
        setActiveIndex(Math.max(0, selectedIndex));
        setOpen(true);
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
    }
  }

  return (
    <div
      ref={rootRef}
      className={["ic-select", className].filter(Boolean).join(" ")}
    >
      {label && (
        <label
          id={labelId}
          htmlFor={selectId}
          data-required={required || undefined}
        >
          {label}
        </label>
      )}

      <div className={controlClassName}>
        {name ? <input type="hidden" name={name} value={value} /> : null}

        <button
          ref={triggerRef}
          id={selectId}
          type="button"
          className="ic-select-trigger"
          role="combobox"
          aria-haspopup="listbox"
          aria-expanded={open ? "true" : "false"}
          aria-controls={listboxId}
          aria-labelledby={label ? `${labelId} ${valueId}` : undefined}
          aria-required={required || undefined}
          aria-invalid={error || undefined}
          aria-describedby={helperText ? helperId : undefined}
          disabled={disabled}
          onClick={toggleMenu}
          onKeyDown={handleTriggerKeyDown}
        >
          <span id={valueId} className="ic-select-trigger-value">
            {selectedOption?.triggerLabel ?? selectedOption?.label}
          </span>

          <span className="ic-select-trigger-icon" aria-hidden="true">
            {open ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
          </span>
        </button>

        {open
          ? createPortal(
              <div
                ref={menuRef}
                className="ic-dropdown-menu-wrapper"
                style={menuPosition}
              >
                <div
                  id={listboxId}
                  className="ic-dropdown-menu"
                  role="listbox"
                  aria-labelledby={label ? labelId : valueId}
                >
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
                          "ic-dropdown-menu-option",
                          isSelected && "is-selected",
                          isActive && "is-active",
                        ]
                          .filter(Boolean)
                          .join(" ")}
                        onClick={() => commit(index)}
                        onMouseEnter={() => setActiveIndex(index)}
                        onKeyDown={(event) => handleOptionKeyDown(event, index)}
                      >
                        <span
                          className="ic-dropdown-menu-option-check"
                          aria-hidden="true"
                        >
                          {isSelected ? <Check size={16} /> : null}
                        </span>

                        <span className="ic-dropdown-menu-option-content">
                          {option.label}
                        </span>
                      </button>
                    );
                  })}
                </div>
              </div>,
              document.body,
            )
          : null}
      </div>

      {helperText && <p id={helperId}>{helperText}</p>}
    </div>
  );
}

Select.displayName = "Select";
