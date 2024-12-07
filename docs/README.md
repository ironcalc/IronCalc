# IronCalc Docs

This repository contains IronCalc's end-user documentation. Here, you can explore supported features, functions, and more.

## Prerequisites

To manage the documentation, we use [VitePress](https://vitepress.dev/guide/what-is-vitepress), a Static Site Generator (SSG).

First, ensure you have [nodejs](https://nodejs.org/) installed in your system.

## Installation

Start installing the required dependencies by changing directory to the _docs_ folder of the IronCalc repository and running the following command in your terminal:

```bash
npm install
```

## Running the Project

Start a development instance of the documentation server with:

```bash
npm run dev
```

After running the command, you can view the documentation in your browser at http://localhost:5173 if the port is available.
Making changes to the Markdown will automatically reload your browser.


## Build the Project

To deploy the project:

```bash
npm run build
```

The project will be build in `src/.vitepress/dist`

