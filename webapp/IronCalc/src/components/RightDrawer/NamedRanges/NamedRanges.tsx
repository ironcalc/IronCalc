import type { DefinedName, WorksheetProperties } from "@ironcalc/wasm";
import { Button, Tooltip, styled } from "@mui/material";
import { t } from "i18next";
import { BookOpen, ChevronRight, Plus } from "lucide-react";
import { useState } from "react";
import { theme } from "../../../theme";
import EditNamedRange from "./EditNamedRange";

interface NamedRangesProps {
  title?: string;
  definedNameList?: DefinedName[];
  worksheets?: WorksheetProperties[];
  updateDefinedName?: (
    name: string,
    scope: number | undefined,
    newName: string,
    newScope: number | undefined,
    newFormula: string,
  ) => void;
  newDefinedName?: (
    name: string,
    scope: number | undefined,
    formula: string,
  ) => void;
  selectedArea?: () => string;
}

const NamedRanges: React.FC<NamedRangesProps> = ({
  definedNameList = [],
  worksheets = [],
  updateDefinedName,
  newDefinedName,
  selectedArea,
}) => {
  const [editingDefinedName, setEditingDefinedName] =
    useState<DefinedName | null>(null);
  const [isCreatingNew, setIsCreatingNew] = useState(false);

  const handleListItemClick = (definedName: DefinedName) => {
    setEditingDefinedName(definedName);
    setIsCreatingNew(false);
  };

  const handleNewClick = () => {
    setIsCreatingNew(true);
    setEditingDefinedName(null);
  };

  const handleCancel = () => {
    setEditingDefinedName(null);
    setIsCreatingNew(false);
  };

  const handleSave = (
    name: string,
    scope: string,
    formula: string,
  ): string | undefined => {
    if (isCreatingNew) {
      if (!newDefinedName) return undefined;

      const scope_index = worksheets.findIndex((s) => s.name === scope);
      const newScope = scope_index >= 0 ? scope_index : undefined;
      try {
        newDefinedName(name, newScope, formula);
        setIsCreatingNew(false);
        return undefined;
      } catch (e) {
        return `${e}`;
      }
    } else {
      if (!editingDefinedName || !updateDefinedName) return undefined;

      const scope_index = worksheets.findIndex((s) => s.name === scope);
      const newScope = scope_index >= 0 ? scope_index : undefined;
      try {
        updateDefinedName(
          editingDefinedName.name,
          editingDefinedName.scope,
          name,
          newScope,
          formula,
        );
        setEditingDefinedName(null);
        return undefined;
      } catch (e) {
        return `${e}`;
      }
    }
  };

  // Show edit view if a named range is being edited or created
  if (editingDefinedName || isCreatingNew) {
    let name = "";
    let scopeName = "[global]";
    let formula = "";

    if (editingDefinedName) {
      name = editingDefinedName.name;
      scopeName =
        editingDefinedName.scope !== undefined
          ? worksheets[editingDefinedName.scope]?.name || "[unknown]"
          : "[global]";
      formula = editingDefinedName.formula;
    } else if (isCreatingNew && selectedArea) {
      formula = selectedArea();
    }

    return (
      <Container>
        <Content>
          <EditNamedRange
            worksheets={worksheets}
            name={name}
            scope={scopeName}
            formula={formula}
            onSave={handleSave}
            onCancel={handleCancel}
          />
        </Content>
      </Container>
    );
  }

  // Show list view
  return (
    <Container>
      <Content>
        {definedNameList.length > 0 && (
          <ListContainer>
            {definedNameList.map((definedName) => {
              const scopeName =
                definedName.scope !== undefined
                  ? worksheets[definedName.scope]?.name || "[unknown]"
                  : "[global]";
              return (
                <ListItem
                  key={`${definedName.name}-${definedName.scope}`}
                  onClick={() => handleListItemClick(definedName)}
                  tabIndex={0}
                >
                  <ListItemText>
                    <ScopeText>{scopeName}</ScopeText>
                    <ChevronRightStyled />
                    <NameText>{definedName.name}</NameText>
                  </ListItemText>
                </ListItem>
              );
            })}
          </ListContainer>
        )}
      </Content>
      <Footer>
        <Tooltip
          title={t("name_manager_dialog.help")}
          slotProps={{
            popper: {
              modifiers: [{ name: "offset", options: { offset: [0, -8] } }],
            },
          }}
        >
          <HelpLink
            href="https://docs.ironcalc.com/web-application/name-manager.html"
            target="_blank"
            rel="noopener noreferrer"
          >
            <BookOpen />
          </HelpLink>
        </Tooltip>
        <NewButton
          variant="contained"
          disableElevation
          startIcon={<Plus size={16} />}
          onClick={handleNewClick}
        >
          {t("name_manager_dialog.new")}
        </NewButton>
      </Footer>
    </Container>
  );
};

const Container = styled("div")({
  height: "100%",
  display: "flex",
  flexDirection: "column",
});

const Content = styled("div")({
  flex: 1,
  color: theme.palette.grey[700],
  lineHeight: "1.5",
  overflow: "auto",
});

const ListContainer = styled("div")({
  display: "flex",
  flexDirection: "column",
});

const ListItem = styled("div")({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  padding: "8px 12px",
  minHeight: "40px",
  cursor: "pointer",
  boxSizing: "border-box",
  borderBottom: `1px solid ${theme.palette.grey[200]}`,
  "&:hover": {
    backgroundColor: theme.palette.grey[50],
  },
});

const ListItemText = styled("div")({
  fontSize: "12px",
  color: theme.palette.common.black,
  fontFamily: theme.typography.fontFamily,
  flex: 1,
  display: "flex",
  alignItems: "center",
  gap: "4px",
});

const ScopeText = styled("span")({
  fontSize: "12px",
  color: theme.palette.common.black,
});

const ChevronRightStyled = styled(ChevronRight)({
  width: "12px",
  height: "12px",
  color: theme.palette.grey[500],
});

const NameText = styled("span")({
  fontSize: "12px",
  color: theme.palette.common.black,
});

const Footer = styled("div")`
  padding: 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: ${theme.palette.grey["600"]};
  border-top: 1px solid ${theme.palette.grey["300"]};
`;

const HelpLink = styled("a")`
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 400;
  font-family: "Inter";
  color: ${theme.palette.grey["600"]};
  text-decoration: none;
  &:hover {
    text-decoration: underline;
  }
  svg {
    width: 16px;
    height: 16px;
    color: ${theme.palette.grey["600"]};
  }
`;

const NewButton = styled(Button)`
  text-transform: none;
  min-width: fit-content;
  font-size: 12px;
`;

export default NamedRanges;
