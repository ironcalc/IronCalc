import { styled } from "@mui/material";
import type { ReactNode } from "react";

interface TemplatesListItemProps {
  title: string;
  description: string;
  icon: ReactNode;
  iconColor: string;
  active: boolean;
  onClick: () => void;
}

function TemplatesListItem({
  title,
  description,
  icon,
  iconColor,
  active,
  onClick,
}: TemplatesListItemProps) {
  return (
    <ListItemWrapper active={active} iconColor={iconColor} onClick={onClick}>
      <StyledIcon iconColor={iconColor}>{icon}</StyledIcon>
      <TemplatesListItemTitle>
        <Title>{title}</Title>
        <Subtitle>{description}</Subtitle>
      </TemplatesListItemTitle>
      <RadioButton active={active}>
        <RadioButtonDot />
      </RadioButton>
    </ListItemWrapper>
  );
}

const ListItemWrapper = styled("div")<{ active?: boolean; iconColor?: string }>`
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  gap: 8px;
  font-size: 12px;
  color: #424242;
  border: 1px solid ${(props) => (props.active ? props.iconColor || "#424242" : "rgba(224, 224, 224, 0.60)")};
  background-color: #FFFFFF;
  padding: 16px;
  border-radius: 8px;
  box-shadow: 0 1px 1px rgba(0, 0, 0, 0.1);
  cursor: pointer;
  outline: ${(props) => (props.active ? `4px solid ${props.iconColor || "#424242"}24` : "none")};
  transition: border 0.1s ease-in-out;
  user-select: none;
  &:hover {
    border: 1px solid ${(props) => props.iconColor};
    transition: border 0.1s ease-in-out;
  }
`;

const TemplatesListItemTitle = styled("div")`
  display: flex;
  flex-direction: column;
  color: #424242;
  width: 100%;
  gap: 2px;
`;

const Title = styled("div")`
  font-weight: 600;
  color: #424242;
  line-height: 16px;
`;

const Subtitle = styled("div")`
  color: #757575;
`;

const StyledIcon = styled("div")<{ iconColor?: string }>`
  display: flex;
  align-items: center;
  margin-top: -1px;
  svg {
    width: 18px;
    height: 100%;
    color: ${(props) => props.iconColor || "#424242"};
  }
`;

const RadioButton = styled("div")<{ active?: boolean }>`
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  min-width: 16px;
  height: 16px;
  border-radius: 16px;
  margin-top: -4px;
  margin-right: -4px;
  background-color: ${(props) => (props.active ? "#F2994A" : "#FFFFFF")};
  border: ${(props) => (props.active ? "none" : "1px solid #E0E0E0")};
`;

const RadioButtonDot = styled("div")`
  width: 6px;
  height: 6px;
  border-radius: 6px;
  background-color: #FFF;
`;

export default TemplatesListItem;
