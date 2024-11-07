export async function auth() {
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
    const data = await fetchResponse.json();
    console.log(data);
    window.jwt = data
}

export function jwt() {
    window.jwt
}