export interface Template {
  id: string;
  titleKey: string;
  categoryKey: string;
  categoryId: string;
}

export const TEMPLATES: Template[] = [
  {
    id: "yearly_calendar",
    titleKey: "welcome_dialog.templates.yearly_calendar",
    categoryKey: "welcome_dialog.templates.category_lifestyle",
    categoryId: "lifestyle",
  },
  {
    id: "crossword",
    titleKey: "welcome_dialog.templates.crossword",
    categoryKey: "welcome_dialog.templates.category_games",
    categoryId: "games",
  },
  {
    id: "travel_expenses_tracker",
    titleKey: "welcome_dialog.templates.travel_expenses_tracker",
    categoryKey: "welcome_dialog.templates.category_lifestyle",
    categoryId: "lifestyle",
  },
  {
    id: "invoice",
    titleKey: "welcome_dialog.templates.invoice",
    categoryKey: "welcome_dialog.templates.category_finance",
    categoryId: "finance",
  },
  {
    id: "gantt_project_tracker",
    titleKey: "welcome_dialog.templates.gantt_project_tracker",
    categoryKey: "welcome_dialog.templates.category_project_management",
    categoryId: "project_management",
  },
  {
    id: "weekly_timesheet",
    titleKey: "welcome_dialog.templates.weekly_timesheet",
    categoryKey: "welcome_dialog.templates.category_project_management",
    categoryId: "project_management",
  },
  {
    id: "wordle",
    titleKey: "welcome_dialog.templates.wordle",
    categoryKey: "welcome_dialog.templates.category_games",
    categoryId: "games",
  },
  {
    id: "event_calendar",
    titleKey: "welcome_dialog.templates.event_calendar",
    categoryKey: "welcome_dialog.templates.category_lifestyle",
    categoryId: "lifestyle",
  },
];

export const TEMPLATE_CATEGORIES: { id: string; labelKey: string }[] =
  Array.from(
    new Map(TEMPLATES.map((t) => [t.categoryId, t.categoryKey])).entries(),
  ).map(([id, labelKey]) => ({ id, labelKey }));
