import Breadcrumbs from "@mui/material/Breadcrumbs";
import Link from "@mui/material/Link";
import Tooltip from "@mui/material/Tooltip";
import { styled } from "@mui/material/styles";
import { t } from "i18next";
import { X } from "lucide-react";
import type { ReactNode } from "react";
import { theme } from "../../theme";
import { TOOLBAR_HEIGHT } from "../constants";
import NamedRanges from "./NamedRanges/NamedRanges";

const DEFAULT_DRAWER_WIDTH = 300;

interface RightDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  width?: number;
  children?: ReactNode;
  showCloseButton?: boolean;
  backgroundColor?: string;
  title?: string;
}

const RightDrawer = ({
  isOpen,
  onClose,
  width = DEFAULT_DRAWER_WIDTH,
  children,
  showCloseButton = true,
  title = "Named Ranges",
}: RightDrawerProps) => {
  if (!isOpen) return null;

  return (
    <DrawerContainer $drawerWidth={width}>
      {showCloseButton && (
        <Header>
          <HeaderTitle>
            <HeaderBreadcrumbs separator="›">
              <HeaderBreadcrumbLink href="/">{title}</HeaderBreadcrumbLink>
            </HeaderBreadcrumbs>
          </HeaderTitle>
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
            <CloseButton
              onClick={onClose}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  onClose();
                }
              }}
              aria-label="Close drawer"
              tabIndex={0}
            >
              <X />
            </CloseButton>
          </Tooltip>
        </Header>
      )}
      {children}
      <Divider />
      <DrawerContent>
        <NamedRanges title={title} />
      </DrawerContent>
    </DrawerContainer>
  );
};

type DrawerContainerProps = {
  $drawerWidth: number;
};
const DrawerContainer = styled("div")<DrawerContainerProps>(
  ({ $drawerWidth }) => ({
    position: "absolute",
    overflow: "hidden",
    backgroundColor: theme.palette.common.white,
    right: 0,
    top: `${TOOLBAR_HEIGHT + 1}px`,
    bottom: 0,
    borderLeft: `1px solid ${theme.palette.grey[300]}`,
    width: `${$drawerWidth}px`,
    display: "flex",
    flexDirection: "column",
  }),
);

const Header = styled("div")({
  height: "40px",
  display: "flex",
  alignItems: "center",
  justifyContent: "flex-end",
  padding: "0 8px",
});

const HeaderTitle = styled("div")({
  width: "100%",
});

const HeaderBreadcrumbs = styled(Breadcrumbs)({
  fontSize: "12px",
  marginRight: "8px",
  width: "100%",
});

const HeaderBreadcrumbLink = styled(Link)({
  color: theme.palette.grey[900],
  textDecoration: "none",
});

const CloseButton = styled("div")`
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

const Divider = styled("div")({
  height: "1px",
  width: "100%",
  backgroundColor: theme.palette.grey[300],
  margin: "0",
});

const DrawerContent = styled("div")({
  flex: 1,
  height: "100%",
});

export default RightDrawer;
export { DEFAULT_DRAWER_WIDTH };
