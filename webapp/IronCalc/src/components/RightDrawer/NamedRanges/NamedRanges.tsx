import type { DefinedName, WorksheetProperties } from "@ironcalc/wasm";
import { Button, styled, Tooltip } from "@mui/material";
import { t } from "i18next";
import {
  ArrowLeft,
  BookOpen,
  PackageOpen,
  PencilLine,
  Plus,
  Trash2,
  X,
} from "lucide-react";
import { useState } from "react";
import { theme } from "../../../theme";
import EditNamedRange, { type SaveError } from "./EditNamedRange";

const normalizeRangeString = (range: string): string => {
  return range.trim().replace(/['"]/g, "");
};

interface NamedRangesProps {
  title: string;
  onClose: () => void;
  definedNameList: DefinedName[];
  worksheets: WorksheetProperties[];
  updateDefinedName: (
    name: string,
    scope: number | null,
    newName: string,
    newScope: number | null,
    newFormula: string,
  ) => void;
  newDefinedName: (
    name: string,
    scope: number | null,
    formula: string,
  ) => void;
  deleteDefinedName: (name: string, scope: number | null) => void;
  selectedArea: () => string;
}

function NamedRanges({
  title,
  onClose,
  definedNameList = [],
  worksheets = [],
  updateDefinedName,
  newDefinedName,
  deleteDefinedName,
  selectedArea,
}: NamedRangesProps) {
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
  ): SaveError => {
    if (isCreatingNew) {
      if (!newDefinedName) return {};

      const scope_index = worksheets.findIndex((s) => s.name === scope);
      const newScope = scope_index >= 0 ? scope_index : null;
      try {
        newDefinedName(name, newScope, formula);
        setIsCreatingNew(false);
        return {};
      } catch (e) {
        // Since name validation is done client-side, errors from model are formula errors
        return { formulaError: `${e}` };
      }
    } else {
      if (!editingDefinedName || !updateDefinedName) return {};

      const scope_index = worksheets.findIndex((s) => s.name === scope);
      const newScope = scope_index >= 0 ? scope_index : null;
      try {
        updateDefinedName(
          editingDefinedName.name,
          editingDefinedName.scope ?? null,
          name,
          newScope,
          formula,
        );
        setEditingDefinedName(null);
        return {};
      } catch (e) {
        // Since name validation is done client-side, errors from model are formula errors
        return { formulaError: `${e}` };
      }
    }
  };

  // Show edit view if a named range is being edited or created
  if (editingDefinedName || isCreatingNew) {
    let name = "";
    let scopeName = "[Global]";
    let formula = "";

    if (editingDefinedName) {
      name = editingDefinedName.name;
      scopeName =
        editingDefinedName.scope != null
          ? worksheets[editingDefinedName.scope]?.name || "[unknown]"
          : "[Global]";
      formula = editingDefinedName.formula;
    } else if (isCreatingNew && selectedArea) {
      formula = selectedArea();
    }

    const headerTitle = isCreatingNew
      ? t("name_manager_dialog.add_new_range")
      : t("name_manager_dialog.edit_range");

    return (
      <Container>
        <EditHeader>
          <Tooltip title={t("name_manager_dialog.back_to_list")}>
            <IconButtonWrapper
              onClick={handleCancel}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  handleCancel();
                }
              }}
              aria-label={t("name_manager_dialog.back_to_list")}
              tabIndex={0}
            >
              <ArrowLeft />
            </IconButtonWrapper>
          </Tooltip>
          <EditHeaderTitle>{headerTitle}</EditHeaderTitle>
          {onClose && (
            <Tooltip
              title={t("right_drawer.close")}
              slotProps={{
                popper: {
                  modifiers: [
                    {
                      name: "offset",
                      options: {
                        offset: [0, -8],
                      },
                    },
                  ],
                },
              }}
            >
              <IconButtonWrapper
                onClick={onClose}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    onClose();
                  }
                }}
                aria-label={t("right_drawer.close")}
                tabIndex={0}
              >
                <X />
              </IconButtonWrapper>
            </Tooltip>
          )}
        </EditHeader>
        <Content>
          <EditNamedRange
            worksheets={worksheets}
            name={name}
            scope={scopeName}
            formula={formula}
            onSave={handleSave}
            onCancel={handleCancel}
            definedNameList={definedNameList}
            editingDefinedName={editingDefinedName}
          />
        </Content>
      </Container>
    );
  }

  const currentSelectedArea = selectedArea ? selectedArea() : null;

  return (
    <Container>
      {onClose && (
        <Header>
          <HeaderTitle>{title}</HeaderTitle>
          <Tooltip
            title={t("right_drawer.close")}
            slotProps={{
              popper: {
                modifiers: [
                  {
                    name: "offset",
                    options: {
                      offset: [0, -8],
                    },
                  },
                ],
              },
            }}
          >
            <IconButtonWrapper
              onClick={onClose}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  onClose();
                }
              }}
              aria-label={t("right_drawer.close")}
              tabIndex={0}
            >
              <X />
            </IconButtonWrapper>
          </Tooltip>
        </Header>
      )}
      <Content>
        {definedNameList.length === 0 ? (
          <EmptyStateMessage>
            <IconWrapper>
              <PackageOpen />
            </IconWrapper>
            {t("name_manager_dialog.empty_message1")}
            <br />
            {t("name_manager_dialog.empty_message2")}
          </EmptyStateMessage>
        ) : (
          <ListContainer>
            {definedNameList.map((definedName) => {
              const scopeName =
                definedName.scope != null
                  ? worksheets[definedName.scope]?.name || "[Unknown]"
                  : "[Global]";
              const isSelected =
                currentSelectedArea !== null &&
                normalizeRangeString(definedName.formula) ===
                  normalizeRangeString(currentSelectedArea);
              return (
                <ListItem
                  key={`${definedName.name}-${definedName.scope}`}
                  tabIndex={0}
                  $isSelected={isSelected}
                  onClick={() => handleListItemClick(definedName)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      handleListItemClick(definedName);
                    }
                  }}
                >
                  <ListItemText>
                    <NameText>{definedName.name}</NameText>
                    <ScopeText>{scopeName}</ScopeText>
                    <FormulaText>{definedName.formula}</FormulaText>
                  </ListItemText>

                  <IconsWrapper>
                    <Tooltip title={t("name_manager_dialog.edit")}>
                      <IconButton
                        onClick={(e) => {
                          e.stopPropagation();
                          handleListItemClick(definedName);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter" || e.key === " ") {
                            e.preventDefault();
                            e.stopPropagation();
                            handleListItemClick(definedName);
                          }
                        }}
                        aria-label={t("name_manager_dialog.edit")}
                        tabIndex={0}
                      >
                        <PencilLine size={16} />
                      </IconButton>
                    </Tooltip>
                    <Tooltip title={t("name_manager_dialog.delete")}>
                      <IconButton
                        onClick={(e) => {
                          e.stopPropagation();
                          if (deleteDefinedName) {
                            deleteDefinedName(
                              definedName.name,
                              definedName.scope ?? null,
                            );
                          }
                        }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter" || e.key === " ") {
                            e.preventDefault();
                            e.stopPropagation();
                            if (deleteDefinedName) {
                              deleteDefinedName(
                                definedName.name,
                                definedName.scope ?? null,
                              );
                            }
                          }
                        }}
                        aria-label={t("name_manager_dialog.delete")}
                        tabIndex={0}
                      >
                        <Trash2 size={16} />
                      </IconButton>
                    </Tooltip>
                  </IconsWrapper>
                </ListItem>
              );
            })}
          </ListContainer>
        )}
      </Content>
      <Footer>
        <HelpLink
          href="https://docs.ironcalc.com/web-application/name-manager.html"
          target="_blank"
          rel="noopener noreferrer"
        >
          <BookOpen />
          {t("name_manager_dialog.help")}
        </HelpLink>
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
}

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

