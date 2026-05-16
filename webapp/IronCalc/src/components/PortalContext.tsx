import { createContext, type ReactNode, useContext, useState } from "react";
import { createPortal } from "react-dom";

const PortalContext = createContext<HTMLElement | null>(null);

/**
 * Sets up a portal target for descendants, unless an ancestor already did —
 * in which case popups bubble up to the outermost provider. Wrap your app
 * once at the top level to control where widget popups mount.
 */
export function PortalProvider({ children }: { children: ReactNode }) {
  const inherited = useContext(PortalContext);
  const [container, setContainer] = useState<HTMLDivElement | null>(null);

  if (inherited) {
    return <>{children}</>;
  }

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
