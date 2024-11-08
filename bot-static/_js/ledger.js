import { get_token } from './auth.js';

export async function get_week(id, force) {
    let jwt = get_token();
    if (jwt) {
        const settings = {
            method: 'GET',
            headers: {
                Accept: 'application/json',
                'Content-Type': 'application/json',
                "Authorization": "Bearer " + jwt
            }
        };
        const response = await fetch('/schedule', settings);
        if (response.status === 200) {
            const data = await response.json();
            return data;
        }
    }
    return null;
}


