<template>
  <div ref="container"></div>
</template>

<script setup lang="ts">
import { useData } from "vitepress";
import { onMounted, ref, watch } from "vue";

type IronCalcEmbedTheme = "light" | "dark" | "system";

declare global {
  interface Window {
    IronCalcEmbed?: {
      mount: (
        target: HTMLElement,
        options: {
          workbookBytes: ArrayBuffer;
          style?: Partial<CSSStyleDeclaration>;
          theme?: IronCalcEmbedTheme;
        },
      ) => HTMLIFrameElement;
      setTheme: (iframe: HTMLIFrameElement, theme: IronCalcEmbedTheme) => void;
    };
  }
}

const props = defineProps<{
  src: string; // e.g. "/examples/fv.ic"
}>();

const container = ref<HTMLElement | null>(null);
const { isDark } = useData();

async function loadScript() {
  if (window.IronCalcEmbed) return;

  await new Promise<void>((resolve, reject) => {
    const script = document.createElement("script");
    script.src = "https://embed.ironcalc.com/embed.js";
    script.async = true;
    script.onload = () => resolve();
    script.onerror = () => reject(new Error("Failed to load embed.js"));
    document.head.appendChild(script);
  });
}

onMounted(async () => {
  if (!container.value) return;

  await loadScript();
  const embed = window.IronCalcEmbed;
  if (!embed) return;

  const response = await fetch(props.src);
  if (!response.ok) {
    throw new Error(`Failed to load ${props.src}`);
  }
  const workbookBytes = await response.arrayBuffer();

  const iframe = embed.mount(container.value, {
    workbookBytes,
    theme: isDark.value ? "dark" : "light",
    style: {
      width: "100%",
      height: "600px",
      border: "1px solid var(--vp-c-divider)",
      borderRadius: "8px",
    },
  });

  // Follow the site's appearance toggle.
  watch(isDark, (dark) => embed.setTheme(iframe, dark ? "dark" : "light"));
});
</script>
