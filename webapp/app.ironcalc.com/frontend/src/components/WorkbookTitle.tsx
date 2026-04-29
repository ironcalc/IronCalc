import "./workbook-title.css";
import {
  type ChangeEvent,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

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

  return (
    <div
      className="workbook-title"
      style={{ width: Math.min(width, properties.maxWidth) }}
    >
      <input
        className="workbook-title-input"
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
        style={{ width: Math.min(width, properties.maxWidth) }}
        spellCheck={false}
      />
      <div className="workbook-title-mirror" ref={mirrorDivRef}>
        {name}
      </div>
    </div>
  );
}
