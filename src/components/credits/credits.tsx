import styles from "./credits.module.scss";

import devsoc from "../../assets/devsoc.png";

export function Credits() {
    return (
        <div class={styles.footer}>
            <span class={styles.footerText}>Made with ❤️ and 🎵 by</span>
            <img src={devsoc} class={styles.logo} alt={"Developers' Society BITS Goa"} />
        </div>
    );
}
