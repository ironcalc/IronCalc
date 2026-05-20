import { X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { IconButton } from "../../Button/IconButton";
import { Tooltip } from "../../Tooltip/Tooltip";
import "./functions.css";

type FunctionsProps = {
  onClose: () => void;
};

const Functions = ({ onClose }: FunctionsProps) => {
  const { t } = useTranslation();

  return (
    <div className="ic-functions-container">
      <div className="ic-functions-header">
        <div className="ic-functions-header-title">{t("functions.title")}</div>
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

export default Functions;
