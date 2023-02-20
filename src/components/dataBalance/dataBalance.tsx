import { ScoreMeter } from "@cred/neopop-web/lib/components";
import { useState } from "preact/hooks";
import { Credentials, Traffic, TrafficUnits } from "../../types";

import styles from "./dataBalance.module.scss";

function DataInfo(props: { title: string; amount: number; unit: string }) {
    return (
        <div class={styles.dataInfo}>
            <div>{props.title}</div>
            <div>
                {Math.round(props.amount * 100) / 100}
                <span class={styles.dataUnit}>{props.unit}</span>
            </div>
        </div>
    );
}

export function DataBalance(props: {
    credentials: Credentials;
    traffic: Traffic;
    trafficUnits: TrafficUnits;
}) {
    const [toShow, show] = useState<boolean>(false);

    if (
        props.credentials.username === "" ||
        props.credentials.password === "" ||
        props.traffic.total === 0
    ) {
        show(false);
    } else if (props.traffic.total !== 0) {
        show(true);
    }

    return toShow ? (
        <div class={styles.dataContainer}>
            <ScoreMeter
                key={props.traffic.remaining}
                reading={Math.round(props.traffic.remaining)}
                scoreDesc={props.trafficUnits.remaining}
                oldReading={0}
                lowerLimit={0}
                upperLimit={props.traffic.total}
                type={
                    props.traffic.remaining < props.traffic.total / 4
                        ? "poor"
                        : props.traffic.remaining < props.traffic.total / 2
                        ? "average"
                        : "excellent"
                }
            />
            <div class={styles.infoContainer}>
                <DataInfo
                    title="Data Limit:"
                    amount={props.traffic.total}
                    unit={props.trafficUnits.total}
                />
                <DataInfo
                    title="Data Used:"
                    amount={props.traffic.used}
                    unit={props.trafficUnits.used}
                />
                <DataInfo
                    title="Data Left:"
                    amount={props.traffic.remaining}
                    unit={props.trafficUnits.remaining}
                />
            </div>
        </div>
    ) : (
        <></>
    );
}
