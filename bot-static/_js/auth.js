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

    let tg_data = Telegram.WebApp.initData;
    console.log(tg_data);
    const settings = {
        method: 'POST',
        headers: {
            Accept: 'application/json',
            'Content-Type': 'application/json',
        }
    };
    const response = await fetch('/auth?' + tg_data, settings);
    const data = await response.json();
    localStorage.setItem("jwt", data.key);
}

export function get_token() {
    try {
        return localStorage.getItem("jwt");
    } catch (e) {
        return null;
    }
}