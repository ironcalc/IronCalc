import { Button, Tooltip, styled } from "@mui/material";
import { t } from "i18next";
import { BookOpen, Plus } from "lucide-react";
import type React from "react";
import { theme } from "../../../theme";

interface NamedRangesProps {
  title?: string;
}

const NamedRanges: React.FC<NamedRangesProps> = ({
  title = "Named Ranges",
}) => {
  return (
    <Container>
      <Content>
        <h3>{title}</h3>
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

  "& p": {
    margin: "0 0 12px 0",
  },
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
`;

export default NamedRanges;
