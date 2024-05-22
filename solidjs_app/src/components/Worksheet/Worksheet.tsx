import styles from "./worksheet.module.css";

function Worksheet() {
  const onkeydown = (event: KeyboardEvent) => {
    console.log("key pressed: ", event);
  };
  return (
    <div class={styles.worksheet} onkeydown={onkeydown} tabIndex={0}>

    </div>
  );
}

export default Worksheet;