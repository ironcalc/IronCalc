import { Button } from "@ironcalc/workbook";
import { Orbit } from "lucide-react";

const MARGARET_URL =
  "https://en.wikipedia.org/wiki/Margaret_Hamilton_(software_engineer)";

export function MargaretLink() {
  return (
    <Button
      type="button"
      onClick={() => {
        window.open(MARGARET_URL, "_blank", "noopener,noreferrer");
      }}
      startIcon={<Orbit />}
      className="app-ic-margaret-link"
      aria-label="Margaret"
    >
      <span className="app-ic-margaret-link-text">Margaret</span>
    </Button>
  );
}
