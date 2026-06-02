import { X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { IconButton } from "../../Button/IconButton";
import { Tooltip } from "../../Tooltip/Tooltip";
import "./themes.css";

type ThemesProps = {
  onClose: () => void;
};

const Themes = ({ onClose }: ThemesProps) => {
  const { t } = useTranslation();

  return (
    <div className="ic-themes-container">
      <div className="ic-themes-header">
        <div className="ic-themes-header-title">{t("themes.panel_title")}</div>
        <Tooltip title={t("right_drawer.close")}>
          <IconButton
            icon={<X />}
            onClick={onClose}
            aria-label={t("right_drawer.close")}
          />
        </Tooltip>
      </div>
    </div>
  );
};

export default Themes;
