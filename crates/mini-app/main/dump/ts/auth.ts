import { initData, alert } from "./tg.js";

export async function auth() {
    let jwt = get_token();
    if (jwt) {
        const settings = {
            method: 'POST',
            headers: {
                Accept: 'application/json',
                'Content-Type': 'application/json',
                "Authorization": "Bearer " + jwt
            }
        };
        const response = await fetch('/auth', settings);
        console.log(response);
        if (response.status === 200) {
            return;
        }
    }

    let tg_data = initData();
    console.log(tg_data);
    const settings = {
        method: 'POST',
        headers: {
            Accept: 'application/json',
            'Content-Type': 'application/json',
        }
    };
    const response = await fetch('/auth?' + tg_data, settings);
    if (response.status !== 200) {
        console.log(response);
        throw new Error(response.statusText);
    }

    const data = await response.json();
    localStorage.setItem("jwt", data.key);
}

export function get_token(): string | null {
    try {
        return localStorage.getItem("jwt");
    } catch (e) {
        return null;
    }
}