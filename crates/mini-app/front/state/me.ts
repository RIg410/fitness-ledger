import { UserView } from '../model/user.js';
import { get_token } from '../auth.js';

var me: UserView = null;
var last_update: number = 0;

const me_ttl = 60 * 1000;

export async function reload_me() {
    let jwt = get_token();
    if (!jwt) {
        console.error("No token. Can't reload me");
        return;
    }

    const settings = {
        method: 'GET',
        headers: {
            Accept: 'application/json',
            'Content-Type': 'application/json',
            "Authorization": "Bearer " + jwt
        }
    };
    const response = await fetch('/user', settings);
    if (response.status !== 200) {
        console.log(response);
        throw new Error(response.statusText);
    }

    const data = await response.json();
    console.log(data);
    // me = new UserView(data);
    last_update = Date.now();
}