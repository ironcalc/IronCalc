import { useTranslation } from "react-i18next";
import { Prompt } from "../Modal";

type FormatPickerProps = {
  className?: string;
  open: boolean;
  onClose: () => void;
  numFmt: string;
  onChange: (numberFmt: string) => void;
};

const FormatPicker = (properties: FormatPickerProps) => {
  const { t } = useTranslation();

  return (
    <Prompt
      open={properties.open}
      onClose={properties.onClose}
      onSubmit={properties.onChange}
      title={t("num_fmt.title")}
      className={properties.className}
      defaultValue={properties.numFmt}
      cancelLabel={t("num_fmt.cancel")}
      confirmLabel={t("num_fmt.save")}
      closeLabel={t("num_fmt.close")}
    />
  );
};

export default FormatPicker;
