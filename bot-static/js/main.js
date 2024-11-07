import { tg_init } from "./tg.js";
import { auth } from "./auth.js";
import { init as initNavi } from "./navibar.js";

async function run() {
    console.log("Starting...");
    tg_init();
    try {
        await auth();
    } catch (e) {
        console.error("Failed to auth", e);

    }
    console.log("Loading main parts...");
    await Promise.all([
        loadParts('pages/home.html', 'main-home-frame'),
        loadParts('pages/schedule.html', 'main-schedule-frame'),
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