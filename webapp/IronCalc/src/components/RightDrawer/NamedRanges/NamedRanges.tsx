import React from 'react';
import Button from "@mui/material/Button";
import { styled } from "@mui/material/styles";
import { theme } from "../../../theme";

interface NamedRangesProps {
  title?: string;
  // Add props as needed for your use case
}

const NamedRanges: React.FC<NamedRangesProps> = ({ title = "Named Ranges" }) => {
  return (
    <Container>
      <Content>
        <h3>{title}</h3>
        <p>This is the Named Ranges component.</p>
        <p>You can customize this content based on your specific needs.</p>
      </Content>
      <Divider />
      <Footer>
        <FooterButton variant="contained" color="primary">
          Save changes
        </FooterButton>
      </Footer>
    </Container>
  );
};

const Container = styled("div")({
  padding: "16px",
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

const Divider = styled("div")({
  height: "1px",
  width: "100%",
  backgroundColor: theme.palette.grey[300],
  margin: "0",
});

const Footer = styled("div")({
  height: "40px",
  display: "flex",
  alignItems: "center",
  justifyContent: "flex-end",
  padding: "0 8px",
  flexShrink: 0,
});

const FooterButton = styled(Button)({
  width: "100%",
});

export default NamedRanges;
