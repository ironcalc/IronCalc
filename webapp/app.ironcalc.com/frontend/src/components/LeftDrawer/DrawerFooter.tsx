import { BookOpen } from "lucide-react";
import { useTranslation } from "react-i18next";

function DrawerFooter() {
  const { t } = useTranslation();
  return (
    <div className="drawer-footer">
      <a
        className="drawer-footer-link"
        href="https://docs.ironcalc.com/"
        target="_blank"
        rel="noopener noreferrer"
      >
        <span className="drawer-footer-icon">
          <BookOpen />
        </span>
        <span>{t("left_drawer.documentation")}</span>
      </a>
    </div>
  );
}

export default DrawerFooter;
