import type { Model } from "@ironcalc/workbook";
import { useEffect, useState } from "react";
import { shareModel } from "../rpc";

export function useShareDialog(model: Model | undefined): {
  url: string;
  copied: boolean;
  handleCopy: () => Promise<void>;
} {
  const [url, setUrl] = useState("");
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!model) {
      return;
    }
    const generateUrl = async () => {
      const bytes = model.toBytes();
      const hash = await shareModel(bytes);
      setUrl(`${location.origin}/?model=${hash}`);
    };
    generateUrl();
  }, [model]);

  useEffect(() => {
    if (!copied) {
      return;
    }
    const timeoutId = setTimeout(() => setCopied(false), 2000);
    return () => clearTimeout(timeoutId);
  }, [copied]);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
    } catch (err) {
      console.error("Failed to copy text: ", err);
    }
  };

  return { url, copied, handleCopy };
}