const ListItem = styled("div")<{ $isSelected?: boolean }>(
  ({ $isSelected }) => ({
    display: "flex",
    alignItems: "flex-start",
    justifyContent: "space-between",
    padding: "8px 12px",
    minHeight: "40px",
    boxSizing: "border-box",
    borderBottom: `1px solid ${theme.palette.grey[200]}`,
    paddingLeft: $isSelected ? "20px" : "12px",
    transition: "all 0.2s ease-in-out",
    borderLeft: $isSelected
      ? `3px solid ${theme.palette.primary.main}`
      : "3px solid transparent",
    "&:hover": {
      backgroundColor: theme.palette.grey[50],
      paddingLeft: "20px",
    },
  }),
);

const ListItemText = styled("div")({
  fontSize: "12px",
  color: theme.palette.common.black,
  fontFamily: theme.typography.fontFamily,
  flex: 1,
  display: "flex",
  flexDirection: "column",
  alignItems: "flex-start",
  gap: "2px",
});

const ScopeText = styled("span")({
  fontSize: "12px",
  color: theme.palette.common.black,
});

const FormulaText = styled("span")({
  fontSize: "12px",
  color: theme.palette.grey[600],
});

const NameText = styled("span")({
  fontSize: "12px",
  color: theme.palette.common.black,
  fontWeight: 600,
});

