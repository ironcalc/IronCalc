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
import { Check, ChevronDown, ChevronUp } from "lucide-react";

export interface SelectOption {
  value: string;
  label: ReactNode;
  triggerLabel?: ReactNode;
}

interface SelectProps {
  value: string;
  options: SelectOption[];
  onChange: (value: string) => void;
  id?: string;
  name?: string;
  disabled?: boolean;
  ariaLabelledBy?: string;
  ariaDescribedBy?: string;
  className?: string;
  compact?: boolean;
}

export function Select({
  value,
  options,
  onChange,
  id,
  name,
  disabled = false,
  compact = false,
  ariaLabelledBy,
  ariaDescribedBy,
  className = "",
}: SelectProps) {
  const autoId = useId();
  const selectId = id ?? autoId;

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

  const rootClassName = useMemo(() => {
    return [
      "ic-select",
      open && "is-open",
      disabled && "is-disabled",
      compact && "compact",
      className,
    ]
      .filter(Boolean)
      .join(" ");
  }, [open, disabled, compact, className]);

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
    <div ref={rootRef} className={rootClassName}>
      {name ? <input type="hidden" name={name} value={value} /> : null}

      <button
        ref={triggerRef}
        id={selectId}
        type="button"
        className="ic-select-trigger"
        aria-haspopup="listbox"
        aria-expanded={open ? "true" : "false"}
        aria-controls={listboxId}
        aria-labelledby={ariaLabelledBy}
        aria-describedby={ariaDescribedBy}
        disabled={disabled}
        onClick={toggleMenu}
        onKeyDown={handleTriggerKeyDown}
      >
        <span className="ic-select-trigger-value">
          {selectedOption?.triggerLabel ?? selectedOption?.label}
        </span>
        <span className="ic-select-trigger-icon" aria-hidden="true">
          {open ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
        </span>
      </button>

      {open ? (
        <div className="ic-select-menu-wrapper">
          <div
            id={listboxId}
            className="ic-select-menu"
            role="listbox"
            aria-labelledby={ariaLabelledBy}
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
                    {isSelected ? <Check size={14} /> : null}
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
  );
}
