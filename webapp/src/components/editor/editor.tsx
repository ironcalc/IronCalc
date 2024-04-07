import {
  CSSProperties,
  useCallback,
  useEffect,
  useState,
  KeyboardEvent,
  useContext,
} from "react";
import { useRef } from "react";
import EditorContext, { Area } from "./editorContext";
import { getStringRange } from "./util";

/**
 * This is the Cell Editor for IronCalc
 * I uses a transparent textarea and a styled mask. What you see is the HTML styled content of the mask
 * and the caret from the textarea. The alternative would be to have a 'contenteditable' div.
 * That turns out to be a much more difficult implementation.
 *
 * The editor grows horizontally with text if it fits in the screen.
 * If it doesn't fit, it wraps and grows vertically. If it doesn't fit vertically it scrolls.
 *
 * Many keyboard and mouse events are handled gracefully by the textarea in full or in part.
 * For example letter key strokes like 'q' or '1' are handled full by the textarea.
 * Some keyboard events like "RightArrow" might need to be handled separately and let them bubble up,
 * or might be handled by the textarea, depending on the "editor mode".
 * Some other like "Enter" we need to intercept and change the normal behaviour.
 */

const commonCSS: CSSProperties = {
  fontWeight: "inherit",
  fontFamily: "inherit",
  fontSize: "inherit",
  position: "absolute",
  left: 0,
  top: 0,
  whiteSpace: "pre",
  width: "100%",
  padding: 0,
};

interface Cell {
  sheet: number;
  row: number;
  column: number;
}

interface EditorOptions {
  minimalWidth: number;
  minimalHeight: number;
  textColor: string;
  originalText: string;
  getStyledText: (
    text: string,
    insertRangeText: string
  ) => {
    html: JSX.Element[];
    isInReferenceMode: boolean;
  };
  onEditEnd: (text: string) => void;
  display: boolean;
  cell: Cell;
  sheetNames: string[];
}

// You can either be editing a formula or content.
// When editing content (behaviour is common to Excel and Google Sheets):
// * If you start editing by typing you are in *accept* mode
// * If you start editing by F2 you are in *cruise* mode
// * If you start editing by double click you are in *cruise* mode
// In Google Sheets "Enter" starts editing and puts you in *cruise* mode. We do not do that
// Once you are in cruise mode it is not possible to switch to accept mode
// The only way to go from accept mode to cruise mode is clicking in the content somewhere

// When editing a formula.
// In Google Sheets you are either in insert mode or cruise mode.
// You can get back to accept mode if you delete the whole formula
// In Excel you can be either in insert or accept but if you click in the formula body
// you switch to cruise mode. Once in cruise mode you can go to insert mode by selecting a range.
// Then you are back in accept/insert modes

