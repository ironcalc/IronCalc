import type { NamedStyle } from "@ironcalc/wasm";
import { PencilLine, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { IconButton } from "../../Button/IconButton";
import { Tooltip } from "../../Tooltip/Tooltip";
import { getTileStyle } from "./named-styles-utils";

interface ManageCustomStylesProps {
  customStyles: NamedStyle[];
  onEdit: (style: NamedStyle) => void;
  onDelete: (style: NamedStyle) => void;
}

const ManageCustomStyles = ({
  customStyles,
  onEdit,
  onDelete,
}: ManageCustomStylesProps) => {
  const { t } = useTranslation();
  return (
    <div className="ic-named-styles-manage-list">
      {customStyles.map((s) => (
        <div key={s.name} className="ic-named-styles-manage-item">
          <div
            className="ic-named-styles-manage-item-preview"
            style={getTileStyle(s.style)}
          >
            Aa
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
