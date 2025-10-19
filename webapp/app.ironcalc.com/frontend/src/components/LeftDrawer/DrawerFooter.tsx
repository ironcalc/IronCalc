import styled from "@emotion/styled";
import { BookOpen } from "lucide-react";

function DrawerFooter() {
  return (
    <StyledDrawerFooter>
      <FooterLink
        href="https://docs.ironcalc.com/"
        target="_blank"
        rel="noopener noreferrer"
      >
        <OpenBookIcon>
          <BookOpen />
        </OpenBookIcon>
        <FooterLinkText>Documentation</FooterLinkText>
      </FooterLink>
    </StyledDrawerFooter>
  );
}

const StyledDrawerFooter = styled("div")`
  display: flex;
  align-items: center;
  padding: 12px;
  justify-content: space-between;
  max-height: 60px;
  height: 60px;
  border-top: 1px solid #e0e0e0;
  box-sizing: border-box;
`;

const FooterLink = styled("a")`
  display: flex;
  gap: 8px;
  justify-content: flex-start;
  font-size: 14px;
  width: 100%;
  min-width: 172px;
  border-radius: 8px;
  padding: 8px 4px 8px 8px;
  transition: gap 0.5s;
  background-color: transparent;
  color: #000;
  text-decoration: none;
  align-items: center;

  &:hover {
    background-color: #e0e0e0 !important;
  }
`;

const OpenBookIcon = styled("div")`
  height: 16px;
  width: 16px;
  svg {
    height: 16px;
    width: 16px;
    stroke: #9e9e9e;
  }
`;

const FooterLinkText = styled("div")`
  color: #000;
  font-size: 12px;
  width: 100%;
  max-width: 240px;
  overflow: hidden;
  text-overflow: ellipsis;
`;

export default DrawerFooter;