const Editor = (options: EditorOptions) => {
  const {
    minimalWidth,
    minimalHeight,
    textColor,
    onEditEnd,
    originalText,
    display,
    cell,
    sheetNames,
  } = options;

  const [width, setWidth] = useState(minimalWidth);
  const [height, setHeight] = useState(minimalHeight);

  const { editorContext, setEditorContext } = useContext(EditorContext);

  const setBaseText = (newText: string) => {
    console.log('Calling setBaseText');
    setEditorContext((c) => {
      return {
        ...c,
        baseText: newText,
      };
    });
  };

  const insertRangeText = editorContext.insertRange
    ? getStringRange(editorContext.insertRange, sheetNames)
    : "";

  const baseText = editorContext.baseText;
  const text = baseText + insertRangeText;
  // console.log('baseText', baseText, 'insertRange:', insertRangeText);

  const formulaRef = useRef<HTMLDivElement>(null);
  const maskRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // useEffect(() => {
  //   setBaseText(originalText);
  // }, [cell]);

  const { html: styledFormula, isInReferenceMode } = options.getStyledText(
    baseText,
    insertRangeText
  );

  if (display && textareaRef.current) {
    textareaRef.current.focus();
  }

  useEffect(() => {
    if (formulaRef.current) {
      const scrollWidth = formulaRef.current.scrollWidth;
      if (scrollWidth > width) {
        setWidth(scrollWidth);
      } else if (scrollWidth <= minimalWidth) {
        setWidth(minimalWidth);
      }
      const scrollHeight = formulaRef.current.scrollHeight;
      if (scrollHeight > height) {
        setHeight(scrollHeight);
      }
    }
  }, [text]);

  useEffect(() => {
    if (isInReferenceMode) {
      setEditorContext((c) => {
        return {
          ...c,
          mode: "insert",
        };
      });
    } else {
      setEditorContext((c) => {
        return {
          ...c,
          mode: "cruise",
          insertRange: null,
        };
      });
    }
  }, [isInReferenceMode]);

  useEffect(() => {
    if (display && textareaRef.current) {
      textareaRef.current.focus();
    }
  }, [display]);

  // console.log("Ok, this is running", text, editorContext.id);

  const onKeyDown = useCallback(
    (event: KeyboardEvent) => {
      const { key, shiftKey, altKey } = event;
      const textarea = textareaRef.current;
      const mode = editorContext.mode;
      if (!textarea) {
        return;
      }
      switch (key) {
        case "Enter": {
          if (altKey) {
            // new line
            const start = textarea.selectionStart;
            const end = textarea.selectionEnd;
            const newText = text.slice(0, start) + "\n" + text.slice(end);
            setBaseText(newText);
            setTimeout(() => {
              textarea.setSelectionRange(start + 1, start + 1);
            }, 1);
            event.stopPropagation();
            event.preventDefault();
            return;
          } else {
            // end edit
            onEditEnd(text);
            textarea.blur();
            // event bubbles up
            return;
          }
          break;
        }
        case "Escape": {
          setBaseText(originalText);
          textarea.blur();
          event.stopPropagation();
          event.preventDefault();
          break;
        }
        case "ArrowLeft": {
          if (mode === "accept") {
            onEditEnd(text);
            textarea.blur();
            // event bubbles up
            return;
          } else if (mode == "insert") {
            if (shiftKey) {
              // increase the inserted range to the left
              if (!editorContext.insertRange) {
                setEditorContext((c) => {
                  return {
                    ...c,
                    insertRange: {
                      absoluteColumnEnd: false,
                      absoluteColumnStart: false,
                      absoluteRowEnd: false,
                      absoluteRowStart: false,
                      sheet: cell.sheet,
                      rowStart: cell.row,
                      rowEnd: cell.row,
                      columnStart: cell.column,
                      columnEnd: cell.column,
                    },
                  };
                });
              } else {
                // const r = insertRage;
                // r.columnStart = Math.max(r.columnStart - 1, 1);
                // setInsertRange(r);
              }
            } else {
              // move inserted cell to the left
              if (!editorContext.insertRange) {
                setEditorContext((c) => {
                  return {
                    ...c,
                    insertRange: {
                      absoluteColumnEnd: false,
                      absoluteColumnStart: false,
                      absoluteRowEnd: false,
                      absoluteRowStart: false,
                      sheet: cell.sheet,
                      rowStart: cell.row,
                      rowEnd: cell.row,
                      columnStart: cell.column,
                      columnEnd: cell.column,
                    },
                  };
                });
              } else {
                setEditorContext((c) => {
                  const range = c.insertRange as Area;
                  const row = range.rowStart;
                  let column = range.columnStart - 1;
                  if (column < 1) {
                    column = 1;
                  }
                  return {
                    ...c,
                    insertRange: {
                      absoluteColumnEnd: false,
                      absoluteColumnStart: false,
                      absoluteRowEnd: false,
                      absoluteRowStart: false,
                      sheet: range.sheet,
                      rowStart: row,
                      rowEnd: row,
                      columnStart: column,
                      columnEnd: column,
                    },
                  };
                });
              }
            }
            event.stopPropagation();
            event.preventDefault();
            return;
          }
          // We don't do anything in "cruise mode" and rely on the textarea default behaviour
          break;
        }
        case "ArrowDown": {
          if (mode === "accept") {
            onEditEnd(text);
            textarea.blur();
          }
          break;
        }
        case "ArrowRight": {
          if (mode === "accept") {
            onEditEnd(text);
            textarea.blur();
          }
          break;
        }
        case "ArrowUp": {
          if (mode === "accept") {
            onEditEnd(text);
            textarea.blur();
          }
          break;
        }
        case "Tab": {
          onEditEnd(text);
          textarea.blur();
          // event bubbles up
        }
      }
      if (editorContext.mode === "insert") {
        setBaseText(text);
        setEditorContext((context) => {
          return {
            ...context,
            mode: "cruise",
            insertRange: null,
          };
        });
      }
    },
    [text, editorContext]
  );

  return (
    <div
      style={{
        position: "relative",
        width,
        height,
        overflow: "hidden",
        background: "#FFF",
        display: display ? "block" : "none",
      }}
      onClick={(_event) => {
        console.log("Click on wrapper");
      }}
      onPointerDown={() => {
        console.log("On pointer down wrapper");
      }}
    >
      <div
        ref={maskRef}
        style={{
          ...commonCSS,
          textAlign: "left",
          pointerEvents: "none",
          height,
        }}
        onClick={(_event) => {
          console.log("Click on mask");
        }}
        onPointerDown={() => {
          console.log("On pointer down mask");
        }}
      >
        <div ref={formulaRef}>{styledFormula}</div>
      </div>
      <textarea
        ref={textareaRef}
        rows={1}
        style={{
          ...commonCSS,
          color: "transparent",
          backgroundColor: "transparent",
          caretColor: textColor,
          outline: "none",
          resize: "none",
          border: "none",
          height,
        }}
        spellCheck="false"
        value={text}
        onChange={(event) => {
          console.log("onChange", event.target.value);
          setBaseText(event.target.value);
        }}
        onScroll={() => {
          if (maskRef.current && textareaRef.current) {
            maskRef.current.style.left = `-${textareaRef.current.scrollLeft}px`;
            maskRef.current.style.top = `-${textareaRef.current.scrollTop}px`;
          }
        }}
        onKeyDown={onKeyDown}
        onClick={(event) => {
          console.log("Setting mode");
          setEditorContext((c) => {
            return {
              ...c,
              mode: "cruise",
            };
          });
          console.log("here");
          // if (display) {
          event.stopPropagation();
          // }
        }}
        onBlur={() => {
          // on blur
        }}
        onPointerDown={(event) => {
          event.stopPropagation();
        }}
      ></textarea>
    </div>
  );
};

export default Editor;
