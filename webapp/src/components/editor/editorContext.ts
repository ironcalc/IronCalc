import { Dispatch, SetStateAction, createContext } from "react";

export interface Area {
  sheet: number | null;
  rowStart: number;
  rowEnd: number;
  columnStart: number;
  columnEnd: number;
  absoluteRowStart: boolean;
  absoluteRowEnd: boolean;
  absoluteColumnStart: boolean;
  absoluteColumnEnd: boolean;
}

// Arrow keys behave in different ways depending on the "edit mode":
// * In _cruise_ mode arrowy keys navigate within the editor
// * In _accept_ mode pressing an arrow key will end editing
// * In _insert_ mode arrow keys will change the selected range
export type EditorMode = "cruise" | "accept" | "insert";

export interface EditorState {
  mode: EditorMode;
  insertRange: null | Area;
  baseText: string;
  id: number;
}

interface EditorContextType {
  editorContext: EditorState;
  setEditorContext: Dispatch<
    SetStateAction<{ mode: EditorMode; insertRange: null | Area }>
  >;
}

const EditorContext = createContext<EditorContextType>({
  editorContext: {
    mode: "accept",
    insertRange: null,
    baseText: '',
    id: Math.floor(Math.random()*1000),
  },
  setEditorContext: () => {},
});

export default EditorContext;
