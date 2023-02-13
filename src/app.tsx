import styles from "./app.module.scss";
import { emit, listen, Event } from "@tauri-apps/api/event";
import { Login } from "./components/login/login";
import { ElevatedCard, ToastContainer } from "@cred/neopop-web/lib/components";
import { useEffect, useState } from "preact/hooks";
import { DataBalance } from "./components/dataBalance/dataBalance";
import { Credentials } from "./types";

import initLogo from "./assets/logos/bits-goa.png";
import initBG from "./assets/backgrounds/bits-goa.jpg";

export function App() {
    const [credentials, setCredentials] = useState({
        username: "",
        password: "",
    });

    const [capsLock, setCapsLock] = useState(false);
    const [logo,setLogo] = useState(initLogo);
    const [BG,setBG] = useState(initBG);

    useEffect(() => {
        listen("credentials", (creds: Event<Credentials>) => {
            setCredentials({
                username: decodeURIComponent(creds.payload.username),
                password: decodeURIComponent(creds.payload.password),
            });
        });
        document.documentElement.style.setProperty("background-image", `url(${BG})`);
        document.addEventListener("visibilitychange", () => {
            if (document.visibilityState === "hidden") emit("minimise");
        });
        window.addEventListener("keyup", (event) =>
            setCapsLock(event.getModifierState("CapsLock"))
        );
    }, []);

    return (
        <div>
            <ToastContainer />
            <ElevatedCard
                backgroundColor="#0D0D0D"
                edgeColors={{
                    bottom: "#161616",
                    right: "#121212",
                }}
            >
                <div class={styles.mainContainer}>
                    <Login
                        credentials={credentials}
                        capsLock={capsLock}
                        setCredentials={setCredentials}
                        logo={logo}
                    />
                    <DataBalance credentials={credentials} />
                </div>
            </ElevatedCard>
        </div>
    );
}
