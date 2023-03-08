type Credentials = {
    username: string;
    password: string;
};

type Traffic = {
    total: number;
    last: number;
    current: number;
    used: number;
    remaining: number;
};

type TrafficUnits = {
    total: string;
    last: string;
    current: string;
    used: string;
    remaining: string;
};

export { Credentials, Traffic, TrafficUnits };
