import { Menu, MenuItem, styled } from "@mui/material";
import { type ComponentProps, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import FormatPicker from "./formatPicker";
import { NumberFormats } from "./formatUtil";

type FormatMenuProps = {
  children: React.ReactNode;
  numFmt: string;
  onChange: (numberFmt: string) => void;
  onExited?: () => void;
  anchorOrigin?: ComponentProps<typeof Menu>["anchorOrigin"];
};

const FormatMenu = (properties: FormatMenuProps) => {
  const { t } = useTranslation();
  const { onChange } = properties;
  const [isMenuOpen, setMenuOpen] = useState(false);
  const [isPickerOpen, setPickerOpen] = useState(false);
  const anchorElement = useRef<HTMLDivElement>(null);

  return (
    <>
      <ChildrenWrapper
        onClick={(): void => setMenuOpen(true)}
        ref={anchorElement}
      >
        {properties.children}
      </ChildrenWrapper>
      <Menu
        open={isMenuOpen}
        onClose={(): void => setMenuOpen(false)}
        anchorEl={anchorElement.current}
        anchorOrigin={properties.anchorOrigin}
      >
        <MenuItemWrapper onClick={(): void => onChange(NumberFormats.AUTO)}>
          <MenuItemText>{t("toolbar.format_menu.auto")}</MenuItemText>
        </MenuItemWrapper>
        <MenuDivider />
        <MenuItemWrapper onClick={(): void => onChange(NumberFormats.NUMBER)}>
          <MenuItemText>{t("toolbar.format_menu.number")}</MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.number_example")}
          </MenuItemExample>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={(): void => onChange(NumberFormats.PERCENTAGE)}
        >
          <MenuItemText>{t("toolbar.format_menu.percentage")}</MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.percentage_example")}
          </MenuItemExample>
        </MenuItemWrapper>

        <MenuDivider />
        <MenuItemWrapper
          onClick={(): void => onChange(NumberFormats.CURRENCY_EUR)}
        >
          <MenuItemText>{t("toolbar.format_menu.currency_eur")}</MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.currency_eur_example")}
          </MenuItemExample>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={(): void => onChange(NumberFormats.CURRENCY_USD)}
        >
          <MenuItemText>{t("toolbar.format_menu.currency_usd")}</MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.currency_usd_example")}
          </MenuItemExample>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={(): void => onChange(NumberFormats.CURRENCY_GBP)}
        >
          <MenuItemText>{t("toolbar.format_menu.currency_gbp")}</MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.currency_gbp_example")}
          </MenuItemExample>
        </MenuItemWrapper>

        <MenuDivider />
        <MenuItemWrapper
          onClick={(): void => onChange(NumberFormats.DATE_SHORT)}
        >
          <MenuItemText>{t("toolbar.format_menu.date_short")}</MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.date_short_example")}
          </MenuItemExample>
        </MenuItemWrapper>
        <MenuItemWrapper
          onClick={(): void => onChange(NumberFormats.DATE_LONG)}
        >
          <MenuItemText>{t("toolbar.format_menu.date_long")}</MenuItemText>
          <MenuItemExample>
            {t("toolbar.format_menu.date_long_example")}
          </MenuItemExample>
        </MenuItemWrapper>

        <MenuDivider />
        <MenuItemWrapper onClick={(): void => setPickerOpen(true)}>
          <MenuItemText>{t("toolbar.format_menu.custom")}</MenuItemText>
        </MenuItemWrapper>
      </Menu>
      <FormatPicker
        numFmt={properties.numFmt}
        onChange={properties.onChange}
        open={isPickerOpen}
        onClose={(): void => setPickerOpen(false)}
        onExited={properties.onExited}
      />
    </>
  );
};

const MenuItemWrapper = styled(MenuItem)`
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  width: 100%;
`;

const ChildrenWrapper = styled("div")`
  display: flex;
`;

const MenuDivider = styled("div")``;

const MenuItemText = styled("div")`
  color: #000;
`;

const MenuItemExample = styled("div")`
  margin-left: 20px;
`;

export default FormatMenu;
