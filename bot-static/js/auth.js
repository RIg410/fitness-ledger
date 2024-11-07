async function auth() {
    let tg_data = Telegram.WebApp.initData();
    console.log(data);
    const settings = {
        method: 'POST',
        headers: {
            Accept: 'application/json',
            'Content-Type': 'application/json',
        }
    };
    const response = await fetch(`/auth?{data}`, settings);
    const data = await fetchResponse.json();
    console.log(data);
}