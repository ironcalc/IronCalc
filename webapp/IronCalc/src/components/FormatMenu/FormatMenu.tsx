import { Menu, MenuItem, styled } from "@mui/material";
import { Check } from "lucide-react";
import { type ComponentProps, useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import FormatPicker from "./FormatPicker";
import { KNOWN_FORMATS, NumberFormats } from "./formatUtil";

type FormatMenuProps = {
  children: React.ReactNode;
  numFmt: string;
  onChange: (numberFmt: string) => void;
  onExited?: () => void;
  anchorOrigin?: ComponentProps<typeof Menu>["anchorOrigin"];
};

const FormatMenu = (properties: FormatMenuProps) => {
  const { t } = useTranslation();
  const [isMenuOpen, setMenuOpen] = useState(false);
  const [isPickerOpen, setPickerOpen] = useState(false);
  const anchorElement = useRef<HTMLDivElement>(null);

  const onSelect = useCallback(
    (s: string) => {
      properties.onChange(s);
      setMenuOpen(false);
    },
    [properties.onChange],
  );

  const isCustomFormat = !KNOWN_FORMATS.has(properties.numFmt);

  return (
    <>
      <ChildrenWrapper
        onClick={(): void => setMenuOpen(true)}
        ref={anchorElement}
      >
        {properties.children}
      </ChildrenWrapper>
      <StyledMenu
        open={isMenuOpen}
        onClose={(): void => setMenuOpen(false)}
        anchorEl={anchorElement.current}
        anchorOrigin={{
          vertical: "bottom",
          horizontal: "left",
        }}
        transformOrigin={{
          vertical: "top",
          horizontal: "left",
        }}
      >
        <MenuItemWrapper onClick={(): void => onSelect(NumberFormats.AUTO)}>
          <MenuItemText>
            <CheckIcon $active={properties.numFmt === NumberFormats.AUTO} />
            {t("toolbar.format_menu.auto")}
          </MenuItemText>
        </MenuItemWrapper>
        <MenuDivider />
        <MenuItemWrapper onClick={(): void => onSelect(NumberFormats.NUMBER)}>
          <MenuItemText>
            <CheckIcon $active={properties.numFmt === NumberFormats.NUMBER} />
            {t("toolbar.format_menu.number")}
          </MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.number_example")}
          </MenuItemExample>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={(): void => onSelect(NumberFormats.PERCENTAGE)}
        >
          <MenuItemText>
            <CheckIcon
              $active={properties.numFmt === NumberFormats.PERCENTAGE}
            />
            {t("toolbar.format_menu.percentage")}
          </MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.percentage_example")}
          </MenuItemExample>
        </MenuItemWrapper>

        <MenuDivider />
        <MenuItemWrapper
          onClick={(): void => onSelect(NumberFormats.CURRENCY_EUR)}
        >
          <MenuItemText>
            <CheckIcon
              $active={properties.numFmt === NumberFormats.CURRENCY_EUR}
            />
            {t("toolbar.format_menu.currency_eur")}
          </MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.currency_eur_example")}
          </MenuItemExample>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={(): void => onSelect(NumberFormats.CURRENCY_USD)}
        >
          <MenuItemText>
            <CheckIcon
              $active={properties.numFmt === NumberFormats.CURRENCY_USD}
            />
            {t("toolbar.format_menu.currency_usd")}
          </MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.currency_usd_example")}
          </MenuItemExample>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={(): void => onSelect(NumberFormats.CURRENCY_GBP)}
        >
          <MenuItemText>
            <CheckIcon
              $active={properties.numFmt === NumberFormats.CURRENCY_GBP}
            />
            {t("toolbar.format_menu.currency_gbp")}
          </MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.currency_gbp_example")}
          </MenuItemExample>
        </MenuItemWrapper>

        <MenuDivider />
        <MenuItemWrapper
          onClick={(): void => onSelect(NumberFormats.DATE_SHORT)}
        >
          <MenuItemText>
            <CheckIcon
              $active={properties.numFmt === NumberFormats.DATE_SHORT}
            />
            {t("toolbar.format_menu.date_short")}
          </MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.date_short_example")}
          </MenuItemExample>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={(): void => onSelect(NumberFormats.DATE_LONG)}
        >
          <MenuItemText>
            <CheckIcon
              $active={properties.numFmt === NumberFormats.DATE_LONG}
            />
            {t("toolbar.format_menu.date_long")}
          </MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.date_long_example")}
          </MenuItemExample>
        </MenuItemWrapper>

        <MenuDivider />
        <MenuItemWrapper onClick={(): void => setPickerOpen(true)}>
          <MenuItemText>
            <CheckIcon $active={isCustomFormat} />
            {t("toolbar.format_menu.custom")}
          </MenuItemText>
        </MenuItemWrapper>
      </StyledMenu>
      <FormatPicker
        numFmt={properties.numFmt}
        onChange={onSelect}
        open={isPickerOpen}
        onClose={(): void => setPickerOpen(false)}
        onExited={properties.onExited}
      />
    </>
  );
};

const StyledMenu = styled(Menu)`
  & .MuiPaper-root {
    border-radius: 8px;
    padding: 4px 0px;
    margin-left: -4px; // Starting with a small offset
  }
  & .MuiList-root {
    padding: 0;
  }
`;

const MenuItemWrapper = styled(MenuItem)`
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  width: calc(100% - 8px);
  min-width: 172px;
  margin: 0px 4px;
  border-radius: 4px;
  padding: 8px;
  height: 32px;
`;

const ChildrenWrapper = styled("div")`
  display: flex;
`;

const MenuDivider = styled("div")`
  width: 100%;
  margin: auto;
  margin-top: 4px;
  margin-bottom: 4px;
  border-top: 1px solid #eeeeee;
`;

const CheckIcon = styled(Check)<{ $active: boolean }>`
  width: 16px;
  height: 16px;
  color: ${(props) => (props.$active ? "currentColor" : "transparent")};
  margin-right: 8px;
  flex-shrink: 0;
`;

const MenuItemText = styled("div")`
  color: #000;
  display: flex;
  align-items: center;
`;

const MenuItemExample = styled("div")`
  color: #bdbdbd;
  margin-left: 20px;
`;

export default FormatMenu;
