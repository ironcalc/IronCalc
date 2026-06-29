import type { FmtSettings, Model, NamedStyle } from "@ironcalc/wasm";
import { PackageOpen, PencilLine, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { IconButton } from "../../Button/IconButton";
import { Tooltip } from "../../Tooltip/Tooltip";
import { getPreviewText, getTileStyle } from "./named-styles-utils";

interface ManageCustomStylesProps {
  model: Model;
  customStyles: NamedStyle[];
  formatOptions: FmtSettings;
  onEdit: (style: NamedStyle) => void;
  onDelete: (style: NamedStyle) => void;
}

const ManageCustomStyles = ({
  model,
  customStyles,
  formatOptions,
  onEdit,
  onDelete,
}: ManageCustomStylesProps) => {
  const { t } = useTranslation();

  if (customStyles.length === 0) {
    return (
      <div className="ic-named-styles-empty-state-message">
        <div className="ic-named-styles-icon-wrapper">
          <PackageOpen />
        </div>
        {t("named_styles.empty_message1")}
        <br />
        {t("named_styles.empty_message2")}
      </div>
    );
  }

  return (
    <div className="ic-named-styles-manage-list">
      {customStyles.map((s) => (
        <div key={s.name} className="ic-named-styles-manage-item">
          <div
            className="ic-named-styles-manage-item-preview"
            style={getTileStyle(model, s.style)}
          >
            {getPreviewText(s.style.num_fmt, formatOptions, t)}
          </div>
          <div className="ic-named-styles-manage-item-name">{s.name}</div>
          <div className="ic-named-styles-manage-item-icons">
            <Tooltip title={t("named_styles.update_style")}>
              <IconButton
                icon={<PencilLine size={16} />}
                onClick={() => onEdit(s)}
                aria-label={t("named_styles.update_style")}
              />
            </Tooltip>
            <Tooltip title={t("named_styles.delete_style")}>
              <IconButton
                icon={<Trash2 size={16} />}
                onClick={() => onDelete(s)}
                aria-label={t("named_styles.delete_style")}
              />
            </Tooltip>
          </div>
        </div>
      ))}
    </div>
  );
};

export default ManageCustomStyles;
