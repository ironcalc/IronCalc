import { House, TicketsPlane } from "lucide-react";
import { useTranslation } from "react-i18next";
import TemplatesListItem from "./TemplatesListItem";
import "./welcome-dialog.css";

function TemplatesList(props: {
  selectedTemplate: string;
  handleTemplateSelect: (templateId: string) => void;
}) {
  const { selectedTemplate, handleTemplateSelect } = props;
  const { t } = useTranslation();
  return (
    <div className="app-ic-wd-templates-list">
      <TemplatesListItem
        title={t("welcome_dialog.templates.mortgage_calculator")}
        description={t(
          "welcome_dialog.templates.mortgage_calculator_description",
        )}
        icon={<House />}
        iconColor="#2F80ED"
        active={selectedTemplate === "mortgage_calculator"}
        onClick={() => handleTemplateSelect("mortgage_calculator")}
      />
      <TemplatesListItem
        title={t("welcome_dialog.templates.travel_expenses_tracker")}
        description={t(
          "welcome_dialog.templates.travel_expenses_tracker_description",
        )}
        icon={<TicketsPlane />}
        iconColor="#EB5757"
        active={selectedTemplate === "travel_expenses_tracker"}
        onClick={() => handleTemplateSelect("travel_expenses_tracker")}
      />
    </div>
  );
}

export default TemplatesList;
