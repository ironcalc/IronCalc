import styles from "./formulabar.module.css";

function FormulaBar() {
  const onkeydown = (event: KeyboardEvent) => {
    console.log("key pressed: ", event);
  };
  return (
    <div class={styles.toolbar} onkeydown={onkeydown} tabIndex={0}>

    </div>
  );
}

export default FormulaBar;