export interface Template {
  id: string;
  titleKey: string;
  categoryKey: string;
  categoryId: string;
}

export const TEMPLATES: Template[] = [
  {
    id: "mortgage_calculator",
    titleKey: "welcome_dialog.templates.mortgage_calculator",
    categoryKey: "welcome_dialog.templates.category_finance",
    categoryId: "finance",
  },
  {
    id: "invoice",
    titleKey: "welcome_dialog.templates.invoice",
    categoryKey: "welcome_dialog.templates.category_finance",
    categoryId: "finance",
  },
  {
    id: "eu_salary_calculator",
    titleKey: "welcome_dialog.templates.eu_salary_calculator",
    categoryKey: "welcome_dialog.templates.category_finance",
    categoryId: "finance",
  },
  {
    id: "portfolio_tracker",
    titleKey: "welcome_dialog.templates.portfolio_tracker",
    categoryKey: "welcome_dialog.templates.category_finance",
    categoryId: "finance",
  },
  {
    id: "weekly_timesheet",
    titleKey: "welcome_dialog.templates.weekly_timesheet",
    categoryKey: "welcome_dialog.templates.category_project_management",
    categoryId: "project_management",
  },
  {
    id: "project_tracker",
    titleKey: "welcome_dialog.templates.project_tracker",
    categoryKey: "welcome_dialog.templates.category_project_management",
    categoryId: "project_management",
  },
  {
    id: "travel_expenses_tracker",
    titleKey: "welcome_dialog.templates.travel_expenses_tracker",
    categoryKey: "welcome_dialog.templates.category_lifestyle",
    categoryId: "lifestyle",
  },
  {
    id: "yearly_calendar",
    titleKey: "welcome_dialog.templates.yearly_calendar",
    categoryKey: "welcome_dialog.templates.category_lifestyle",
    categoryId: "lifestyle",
  },
  {
    id: "events_tracker",
    titleKey: "welcome_dialog.templates.events_tracker",
    categoryKey: "welcome_dialog.templates.category_lifestyle",
    categoryId: "lifestyle",
  },
];

export const TEMPLATE_CATEGORIES: { id: string; labelKey: string }[] =
  Array.from(
    new Map(TEMPLATES.map((t) => [t.categoryId, t.categoryKey])).entries(),
  ).map(([id, labelKey]) => ({ id, labelKey }));
