import { ScoreMeter } from "@cred/neopop-web/lib/components";
import { fetch, getClient, Body, ResponseType } from "@tauri-apps/api/http";
import { useEffect, useState } from "preact/hooks";

import styles from "./dataBalance.module.scss";

function DataInfo(props: { title: string; amount: number; unit: string }) {
    return (
        <div class={styles.dataInfo}>
            <div>{props.title}</div>
            <div>
                {props.amount}
                <span class={styles.dataUnit}>{props.unit}</span>
            </div>
        </div>
    );
}

export function DataBalance(props: { username: string; password: string }) {
    const [datas, setDatas] = useState<Array<number>>([1, 0, 0, 1, 0]);
    const [units, setUnits] = useState<Array<string>>(["", "", "", "", ""]);
    const [toShow, show] = useState<boolean>(false);
    const [balanceTimeout, setBalanceTimeOut] = useState<NodeJS.Timeout>();

    useEffect(() => {
        clearTimeout(balanceTimeout);
        getBalance();
    }, [props]);

    function getBalance() {
        let cookie = "";
        let csrf = "";
        getClient({ maxRedirections: 1 }).then((client) =>
            client
                .post(
                    "https://campnet.bits-goa.ac.in:8093/userportal/Controller",
                    Body.form({
                        mode: "451",
                        json: JSON.stringify({
                            username: props.username,
                            password: props.password,
                            languageid: 1,
                            browser: "Chrome_106",
                        }),
                    })
                )
                .then(
                    (res) => (cookie = res.headers["set-cookie"].split(";")[0])
                )
                .then(() =>
                    fetch(
                        "https://campnet.bits-goa.ac.in:8093/userportal/webpages/myaccount/index.jsp",
                        {
                            method: "GET",
                            headers: {
                                Cookie: cookie,
                            },
                            responseType: ResponseType.Text,
                        }
                    )
                )
                .then((res) => {
                    //@ts-ignore
                    csrf = String(res.data).match(/k3n = '(.+)'/)[1];
                })
                .then(() =>
                    client.get(
                        "https://campnet.bits-goa.ac.in:8093/userportal/webpages/myaccount/AccountStatus.jsp",
                        {
                            headers: {
                                Cookie: cookie,
                                "X-CSRF-Token": csrf,
                                Referer:
                                    "https://campnet.bits-goa.ac.in:8093/userportal/webpages/myaccount/login.jsp",
                            },
                            query: {
                                popup: `${0}`,
                                t: `${Date.now()}`,
                            },
                            responseType: ResponseType.Text,
                        }
                    )
                )
                .then((res: any) => {
                    const nodes = new DOMParser().parseFromString(
                        res.data,
                        "text/html"
                    );
                    const trimmedNodes = [
                        ...nodes
                            .querySelector("#content3")
                            ?.querySelectorAll("td.tabletext")!!,
                    ].slice(-5);
                    setDatas(
                        trimmedNodes.map((iter: any) =>
                            Number(
                                (iter.childNodes[0].nodeValue as string).trim()
                            )
                        )
                    );
                    setUnits(
                        trimmedNodes.map((iter: any) =>
                            iter.children[0].id.replace(/Language./, "")
                        )
                    );
                    show(true);
                })
                .then(() => setBalanceTimeOut(setTimeout(getBalance, 15000)))
                .catch((err) => console.error(err))
        );
    }

    return toShow ? (
        <div class={styles.dataContainer}>
            <ScoreMeter
                key={datas[4]}
                reading={Math.round(datas[4])}
                oldReading={0}
                lowerLimit={0}
                upperLimit={datas[0]}
                scoreDesc={units[4]}
                type={
                    datas[4] < datas[0] / 5
                        ? "poor"
                        : datas[4] < datas[0] / 3
                        ? "average"
                        : "excellent"
                }
            />
            <div class={styles.infoContainer}>
                <DataInfo
                    title="Data Limit:"
                    amount={datas[0]}
                    unit={units[0]}
                />
                <DataInfo
                    title="Data Used:"
                    amount={datas[3]}
                    unit={units[3]}
                />
                <DataInfo
                    title="Data Left:"
                    amount={datas[4]}
                    unit={units[4]}
                />
            </div>
        </div>
    ) : (
        <></>
    );
}
