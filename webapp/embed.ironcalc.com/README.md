# IronCalc Embed

Embed a fully functional **IronCalc** spreadsheet into any website using a single `<script>` and a small API.

This powers <https://embed.ironcalc.com/embed.js>


## 🚀 Quick Start

1. Include the script

```html
<script src="https://embed.ironcalc.com/embed.js"></script>
```

2. Add a container

```html
<div id="sheet"></div>
```

3. Mount the spreadsheet

```html
<script>
  async function main() {
    const response = await fetch("example.ic");
    const workbookBytes = await response.arrayBuffer();

    IronCalcEmbed.mount("#sheet", {
      workbookBytes,
      style: {
        width: "100%",
        height: "600px",
        border: "1px solid #e5e5e5",
        borderRadius: "8px",
      },
    });
  }

  main();
</script>
```

At the moment you can create an `ic` workbook using the `xlsx_2_icalc` utility
