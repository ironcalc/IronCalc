import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";
import "./fonts.css";
import "./i18n";
import { StrictMode } from "react";
import { initLocalStorage } from "./components/storage.ts";

// Preferences (theme, locale) are read synchronously during the first render,
// so localStorage must be in its current layout before React mounts.
initLocalStorage();

// biome-ignore lint: we know the 'root' element exists.
createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
