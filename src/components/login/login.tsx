import { Button, InputField, showToast } from "@cred/neopop-web/lib/components";
import { StateUpdater, useEffect, useState } from "preact/hooks";
import { fetch, Body } from "@tauri-apps/api/http";
import { emit } from "@tauri-apps/api/event";

import styles from "./login.module.scss";

import bits_logo from "../../assets/bitslogo.png";
import { ChangeEvent } from "preact/compat";

export function Login(props: {
    credentials: {
        username: string;
        password: string;
    };
    setCredentials: StateUpdater<{
        username: string;
        password: string;
    }>;
}) {
    const [localUsername, setLocalUsername] = useState(
        props.credentials.username
    );
    const [localPassword, setLocalPassword] = useState(
        props.credentials.password
    );
    useEffect(() => {
        setLocalUsername(props.credentials.username);
        setLocalPassword(props.credentials.password);
    }, [props.credentials]);
    return (
        <div>
            <div class={styles.loginContainer}>
                <img
                    src={bits_logo}
                    class={styles.bitsLogo}
                    alt={"BITS Goa Logo"}
                />
                <InputField
                    label="Username"
                    placeholder="f20xxyyyy"
                    id="username"
                    // @ts-ignore
                    type="text"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                        setLocalUsername(
                            (event.target as HTMLInputElement).value
                        )
                    }
                    value={localUsername}
                    autoFocus
                    style={{
                        margin: "0.5rem 0",
                    }}
                />
                <InputField
                    label="Password"
                    id="password"
                    placeholder="fdxxxxxxxx"
                    // @ts-ignore
                    type="password"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                        setLocalPassword(
                            (event.target as HTMLInputElement).value
                        )
                    }
                    value={localPassword}
                    style={{
                        margin: "0.5rem 0",
                    }}
                />
                <Button
                    variant="primary"
                    kind="elevated"
                    style={{
                        alignSelf: "flex-end",
                    }}
                    colorMode={"light"}
                    onClick={() => {
                        showToast("Verifying credentials", {
                            type: "warning",
                            autoCloseTime: 3000,
                            content: "Verifying credentials",
                        });
                        fetch(
                            "https://campnet.bits-goa.ac.in:8093/userportal/Controller",
                            {
                                method: "POST",
                                body: Body.form({
                                    mode: "451",
                                    json: JSON.stringify({
                                        username: localUsername,
                                        password: localPassword,
                                        languageid: 1,
                                        browser: "Chrome_106",
                                    }),
                                }),
                            }
                        )
                            .then((res: any) => {
                                if (res.data.status === 200) {
                                    showToast("Credentias verified!", {
                                        type: "success",
                                        autoCloseTime: 3000,
                                        content: "Credentias verified!",
                                    });
                                    props.setCredentials({
                                        username: localUsername,
                                        password: localPassword,
                                    });
                                    emit("save", {
                                        username:
                                            encodeURIComponent(localUsername),
                                        password:
                                            encodeURIComponent(localPassword),
                                    });
                                } else {
                                    showToast("Incorrect credentials!", {
                                        type: "error",
                                        autoCloseTime: 3000,
                                        content: "Incorrect credentials!",
                                    });
                                }
                            })
                            .catch((err) => console.error(err));
                    }}
                >
                    Save
                </Button>
            </div>
        </div>
    );
}
