import { FormHelperText, TextField } from "@mui/material";
import { styled } from "@mui/material/styles";

const Container = styled("div")({
  height: "100%",
  display: "flex",
  flexDirection: "column",
});

const Header = styled("div")(({ theme }) => ({
  height: 40,
  display: "flex",
  alignItems: "center",
  justifyContent: "flex-end",
  padding: "0 8px",
  borderBottom: `1px solid ${theme.palette.grey[300]}`,
}));

const HeaderTitle = styled("div")({
  width: "100%",
  fontSize: "12px",
});

const IconButtonWrapper = styled("div")(({ theme }) => ({
  display: "flex",
  borderRadius: 4,
  height: 24,
  width: 24,
  cursor: "pointer",
  alignItems: "center",
  justifyContent: "center",
  "&:hover": {
    backgroundColor: theme.palette.grey[50],
  },
  "& svg": {
    width: 16,
    height: 16,
    strokeWidth: 1.5,
  },
}));

const Content = styled("div")({
  flex: 1,
  display: "flex",
  flexDirection: "column",
  fontSize: "12px",
  overflow: "auto",
});

const FormSection = styled("div")(({ theme }) => ({
  display: "flex",
  flexDirection: "column",
  gap: 12,
  padding: "16px 12px",
  borderBottom: `1px solid ${theme.palette.grey[300]}`,
  "&:last-child": {
    borderBottom: "none",
  },
}));

const StyledSectionTitle = styled("h1")(({ theme }) => ({
  fontSize: 14,
  fontWeight: 600,
  fontFamily: "Inter",
  margin: 0,
  color: theme.palette.text.primary,
}));

const StyledHelperText = styled(FormHelperText)(({ theme }) => ({
  fontSize: 12,
  fontFamily: "Inter",
  color: theme.palette.grey[500],
  margin: 0,
  marginTop: 6,
  padding: 0,
  lineHeight: 1.4,
}));

const StyledTextField = styled(TextField)`
  & .MuiInputBase-root {
    font-size: 12px;
    height: 32px;
  }
  & .MuiInputBase-input {
    font-size: 12px;
    padding: 8px !important;
  }
`;

const StyledLabel = styled("label")(({ theme }) => ({
  fontSize: 12,
  fontFamily: "Inter",
  fontWeight: 500,
  color: theme.palette.text.primary,
  display: "block",
}));

const StyledColorInput = styled("div")`
  width: 32px;
  height: 32px;
  padding: 0px;
  border: none;
  background: none;
  cursor: pointer;
`;

export {
  Container,
  Header,
  HeaderTitle,
  IconButtonWrapper,
  Content,
  FormSection,
  StyledSectionTitle,
  StyledHelperText,
  StyledTextField,
  StyledColorInput,
  StyledLabel,
};
