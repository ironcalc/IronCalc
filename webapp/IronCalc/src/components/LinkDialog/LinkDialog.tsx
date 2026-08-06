import type { Link } from "@ironcalc/wasm";
import { quoteName } from "@ironcalc/wasm";
import { Check, Trash2, X } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../Button/Button";
import { IconButton } from "../Button/IconButton";
import { Input } from "../Input/Input";
import { ModalDialog } from "../Modal/ModalDialog";
import { useModalFocus } from "../Modal/useModalFocus";
import { useModalKeyDown } from "../Modal/useModalKeyDown";
import { Select } from "../Select/Select";
import {
  buildMailto,
  CELL_REFERENCE_REGEX,
  DEFINED_NAME_REGEX,
  isValidEmail,
  normalizeUrl,
  parseLocation,
  parseMailto,
} from "./util";
import "../Modal/modal-dialog.css";
import "./link-dialog.css";

type LinkDialogTab = "external" | "document" | "email";

interface LinkDialogProperties {
  open: boolean;
  onClose: () => void;
  /// The names of the sheets in the workbook, for the document link tab
  sheetNames: string[];
  /// The name of the sheet the cell is in (initial value of the sheet select)
  selectedSheetName: string;
  /// The link currently in the cell: null means the dialog adds a new link
  initialLink: Link | null;
  /// The content of the cell, used to pre-fill the label input
  initialLabel: string;
  onSave: (link: Link, label: string) => void;
  onDelete: () => void;
}

