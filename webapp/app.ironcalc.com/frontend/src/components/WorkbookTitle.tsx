import {
  type ChangeEvent,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

import "./workbook-title.css";

// This element has an in situ editable text.
// We use a virtual element to compute the size of the input.

export function WorkbookTitle(properties: {
  name: string;
  onNameChange: (name: string) => void;
  maxWidth: number;
}) {
  const [width, setWidth] = useState(0);
  const [name, setName] = useState(properties.name);
  const mirrorDivRef = useRef<HTMLDivElement>(null);

  const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
    setName(event.target.value);
    if (mirrorDivRef.current) {
      setWidth(mirrorDivRef.current.scrollWidth);
    }
  };

  useEffect(() => {
    setName(properties.name);
  }, [properties.name]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: We need to change the width with every value change
  useLayoutEffect(() => {
    if (mirrorDivRef.current) {
      setWidth(mirrorDivRef.current.scrollWidth);
    }
  }, [name]);

  const constrainedWidth = Math.min(width, properties.maxWidth);

  return (
    <div className="app-ic-workbook-title" style={{ width: constrainedWidth }}>
      <input
        className="app-ic-workbook-title-input"
        value={name}
        onChange={handleChange}
        onBlur={(event) => {
          properties.onNameChange(event.target.value);
        }}
        onKeyDown={(event) => {
          switch (event.key) {
            case "Enter": {
              event.currentTarget.blur();
              break;
            }
            case "Escape": {
              setName(properties.name);
              break;
            }
          }
        }}
        style={{ width: constrainedWidth }}
        spellCheck={false}
      />
      <div ref={mirrorDivRef} className="app-ic-workbook-title-mirror">
        {name}
      </div>
    </div>
  );
}
