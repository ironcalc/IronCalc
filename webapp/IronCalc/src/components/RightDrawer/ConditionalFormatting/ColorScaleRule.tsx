import { useTranslation } from "react-i18next";
import { Button } from "../../Button/Button";

interface ColorScaleRuleProps {
  onCancel: () => void;
}

const ColorScaleRule = ({ onCancel }: ColorScaleRuleProps) => {
  const { t } = useTranslation();
  return (
    <>
      <div className="ic-edit-rule-content ic-edit-rule-content--placeholder" />
      <div className="ic-edit-rule-footer">
        <Button variant="secondary" onClick={onCancel}>
          {t("conditional_formatting.cancel")}
        </Button>
      </div>
    </>
  );
};

export default ColorScaleRule;
