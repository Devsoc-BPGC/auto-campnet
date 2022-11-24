import styles from "./app.module.scss";
import { emit, listen, Event } from "@tauri-apps/api/event";
import { Login } from "./components/login/login";
import { ElevatedCard, ToastContainer } from "@cred/neopop-web/lib/components";
import { useEffect, useState } from "preact/hooks";
import { DataBalance } from "./components/dataBalance/dataBalance";
import { Credentials } from "./types";

export function App() {
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");

    useEffect(() => {
        listen("credentials", (creds: Event<Credentials>) => {
            setUsername(decodeURIComponent(creds.payload.username));
            setPassword(decodeURIComponent(creds.payload.password));
        });
    }, []);

    document.addEventListener("visibilitychange", () => {
        if (document.visibilityState === "hidden") emit("minimise");
    });

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
                        username={username}
                        password={password}
                        setUsername={setUsername}
                        setPassword={setPassword}
                    />
                    <DataBalance username={username} password={password} />
                </div>
            </ElevatedCard>
        </div>
    );
}
