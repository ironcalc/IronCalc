import type { ReactNode } from "react";
import { createPortal } from "react-dom";

/**
 * Portals into the nearest `.ic-root` ancestor of `anchor` or renders nothing if there is none.
 */
export function createAnchoredPortal(
  children: ReactNode,
  anchor: Element | null | undefined,
) {
  const container = anchor?.closest<HTMLElement>(".ic-root");
  return container ? createPortal(children, container) : null;
}
