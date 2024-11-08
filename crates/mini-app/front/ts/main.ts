import { tg_init, showPopup, close } from "./tg.js";
import { auth } from "./auth.js";
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
            }
        });
    }
    console.log("Loading main parts...");
    await Promise.all([
        loadParts('pages/home.html', 'main-home-frame'),
        loadParts('pages/calendar.html', 'main-schedule-frame'),
        loadParts('pages/programs.html', 'main-programs-frame'),
        loadParts('pages/instructors.html', 'main-instructors-frame'),
        loadParts('pages/more.html', 'main-more-frame')
    ]);
    console.log("Main parts loaded");
    initNavi();
}

async function loadParts(url, viewId) {
    let response = await fetch(url);
    let html = await response.text();
    document.getElementById(viewId).innerHTML = html;
}

window.addEventListener("load", run);