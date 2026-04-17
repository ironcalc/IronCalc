import { useTranslation } from "react-i18next";
import { Prompt } from "../Modal/Prompt";

// FIXME: the stopPropagation everywhere is because of my bad implementation
// of keyboard handling in the spreadsheet

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
      confirmLabel={t("num_fmt.save")}
      inputProps={{
        name: "format_code",
        autoFocus: true,
        spellCheck: false,
        onFocus: (event) => event.target.select(),
        onPaste: (event) => event.stopPropagation(),
        onCopy: (event) => event.stopPropagation(),
        onCut: (event) => event.stopPropagation(),
      }}
    />
  );
};

export default FormatPicker;
