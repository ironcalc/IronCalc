import type { Model } from "@ironcalc/wasm";
import styles from "./workbook.module.css";
import Toolbar from "./toolbar/Toolbar";
import Navigation from "./navigation/Navigation";
import FormulaBar from "./formulabar/FormulaBar";
import Worksheet from "./Worksheet/Worksheet";

function Workbook(props: { model: Model }) {
  const onkeydown = (event: KeyboardEvent) => {
    console.log("key pressed: ", event);
  };
  return (
    <div class={styles.workbook} onkeydown={onkeydown} tabIndex={0}>
      <Toolbar></Toolbar>
      {/* {props.model.getFormattedCellValue(0, 1, 1)} */}
      <FormulaBar></FormulaBar>
      <Worksheet></Worksheet>
      <Navigation></Navigation>
    </div>
  );
}

export default Workbook;