export function LinkDialog({
  open,
  onClose,
  sheetNames,
  selectedSheetName,
  initialLink,
  initialLabel,
  onSave,
  onDelete,
}: LinkDialogProperties) {
  const modalId = useId();
  const titleId = `${modalId}-title`;
  const { modalRef, restoreFocus } = useModalFocus(open);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const saveButtonRef = useRef<HTMLButtonElement>(null);
  const { t } = useTranslation();

  const [tab, setTab] = useState<LinkDialogTab>("external");
  const [url, setUrl] = useState("");
  const [email, setEmail] = useState("");
  const [subject, setSubject] = useState("");
  // mailto query parameters other than the subject (e.g. body): not shown in
  // the dialog but preserved when the link is saved
  const [emailExtraParams, setEmailExtraParams] = useState("");
  const [sheetName, setSheetName] = useState(selectedSheetName);
  const [cellRef, setCellRef] = useState("");
  const [tooltip, setTooltip] = useState("");
  const [label, setLabel] = useState("");

  useEffect(() => {
    if (!open) {
      return;
    }
    setLabel(initialLabel);
    setTooltip(initialLink?.tooltip || "");
    setUrl("");
    setEmail("");
    setSubject("");
    setEmailExtraParams("");
    setSheetName(selectedSheetName);
    setCellRef("");
    if (!initialLink) {
      setTab("external");
    } else if (initialLink.type === "Internal") {
      setTab("document");
      const location = parseLocation(initialLink.location);
      if (location.sheetName) {
        setSheetName(location.sheetName);
      }
      setCellRef(location.cellRef.replace(/\$/g, ""));
    } else if (initialLink.target.startsWith("mailto:")) {
      setTab("email");
      const mailto = parseMailto(initialLink.target);
      setEmail(mailto.address);
      setSubject(mailto.subject);
      setEmailExtraParams(mailto.otherParams);
    } else {
      setTab("external");
      setUrl(initialLink.target);
    }
  }, [open, initialLink, initialLabel, selectedSheetName]);

  const closeModal = (): void => {
    onClose();
    restoreFocus();
  };

  // Validation. Error messages only show once there is something typed,
  // but the save button stays disabled while the link is not valid.
  const urlError =
    tab === "external" && url.trim() !== "" && !normalizeUrl(url);
  const emailError =
    tab === "email" && email.trim() !== "" && !isValidEmail(email);
  const trimmedCellRef = cellRef.replace(/\$/g, "").trim();
  const cellRefValid =
    CELL_REFERENCE_REGEX.test(trimmedCellRef) ||
    DEFINED_NAME_REGEX.test(trimmedCellRef);
  const cellRefError =
    tab === "document" && cellRef.trim() !== "" && !cellRefValid;

  let canSave = false;
  switch (tab) {
    case "external":
      canSave = normalizeUrl(url) !== null;
      break;
    case "document":
      canSave = cellRef.trim() !== "" && cellRefValid;
      break;
    case "email":
      canSave = email.trim() !== "" && isValidEmail(email);
      break;
  }

  const buildLink = (): Link | null => {
    const linkTooltip = tooltip.trim() || undefined;
    switch (tab) {
      case "external": {
        const target = normalizeUrl(url);
        if (!target) {
          return null;
        }
        return { type: "External", target, tooltip: linkTooltip };
      }
      case "email": {
        if (!isValidEmail(email)) {
          return null;
        }
        // the address input may itself carry mailto parameters
        // ("a@b.com?subject=Hi"): the subject field wins over an inline one
        const inline = parseMailto(email.trim());
        const subjectValue = subject.trim() || inline.subject;
        const otherParams = [inline.otherParams, emailExtraParams]
          .filter(Boolean)
          .join("&");
        return {
          type: "External",
          target: buildMailto(inline.address, subjectValue, otherParams),
          tooltip: linkTooltip,
        };
      }
      case "document": {
        let location: string;
        if (CELL_REFERENCE_REGEX.test(trimmedCellRef)) {
          location = `${quoteName(sheetName)}!${trimmedCellRef.toUpperCase()}`;
        } else if (DEFINED_NAME_REGEX.test(trimmedCellRef)) {
          // a defined name
          location = trimmedCellRef;
        } else {
          return null;
        }
        return { type: "Internal", location, tooltip: linkTooltip };
      }
    }
  };

  const handleSave = (): void => {
    const link = buildLink();
    if (!link) {
      return;
    }
    onSave(link, label);
    closeModal();
  };

  const handleDelete = (): void => {
    onDelete();
    closeModal();
  };

  const { onKeyDown } = useModalKeyDown({
    focusableElements: [closeButtonRef, saveButtonRef],
    onClose: closeModal,
    onConfirm: handleSave,
  });

  if (!open) {
    return null;
  }

  const tabs: { id: LinkDialogTab; label: string }[] = [
    { id: "external", label: t("link_dialog.external_tab") },
    { id: "document", label: t("link_dialog.document_tab") },
    { id: "email", label: t("link_dialog.email_tab") },
  ];

  const stopEditorKeys = (event: React.KeyboardEvent): void => {
    event.stopPropagation();
  };

  return (
    <ModalDialog
      modalRef={modalRef}
      titleId={titleId}
      onClose={closeModal}
      onKeyDown={onKeyDown}
      className="ic-link-dialog"
    >
      <div className="ic-modal-dialog-header">
        <h2 id={titleId}>
          {initialLink
            ? t("link_dialog.title_edit")
            : t("link_dialog.title_add")}
        </h2>
        <IconButton
          ref={closeButtonRef}
          icon={<X />}
          aria-label={t("link_dialog.close")}
          onClick={closeModal}
        />
      </div>
      <div className="ic-modal-dialog-body">
        <div className="ic-link-dialog-tabs" role="tablist">
          {tabs.map((entry) => (
            <button
              key={entry.id}
              type="button"
              role="tab"
              aria-selected={tab === entry.id}
              className={["ic-link-dialog-tab", tab === entry.id && "selected"]
                .filter(Boolean)
                .join(" ")}
              onClick={() => setTab(entry.id)}
            >
              {entry.label}
            </button>
          ))}
        </div>
        {tab === "external" && (
          <Input
            autoFocus
            placeholder={t("link_dialog.url_placeholder")}
            value={url}
            error={urlError}
            helperText={urlError ? t("link_dialog.invalid_url") : undefined}
            onChange={(event) => setUrl(event.target.value)}
            onKeyDown={stopEditorKeys}
          />
        )}
        {tab === "document" && (
          <>
            <Select
              value={sheetName}
              options={sheetNames.map((name) => ({ value: name, label: name }))}
              onChange={setSheetName}
            />
            <Input
              autoFocus
              placeholder={t("link_dialog.cell_reference_placeholder")}
              value={cellRef}
              error={cellRefError}
              helperText={
                cellRefError ? t("link_dialog.invalid_reference") : undefined
              }
              onChange={(event) => setCellRef(event.target.value)}
              onKeyDown={stopEditorKeys}
            />
          </>
        )}
        {tab === "email" && (
          <>
            <Input
              autoFocus
              placeholder={t("link_dialog.email_placeholder")}
              value={email}
              error={emailError}
              helperText={
                emailError ? t("link_dialog.invalid_email") : undefined
              }
              onChange={(event) => setEmail(event.target.value)}
              onKeyDown={stopEditorKeys}
            />
            <Input
              placeholder={t("link_dialog.subject_placeholder")}
              value={subject}
              onChange={(event) => setSubject(event.target.value)}
              onKeyDown={stopEditorKeys}
            />
          </>
        )}
        <Input
          placeholder={t("link_dialog.tooltip_placeholder")}
          value={tooltip}
          onChange={(event) => setTooltip(event.target.value)}
          onKeyDown={stopEditorKeys}
        />
        <Input
          placeholder={t("link_dialog.label_placeholder")}
          value={label}
          onChange={(event) => setLabel(event.target.value)}
          onKeyDown={stopEditorKeys}
        />
      </div>
      <div className="ic-modal-dialog-footer ic-link-dialog-footer">
        {initialLink ? (
          <Button
            size="md"
            variant="ghost"
            className="ic-link-dialog-delete"
            startIcon={<Trash2 size={14} />}
            onClick={handleDelete}
          >
            {t("link_dialog.delete")}
          </Button>
        ) : (
          <span />
        )}
        <Button
          ref={saveButtonRef}
          size="md"
          startIcon={<Check size={14} />}
          onClick={handleSave}
          disabled={!canSave}
        >
          {t("link_dialog.save")}
        </Button>
      </div>
    </ModalDialog>
  );
}

LinkDialog.displayName = "LinkDialog";
