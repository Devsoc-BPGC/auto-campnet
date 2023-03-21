import {
    Button,
    InputField,
    showToast,
    Tag,
    Toggle,
} from "@cred/neopop-web/lib/components";
import { StateUpdater, useEffect, useState } from "preact/hooks";
import { emit } from "@tauri-apps/api/event";

import styles from "./login.module.scss";

import { ChangeEvent } from "preact/compat";
import { invoke } from "@tauri-apps/api";

export function Login(props: {
    credentials: {
        username: string;
        password: string;
    };
    setCredentials: StateUpdater<{
        username: string;
        password: string;
    }>;
    logo: string;
    autolaunch: boolean;
}) {
    const [localUsername, setLocalUsername] = useState(
        props.credentials.username
    );
    const [localPassword, setLocalPassword] = useState(
        props.credentials.password
    );
    const [capsLock, setCapsLock] = useState(false);
    useEffect(() => {
        setLocalUsername(props.credentials.username);
        setLocalPassword(props.credentials.password);
    }, [props.credentials]);
    useEffect(() => {
        window.addEventListener("keyup", (event) =>
            setCapsLock(event.getModifierState("CapsLock"))
        );
    }, []);
    return (
        <div>
            <div class={styles.loginContainer}>
                <img
                    src={props.logo}
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
                {capsLock && (
                    <Tag
                        colorConfig={{
                            background: "#010B14",
                            color: "#F08D32",
                        }}
                    >
                        CapsLock is On!
                    </Tag>
                )}
                <div class={styles.autolaunchContainer}>
                    <span class={styles.autolaunchSwitch}>
                        <Toggle
                            isChecked={props.autolaunch}
                            colorMode={"light"}
                            onChange={(
                                event: ChangeEvent<HTMLInputElement>
                            ) => {
                                emit(
                                    "autolaunch",
                                    (event.target as HTMLInputElement).checked
                                );
                            }}
                        />
                    </span>
                    <span>Start at boot</span>
                </div>
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
                        invoke("credential_check", {
                            username: encodeURIComponent(localUsername),
                            password: encodeURIComponent(localPassword),
                        })
                            .then(() => {
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
                                    username: encodeURIComponent(localUsername),
                                    password: encodeURIComponent(localPassword),
                                });
                            })
                            .catch((err) => {
                                switch (err) {
                                    case "INVALIDCRED":
                                        showToast("Incorrect credentials", {
                                            type: "error",
                                            autoCloseTime: 3000,
                                            content: "Incorrect credentials!",
                                        });
                                        break;
                                    case "NOSOPHOS":
                                        showToast("Not on Sophos!", {
                                            type: "error",
                                            autoCloseTime: 3000,
                                            content: "Not on Sophos!",
                                        });
                                        break;
                                    case "UNKNOWN":
                                        showToast(
                                            "Could not verify credentials!",
                                            {
                                                type: "error",
                                                autoCloseTime: 3000,
                                                content:
                                                    "Could not verify credentials!",
                                            }
                                        );
                                        break;
                                }
                            });
                    }}
                >
                    Save
                </Button>
            </div>
        </div>
    );
}
