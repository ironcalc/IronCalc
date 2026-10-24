import { Loader2, Minus, Plus } from "lucide-react";
import * as pdfjsLib from "pdfjs-dist";
import pdfjsWorker from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

pdfjsLib.GlobalWorkerOptions.workerSrc = pdfjsWorker;

interface PageDimension {
  pageNumber: number;
  originalWidth: number;
  originalHeight: number;
}

interface PdfPagesPreviewProperties {
  blob: Blob | null;
  isLoading: boolean;
  error: string | null;
  onRetry?: () => void;
  onPageCountChange?: (count: number) => void;
  onBackgroundClick?: () => void;
}

export function PdfPagesPreview({
  blob,
  isLoading,
  error,
  onRetry,
  onPageCountChange,
  onBackgroundClick,
}: PdfPagesPreviewProperties) {
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const [containerWidth, setContainerWidth] = useState<number>(0);
  const [pages, setPages] = useState<PageDimension[]>([]);
  const [zoom, setZoom] = useState<number>(1.0);
  const [isRendering, setIsRendering] = useState<boolean>(false);

  const canvasRefs = useRef<Map<number, HTMLCanvasElement>>(new Map());
  const pdfDocRef = useRef<pdfjsLib.PDFDocumentProxy | null>(null);
  const renderTasksRef = useRef<pdfjsLib.RenderTask[]>([]);

  // Observe container width for responsive page sizing
  useLayoutEffect(() => {
    const el = scrollContainerRef.current;
    if (!el) return;

    const updateWidth = () => {
      setContainerWidth(el.clientWidth);
    };
    updateWidth();

    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        if (entry.contentRect.width > 0) {
          setContainerWidth(entry.contentRect.width);
        }
      }
    });

    resizeObserver.observe(el);
    return () => resizeObserver.disconnect();
  }, []);

  // Load PDF document when blob changes
  useEffect(() => {
    if (!blob) {
      setPages([]);
      if (pdfDocRef.current) {
        try {
          pdfDocRef.current.destroy();
        } catch {
          // ignore
        }
        pdfDocRef.current = null;
      }
      return;
    }

    let cancelled = false;
    let currentLoadingTask: ReturnType<typeof pdfjsLib.getDocument> | null =
      null;
    let currentDoc: pdfjsLib.PDFDocumentProxy | null = null;

    async function loadPdf() {
      try {
        if (!blob) return;
        const arrayBuffer = await blob.arrayBuffer();
        if (cancelled) return;

        if (pdfDocRef.current) {
          try {
            pdfDocRef.current.destroy();
          } catch {
            // ignore
          }
          pdfDocRef.current = null;
        }

        currentLoadingTask = pdfjsLib.getDocument({
          data: arrayBuffer,
          cMapUrl: "https://cdn.jsdelivr.net/npm/pdfjs-dist@4.10.38/cmaps/",
          cMapPacked: true,
        });

        const doc = await currentLoadingTask.promise;
        if (cancelled) {
          try {
            doc.destroy();
          } catch {
            // ignore
          }
          return;
        }

        currentDoc = doc;
        pdfDocRef.current = doc;

        const pageDims: PageDimension[] = [];
        for (let i = 1; i <= doc.numPages; i++) {
          if (cancelled) return;
          const page = await doc.getPage(i);
          const viewport = page.getViewport({ scale: 1.0 });
          pageDims.push({
            pageNumber: i,
            originalWidth: viewport.width,
            originalHeight: viewport.height,
          });
        }

        if (cancelled) return;
        setPages(pageDims);
      } catch (err) {
        if (!cancelled) {
          console.error("Error loading PDF in pdfjs:", err);
        }
      }
    }

    loadPdf();

    return () => {
      cancelled = true;
      if (currentLoadingTask) {
        try {
          currentLoadingTask.destroy();
        } catch {
          // ignore
        }
      }
      if (currentDoc) {
        try {
          currentDoc.destroy();
        } catch {
          // ignore
        }
      }
      if (pdfDocRef.current) {
        try {
          pdfDocRef.current.destroy();
          pdfDocRef.current = null;
        } catch {
          // ignore
        }
      }
    };
  }, [blob]);

  // Helper to compute a page's display width and height
  const getPageDisplaySize = useCallback(
    (p: PageDimension) => {
      const availW =
        containerWidth > 0 ? Math.max(containerWidth - 64, 200) : 750;
      const baseScale = Math.min(availW / p.originalWidth, 1.25);
      const scale = Math.max(baseScale * zoom, 0.2);
      return {
        scale,
        width: Math.round(p.originalWidth * scale),
        height: Math.round(p.originalHeight * scale),
      };
    },
    [containerWidth, zoom],
  );

  // Render pages onto canvases whenever pages, containerWidth, or zoom changes
  useEffect(() => {
    const doc = pdfDocRef.current;
    if (!doc || pages.length === 0 || containerWidth <= 0) return;

    let cancelled = false;

    // Cancel any ongoing render tasks
    for (const task of renderTasksRef.current) {
      if (task && typeof task.cancel === "function") {
        try {
          task.cancel();
        } catch {
          // ignore
        }
      }
    }
    renderTasksRef.current = [];

    async function renderAllPages() {
      if (!doc) return;
      setIsRendering(true);
      const dpr = window.devicePixelRatio || 1;

      for (const pageInfo of pages) {
        if (cancelled) break;

        const canvas = canvasRefs.current.get(pageInfo.pageNumber);
        if (!canvas) continue;

        try {
          const page = await doc.getPage(pageInfo.pageNumber);
          if (cancelled) break;

          const { scale, width, height } = getPageDisplaySize(pageInfo);
          const viewport = page.getViewport({ scale });

          canvas.width = Math.round(width * dpr);
          canvas.height = Math.round(height * dpr);
          canvas.style.width = `${width}px`;
          canvas.style.height = `${height}px`;

          const ctx = canvas.getContext("2d", { alpha: false });
          if (!ctx) continue;

          ctx.fillStyle = "#ffffff";
          ctx.fillRect(0, 0, canvas.width, canvas.height);

          const renderContext = {
            canvasContext: ctx,
            viewport,
            transform: dpr !== 1 ? [dpr, 0, 0, dpr, 0, 0] : undefined,
          };

          const task = page.render(renderContext);
          renderTasksRef.current.push(task);

          await task.promise;
        } catch (err: unknown) {
          if ((err as { name?: string })?.name !== "RenderingCancelledException") {
            console.warn(`Page ${pageInfo.pageNumber} render warning:`, err);
          }
        }
      }

      if (!cancelled) {
        setIsRendering(false);
      }
    }

    renderAllPages();

    return () => {
      cancelled = true;
      for (const task of renderTasksRef.current) {
        if (task && typeof task.cancel === "function") {
          try {
            task.cancel();
          } catch {
            // ignore
          }
        }
      }
      renderTasksRef.current = [];
    };
  }, [pages, containerWidth, zoom, getPageDisplaySize]);

  const handleZoomIn = () => {
    setZoom((prev) => Math.min(Number((prev + 0.15).toFixed(2)), 2.0));
  };

  const handleZoomOut = () => {
    setZoom((prev) => Math.max(Number((prev - 0.15).toFixed(2)), 0.5));
  };

  useEffect(() => {
    onPageCountChange?.(pages.length);
  }, [pages.length, onPageCountChange]);

  const handleScrollAreaClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const el = scrollContainerRef.current;
    if (el) {
      const rect = el.getBoundingClientRect();
      const isScrollbar = e.clientX >= rect.left + el.clientWidth;
      if (isScrollbar) {
        return;
      }
    }
    onBackgroundClick?.();
  };

  const isUpdating = isLoading || isRendering;

  return (
    <div
      ref={scrollContainerRef}
      className="app-ic-epd-preview-scroll"
      onClick={handleScrollAreaClick}
    >
      {/* Pages Container - blurred while updating preview, sharp when ready */}
      <div
        className={`app-ic-epd-pages-stack${isUpdating && pages.length > 0 ? " app-ic-epd-pages-stack--blurred" : ""}`}
      >
        {pages.map((p) => {
          const { width, height } = getPageDisplaySize(p);
          return (
            <div
              key={p.pageNumber}
              className="app-ic-epd-page-card"
              style={{
                width: `${width}px`,
                height: `${height}px`,
                flexShrink: 0,
              }}
              onClick={(e) => e.stopPropagation()}
            >
              <canvas
                ref={(el) => {
                  if (el) {
                    canvasRefs.current.set(p.pageNumber, el);
                  } else {
                    canvasRefs.current.delete(p.pageNumber);
                  }
                }}
                className="app-ic-epd-page-canvas"
                style={{
                  width: `${width}px`,
                  height: `${height}px`,
                  display: "block",
                  flexShrink: 0,
                }}
              />
            </div>
          );
        })}
      </div>

      {/* Initial loading state before any pages exist (no floating toaster) */}
      {pages.length === 0 && isLoading && !error && (
        <div
          className="app-ic-epd-preview-initial-loading"
          onClick={(e) => e.stopPropagation()}
        >
          <Loader2 className="app-ic-epd-spin" size={28} />
        </div>
      )}

      {/* Error state */}
      {error && (
        <div
          className="app-ic-epd-preview-error-box"
          onClick={(e) => e.stopPropagation()}
        >
          <span className="app-ic-epd-preview-error-title">
            Preview unavailable
          </span>
          <span className="app-ic-epd-preview-error-msg">{error}</span>
          {onRetry && (
            <button
              type="button"
              className="app-ic-epd-retry-btn"
              onClick={onRetry}
              style={{ marginTop: 8 }}
            >
              Retry
            </button>
          )}
        </div>
      )}

      {pages.length > 0 && !error && (
        <div
          className="app-ic-epd-zoom-controls"
          onClick={(e) => e.stopPropagation()}
        >
          <button
            type="button"
            className="app-ic-epd-zoom-btn"
            onClick={handleZoomIn}
            aria-label="Zoom in"
            title="Zoom in"
            disabled={zoom >= 2.0}
          >
            <Plus size={16} />
          </button>
          <button
            type="button"
            className="app-ic-epd-zoom-btn"
            onClick={handleZoomOut}
            aria-label="Zoom out"
            title="Zoom out"
            disabled={zoom <= 0.5}
          >
            <Minus size={16} />
          </button>
        </div>
      )}
    </div>
  );
}
