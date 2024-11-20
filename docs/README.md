# IronCalc Docs

This repository contains IronCalc's end-user documentation. Here, you can explore supported features, functions, and more.

## Prerequisites

To manage the documentation, we use [VitePress](https://vitepress.dev/guide/what-is-vitepress), a Static Site Generator (SSG). We also leverage [MathJax v3.0](https://vitepress.dev/guide/markdown#math-equations) for rendering mathematical equations.

First, ensure you have the following tools installed:

- **Node.js** (version 18 or higher)
- **npm** (comes bundled with Node.js)
- A terminal to access the VitePress CLI
- A text editor with Markdown syntax support (e.g., VS Code, Cursor)

## Installation

Start installing the required dependencies by running the following command in your terminal:

```bash
npm install
```

## Build the Project

Prepare the project for development by building it:

```bash
npm run build
```

## Running the Project

Start the development server with:

```bash
npm run dev
```

After running the command, you can view the documentation in your browser at http://localhost:3000 (or the URL displayed in your terminal).

## Project Structure

The documentation is organized as follows:

```plaintext
src
├── .vitepress
│   ├── theme
│   │   └── style.css
│   └── config.mts
├── features
├── functions
├── python-bindings
└── more
```

### Notes on the Structure

- **`.vitepress`**: Contains configuration and theming files for VitePress.
  - `theme/style.css`: Use this file to customize styles across the documentation.
  - `config.mts`: Modify this file to change global settings like navigation and layout.
- **`features`**: Describes the supported features of IronCalc.
- **`functions`**: Includes a comprehensive list of all functions, categorized as supported or unsupported.
- **`python-bindings`**: Documentation for using IronCalc with Python.
- **`more`**: Additional content or advanced topics related to IronCalc.
