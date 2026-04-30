import {
  IconButton,
  Input,
  IronCalcIconWhite as IronCalcIcon,
  Tooltip,
} from "@ironcalc/workbook";
import { Plus, Search, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

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
    if (isSearching) searchInputRef.current?.focus();
  }, [isSearching]);

  const closeSearch = () => {
    setSearchQuery("");
    setIsSearching(false);
  };

  return (
    <div className="drawer-header">
      <div className={`drawer-header-logo${isSearching ? " is-hidden" : ""}`}>
        <div className="drawer-header-logo-icon">
          <IronCalcIcon />
        </div>
        <h1 className="drawer-header-title">IronCalc</h1>
      </div>

      <div
        className={`drawer-header-actions${isSearching ? " is-hidden" : ""}`}
      >
        <Tooltip title={t("left_drawer.search_workbook")}>
          <IconButton
            icon={<Search />}
            aria-label={t("left_drawer.search_workbook")}
            size="sm"
            variant="ghost"
            className="drawer-header-button"
            onClick={() => setIsSearching(true)}
          />
        </Tooltip>
        <Tooltip title={t("left_drawer.new_workbook")}>
          <IconButton
            icon={<Plus />}
            aria-label={t("left_drawer.new_workbook")}
            size="sm"
            variant="ghost"
            className="drawer-header-button"
            onClick={onNewModel}
          />
        </Tooltip>
      </div>

      <div className={`drawer-header-search${isSearching ? " is-active" : ""}`}>
        <span className="drawer-header-search-icon">
          <Search />
        </span>
        <Input
          ref={searchInputRef}
          size="sm"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") closeSearch();
          }}
          placeholder={t("left_drawer.search_placeholder")}
          className="drawer-header-search-input"
        />
        <IconButton
          icon={<X />}
          aria-label={t("left_drawer.clear_search")}
          size="xs"
          variant="ghost"
          className="drawer-header-search-clear"
          onClick={closeSearch}
        />
      </div>
    </div>
  );
}

export default DrawerHeader;
