import { BookOpen } from "lucide-react";
import { useTranslation } from "react-i18next";
import "./left-drawer.css";

function DrawerFooter() {
  const { t } = useTranslation();
  return (
    <div className="app-ic-drawer-footer">
      <a
        className="app-ic-drawer-footer-link"
        href="https://docs.ironcalc.com/"
        target="_blank"
        rel="noopener noreferrer"
      >
        <BookOpen />
        {t("left_drawer.documentation")}
      </a>
    </div>
  );
}

export default DrawerFooter;
