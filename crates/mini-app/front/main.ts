import { tg_init, showPopup, close } from "./tg.js";
import { auth } from "./auth.js";

async function run() {
    console.log("Starting...");
    tg_init();

    try {
        await auth();
    } catch (e) {
        console.error("Failed to auth", e);
        showPopup('Произошла ошибка при авторизаци. Попробуйте перезагрузить страницу.', [
            { id: 'ok', type: 'default', text: 'Ок' },
        ], function (btn) {
            if (btn === 'ok') {
                close();
                return;
            }
        });
    }

    
}

window.addEventListener("load", run);