import styled from "@emotion/styled";
import { Share2 } from "lucide-react";

export function ShareButton(properties: { onClick: () => void }) {
  const { onClick } = properties;
  return (
    <Wrapper onClick={onClick} onKeyDown={() => {}}>
      <ShareIcon />
      <ShareText>Share</ShareText>
    </Wrapper>
  );
}

const Wrapper = styled("div")`
  cursor: pointer;
  color: #ffffff;
  background: #f2994a;
  padding: 0px 10px;
  height: 36px;
  line-height: 36px;
  border-radius: 4px;
  margin-right: 10px;
  display: flex;
  align-items: center;
  font-family: "Inter";
  font-size: 12px;
  &:hover {
    background: #d68742;
  }
`;

const ShareIcon = styled(Share2)`
  width: 16px;
  height: 16px;
  margin-right: 10px;
  
  @media (max-width: 440px) {
    margin-right: 0px;
  }
`;

const ShareText = styled.span`
  @media (max-width: 440px) {
    display: none;
  }
`;
