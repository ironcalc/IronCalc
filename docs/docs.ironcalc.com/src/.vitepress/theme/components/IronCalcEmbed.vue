<template>
  <div ref="container"></div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";

const props = defineProps<{
  src: string; // e.g. "/examples/fv.ic"
}>();

const container = ref<HTMLElement | null>(null);

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

  const response = await fetch(props.src);
  if (!response.ok) {
    throw new Error(`Failed to load ${props.src}`);
  }
  const workbookBytes = await response.arrayBuffer();

  window.IronCalcEmbed.mount(container.value, {
    workbookBytes,
    style: {
      width: "100%",
      height: "600px",
      border: "1px solid #e5e5e5",
      borderRadius: "8px",
    },
  });
});
</script>
