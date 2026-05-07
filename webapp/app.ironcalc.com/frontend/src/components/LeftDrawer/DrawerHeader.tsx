import {
  IconButton,
  Input,
  IronCalcIconWhite as IronCalcIcon,
  Tooltip,
} from "@ironcalc/workbook";
import { Plus, Search, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import "./left-drawer.css";

interface DrawerHeaderProps {
  onNewModel: () => void;
  searchQuery: string;
  setSearchQuery: (value: string) => void;
}

function DrawerHeader({
  onNewModel,
  searchQuery,
  setSearchQuery,
}: DrawerHeaderProps) {
  const { t } = useTranslation();
  const [isSearching, setIsSearching] = useState(false);
  const searchInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isSearching) {
      searchInputRef.current?.focus();
    }
  }, [isSearching]);

  return (
    <div className="app-ic-drawer-header">
      <div
        className={`app-ic-drawer-header-logo-wrapper${isSearching ? " hidden" : ""}`}
      >
        <div className="app-ic-drawer-logo">
          <IronCalcIcon />
        </div>
        <h1 className="app-ic-drawer-header-title">IronCalc</h1>
      </div>

      <div
        className={`app-ic-drawer-header-actions${isSearching ? " hidden" : ""}`}
      >
        <Tooltip title={t("left_drawer.search_workbook")}>
          <IconButton
            icon={<Search />}
            aria-label={t("left_drawer.search_workbook")}
            onClick={() => setIsSearching(true)}
          />
        </Tooltip>
        <Tooltip title={t("left_drawer.new_workbook")}>
          <IconButton
            icon={<Plus />}
            aria-label={t("left_drawer.new_workbook")}
            onClick={onNewModel}
          />
        </Tooltip>
      </div>

      <div className={`app-ic-drawer-search${isSearching ? " active" : ""}`}>
        <div className="app-ic-drawer-search-icon">
          <Search />
        </div>
        <Input
          ref={searchInputRef}
          size="sm"
          className="app-ic-drawer-search-input"
          value={searchQuery}
          placeholder={t("left_drawer.search_placeholder")}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              setSearchQuery("");
              setIsSearching(false);
            }
          }}
        />
        <IconButton
          icon={<X />}
          aria-label={t("left_drawer.clear_search")}
          onClick={() => {
            setSearchQuery("");
            setIsSearching(false);
          }}
        />
      </div>
    </div>
  );
}

export default DrawerHeader;
