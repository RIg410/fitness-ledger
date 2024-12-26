import { tg_init, showPopup, close } from "./tg.js";
import { auth } from "./auth.js";
import { reload_me } from "./state/me.js";
import { init as initNavi } from "./navibar.js";


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
    
    console.log("Loading main parts...");
    await Promise.all([
        refresh(),
        loadParts('view/home/home.html', 'main-home-frame'),
        loadParts('view/calendar/calendar.html', 'main-schedule-frame'),
        loadParts('view/programs/programs.html', 'main-programs-frame'),
        loadParts('view/instructors/instructors.html', 'main-instructors-frame'),
        loadParts('view/more/more.html', 'main-more-frame')
    ]);
    console.log("Main parts loaded");
    initNavi();
}

async function loadParts(url, viewId) {
    let response = await fetch(url);
    let html = await response.text();
    document.getElementById(viewId).innerHTML = html;
}

export async function refresh() {
    try {
        await reload_me();
        console.log("refresh");
    } catch (e) {
        console.error("Failed to reload me", e);
        showPopup('Произошла ошибка при загрузке данных. Попробуйте перезагрузить страницу.', [
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