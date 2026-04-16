import { createContext, type ReactNode, useContext, useState } from "react";
import { createPortal } from "react-dom";

const PortalContext = createContext<HTMLElement | null>(null);

/**
 * Renders a container for portaled content and provides it to descendants
 * via context. The container lives inside the widget tree, so portaled
 * content inherits the widget's theme variables and stylesheet scope —
 * including when mounted inside a shadow DOM.
 */
export function PortalProvider({ children }: { children: ReactNode }) {
  const [container, setContainer] = useState<HTMLDivElement | null>(null);
  return (
    <PortalContext.Provider value={container}>
      {children}
      <div ref={setContainer} />
    </PortalContext.Provider>
  );
}

/**
 * Portals `children` into the widget's portal target. Renders nothing until
 * the target is ready, so callers never portal into a stale fallback (e.g.
 * `document.body`) before the widget has mounted.
 */
export function Portal({ children }: { children: ReactNode }) {
  const target = useContext(PortalContext);
  return target ? createPortal(children, target) : null;
}