const IconsWrapper = styled("div")({
  display: "flex",
  alignItems: "center",
  gap: "2px",
});

const IconButton = styled("div")({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: "24px",
  height: "24px",
  borderRadius: "4px",
  backgroundColor: "transparent",
  cursor: "pointer",
  "&:hover": {
    backgroundColor: theme.palette.grey[200],
  },
});

export const Footer = styled("div")`
  padding: 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: ${theme.palette.grey["600"]};
  border-top: 1px solid ${theme.palette.grey["300"]};
  gap: 8px;
`;

const HelpLink = styled("a")`
  display: flex;
  align-items: center;
  gap: 6px;
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
  }
`;

export const NewButton = styled(Button)`
  text-transform: none;
  min-width: fit-content;
  font-size: 12px;
  &.MuiButton-colorSecondary {
    background-color: ${theme.palette.grey[200]};
    color: ${theme.palette.grey[700]};
    &:hover {
      background-color: ${theme.palette.grey[300]};
    }
  }
`;

const Header = styled("div")({
  height: "40px",
  display: "flex",
  alignItems: "center",
  justifyContent: "flex-end",
  padding: "0 8px",
  borderBottom: `1px solid ${theme.palette.grey[300]}`,
});

const HeaderTitle = styled("div")({
  width: "100%",
  fontSize: "12px",
});

const EditHeader = styled("div")({
  height: "40px",
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  padding: "0 8px",
  gap: "8px",
  borderBottom: `1px solid ${theme.palette.grey[300]}`,
});

const EditHeaderTitle = styled("div")({
  flex: 1,
  fontSize: "12px",
  fontWeight: 500,
});

const IconButtonWrapper = styled("div")`
    &:hover {
      background-color: ${theme.palette.grey["50"]};
    }
    display: flex;
    border-radius: 4px;
    height: 24px;
    width: 24px;
    cursor: pointer;
    align-items: center;
    justify-content: center;
    svg {
      width: 16px;
      height: 16px;
      stroke-width: 1.5;
    }
  `;

const EmptyStateMessage = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
  justify-content: center;
  text-align: center;
  width: 100%;
  height: 100%;
  font-size: 12px;
  color: ${theme.palette.grey["600"]};
  font-family: "Inter";
  z-index: 0;
  margin: auto 0px;
  position: relative;
`;

const IconWrapper = styled("div")`
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 4px;
  background-color: ${theme.palette.grey["100"]};
  color: ${theme.palette.grey["600"]};
  svg {
    width: 16px;
    height: 16px;
    stroke-width: 2;
  }
`;

export default NamedRanges;
