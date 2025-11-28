import type { FmtSettings } from "@ironcalc/wasm";
import { Menu, MenuItem, styled } from "@mui/material";
import { Check } from "lucide-react";
import { type ComponentProps, useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import FormatPicker from "./FormatPicker";
import { NumberFormats } from "./formatUtil";

type FormatMenuProps = {
  children: React.ReactNode;
  numFmt: string;
  onChange: (numberFmt: string) => void;
  onExited: () => void;
  anchorOrigin: ComponentProps<typeof Menu>["anchorOrigin"];
  formatOptions: FmtSettings;
};

const FormatMenu = (properties: FormatMenuProps) => {
  const { t } = useTranslation();
  const [isMenuOpen, setMenuOpen] = useState(false);
  const [isPickerOpen, setPickerOpen] = useState(false);
  const anchorElement = useRef<HTMLDivElement>(null);

  const formatOptions = properties.formatOptions;

  const onSelect = useCallback(
    (s: string) => {
      properties.onChange(s);
      setMenuOpen(false);
    },
    [properties.onChange],
  );

  const isAutoFormat = properties.numFmt === NumberFormats.AUTO;
  const isNumberFormat = properties.numFmt === formatOptions.number_fmt;
  const isPercentageFormat = properties.numFmt === NumberFormats.PERCENTAGE;
  const isCurrencyEurosFormat =
    properties.numFmt === NumberFormats.CURRENCY_EUR;
  const isCurrencyUsdFormat = properties.numFmt === NumberFormats.CURRENCY_USD;
  const isCurrencyGbpFormat = properties.numFmt === NumberFormats.CURRENCY_GBP;
  const isShortDateFormat = properties.numFmt === formatOptions.short_date;
  const isLongDateFormat = properties.numFmt === formatOptions.long_date;

  const isCustomFormat = !(
    isAutoFormat ||
    isNumberFormat ||
    isPercentageFormat ||
    isCurrencyEurosFormat ||
    isCurrencyUsdFormat ||
    isCurrencyGbpFormat ||
    isShortDateFormat ||
    isLongDateFormat
  );

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
            <CheckIcon $active={isAutoFormat} />
            {t("toolbar.format_menu.auto")}
          </MenuItemText>
        </MenuItemWrapper>
        <MenuDivider />
        <MenuItemWrapper
          onClick={(): void => onSelect(formatOptions.number_fmt)}
        >
          <MenuItemText>
            <CheckIcon $active={isNumberFormat} />
            {t("toolbar.format_menu.number")}
          </MenuItemText>
          <MenuItemExample>{formatOptions.number_example}</MenuItemExample>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={(): void => onSelect(NumberFormats.PERCENTAGE)}
        >
          <MenuItemText>
            <CheckIcon $active={isPercentageFormat} />
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
            <CheckIcon $active={isCurrencyEurosFormat} />
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
            <CheckIcon $active={isCurrencyUsdFormat} />
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
            <CheckIcon $active={isCurrencyGbpFormat} />
            {t("toolbar.format_menu.currency_gbp")}
          </MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.currency_gbp_example")}
          </MenuItemExample>
        </MenuItemWrapper>

        <MenuDivider />
        <MenuItemWrapper
          onClick={(): void => onSelect(formatOptions.short_date)}
        >
          <MenuItemText>
            <CheckIcon $active={isShortDateFormat} />
            {t("toolbar.format_menu.date_short")}
          </MenuItemText>
          <MenuItemExample>{formatOptions.short_date_example}</MenuItemExample>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={(): void => onSelect(formatOptions.long_date)}
        >
          <MenuItemText>
            <CheckIcon $active={isLongDateFormat} />
            {t("toolbar.format_menu.date_long")}
          </MenuItemText>
          <MenuItemExample>{formatOptions.long_date_example}</MenuItemExample>
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

const CheckIcon = styled(Check, {
  shouldForwardProp: (prop) => prop !== "$active",
})<{ $active: boolean }>`
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
