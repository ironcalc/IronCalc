/* @refresh reload */
import { render } from "solid-js/web";

import App from "./App";
// import "./index.css";
import "./theme.css";

const root = document.getElementById("root");

if (root) {
  render(() => <App />, root);
}
