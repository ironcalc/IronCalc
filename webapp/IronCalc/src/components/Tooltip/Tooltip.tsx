import type { ReactElement } from "react";

type TooltipProperties = {
  title: string;
  children: ReactElement;
};

export function Tooltip({ title, children }: TooltipProperties) {
  return (
    <span title={title} style={{ display: "inline-flex" }}>
      {children}
    </span>
  );
}