import styled from "@emotion/styled";
import {
  type ChangeEvent,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

// This element has a in situ editable text
// We use a virtual element to compute the size of the input

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
    <Container
      style={{
        width: Math.min(width, properties.maxWidth),
      }}
    >
      <TitleInput
        value={name}
        onInput={handleChange}
        onBlur={(event) => {
          properties.onNameChange(event.target.value);
        }}
        onKeyDown={(event) => {
          switch (event.key) {
            case "Enter": {
              // If we hit "Enter" finish editing
              event.currentTarget.blur();
              break;
            }
            case "Escape": {
              // revert changes
              setName(properties.name);
              break;
            }
          }
        }}
        style={{ width: Math.min(width, properties.maxWidth) }}
        spellCheck="false"
      />
      <MirrorDiv ref={mirrorDivRef}>{name}</MirrorDiv>
    </Container>
  );
}

const Container = styled("div")`
  text-align: left;
  padding: 6px 4px;
  font-size: 14px;
  font-weight: 600;
  font-family: Inter;
`;

const MirrorDiv = styled("div")`
  position: absolute;
  top: -9999px;
  left: -9999px;
  white-space: pre-wrap;
  text-wrap: nowrap;
  visibility: hidden;
  font-family: inherit;
  font-size: inherit;
  line-height: inherit;
  padding: inherit;
  border: inherit;
`;

const TitleInput = styled("input")`
  vertical-align: middle;
  text-align: center;
  height: 20px;
  line-height: 20px;
  border-radius: 4px;
  padding: inherit;
  outline: none;
  resize: none;
  text-wrap: nowrap;
  border: none;
  &:hover {
    background-color: #f2f2f2;
  }
  &:focus {
    outline: 1px solid grey;
  }
  font-weight: inherit;
  font-family: inherit;
  font-size: inherit;
  overflow: ellipsis;
  white-space: nowrap;
`;
