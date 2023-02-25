import styles from "./app.module.scss";
import { emit, listen, Event } from "@tauri-apps/api/event";
import { Login } from "./components/login/login";
import { ElevatedCard, ToastContainer } from "@cred/neopop-web/lib/components";
import { useEffect, useState } from "preact/hooks";
import { DataBalance } from "./components/dataBalance/dataBalance";
import { Credentials, Traffic, TrafficUnits } from "./types";
import { Credits } from "./components/credits/credits";

import initLogo from "./assets/logos/bits-goa.png";
import initBG from "./assets/backgrounds/bits-goa.jpg";

export function App() {
    const [credentials, setCredentials] = useState<Credentials>({
        username: "",
        password: "",
    });

    const [traffic, setTraffic] = useState<Traffic>({
        total: 0,
        last: 0,
        current: 0,
        used: 0,
        remaining: 0,
    });

    const [trafficUnits, setTrafficUnits] = useState<TrafficUnits>({
        total: "",
        last: "",
        current: "",
        used: "",
        remaining: "",
    });

    const [capsLock, setCapsLock] = useState(false);
    const [logo, setLogo] = useState(initLogo);
    const [BG, setBG] = useState(initBG);

    useEffect(() => {
        listen("credentials", (creds: Event<Credentials>) => {
            setCredentials({
                username: decodeURIComponent(creds.payload.username),
                password: decodeURIComponent(creds.payload.password),
            });
        });
        listen("traffic", (traffic: Event<Traffic>) => {
            setTraffic(traffic.payload);
        });
        listen("traffic_units", (traffic_units: Event<TrafficUnits>) => {
            setTrafficUnits(traffic_units.payload);
        });
        document.documentElement.style.setProperty(
            "background-image",
            `url(${BG})`
        );
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
                    <DataBalance
                        credentials={credentials}
                        traffic={traffic}
                        trafficUnits={trafficUnits}
                    />
                </div>
                <Credits />
            </ElevatedCard>
        </div>
    );
}
