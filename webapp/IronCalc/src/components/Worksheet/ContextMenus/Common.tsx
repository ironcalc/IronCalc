import { Menu, MenuItem, type MenuItemProps, styled } from "@mui/material";
import { alpha } from "@mui/material/styles";
import { ChevronRight } from "lucide-react";

export const StyledMenu = styled(Menu)({
  "& .MuiPaper-root": {
    borderRadius: 8,
    paddingTop: 4,
    paddingBottom: 4,
  },
  "& .MuiList-padding": {
    padding: 0,
  },
});

export const StyledMenuItem = styled((props: MenuItemProps) => (
  <MenuItem disableRipple {...props} />
))(({ theme }) => ({
  display: "flex",
  justifyContent: "flex-start",
  fontSize: 12,
  width: "calc(100% - 8px)",
  minWidth: 172,
  margin: "0px 4px",
  borderRadius: 4,
  padding: 8,
  height: 32,
  gap: 8,
  "& svg": {
    width: 16,
    height: 16,
    color: theme.palette.grey[600],
  },
}));

export const DeleteButton = styled(StyledMenuItem)(({ theme }) => ({
  color: theme.palette.error.main,
  div: {
    color: theme.palette.error.main,
  },
  "& svg": {
    color: theme.palette.error.main,
  },
  "&:hover": {
    backgroundColor: alpha(theme.palette.error.main, 0.1),
  },
  "&:active": {
    backgroundColor: alpha(theme.palette.error.main, 0.1),
  },
}));

export const MenuDivider = styled("div")(({ theme }) => ({
  width: "100%",
  margin: "auto",
  marginTop: 4,
  marginBottom: 4,
  borderTop: `1px solid ${theme.palette.grey[200]}`,
}));

export const ItemNameStyled = styled("div")(({ theme }) => ({
  fontSize: 12,
  color: theme.palette.grey[900],
  flexGrow: 2,
}));

export const ChevronRightStyled = styled(ChevronRight)(({ theme }) => ({
  width: 16,
  height: 16,
  color: theme.palette.grey[900],
}));
