import { selectionChanged } from './tg.js';
import { show as homeShow } from './view/home/home.js';
import { show as calendarShow } from './view/calendar/calendar.js';
import { show as programsShow } from './view/programs/programs.js';
import { show as instructorsShow } from './view/instructors/instructors.js';
import { show as moreShow } from './view/more/more.js';

export async function init() {
    document.getElementById('main-home-btn').addEventListener('click', () => showView('main-home-frame'));
    document.getElementById('main-schedule-btn').addEventListener('click', () => showView('main-schedule-frame'));
    document.getElementById('main-programs-btn').addEventListener('click', () => showView('main-programs-frame'));
    document.getElementById('main-instructors-btn').addEventListener('click', () => showView('main-instructors-frame'));
    document.getElementById('main-more-btn').addEventListener('click', () => showView('main-more-frame'));
    await showView('main-home-frame');
}


export async function showView(viewId: string) {
    selectionChanged();
    switch (viewId) {
        case "main-home-frame":
            await homeShow('main-home-frame');
            break;
        case "main-schedule-frame":
            await calendarShow('main-schedule-frame');
            break;
        case "main-programs-frame":
            await programsShow('main-programs-frame');
            break;
        case "main-instructors-frame":
            await instructorsShow('main-instructors-frame');
            break;
        case "main-more-frame":
            await moreShow('main-more-frame');
            break;
        default:
            console.log("Unknown view", viewId);
    }


    const views = document.querySelectorAll('.navi-view');
    views.forEach(view => {
        let elm = view as HTMLElement;
        elm.style.display = 'none';
    });
    document.getElementById(viewId).style.display = 'block';
}
