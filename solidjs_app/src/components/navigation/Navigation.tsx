import styles from "./navigation.module.css";

function Navigation() {
  const onkeydown = (event: KeyboardEvent) => {
    console.log("key pressed: ", event);
  };
  return (
    <div class={styles.navigation} onkeydown={onkeydown} tabIndex={0}>

    </div>
  );
}

export default Navigation;