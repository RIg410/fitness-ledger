import { prepareCalendarView } from './pages/calendar.js';
import { selectionChanged } from './tg.js';

export function init() {
    document.getElementById('main-home-btn').addEventListener('click', () => showView('main-home-frame'));
    document.getElementById('main-schedule-btn').addEventListener('click', () => showView('main-schedule-frame'));
    document.getElementById('main-programs-btn').addEventListener('click', () => showView('main-programs-frame'));
    document.getElementById('main-instructors-btn').addEventListener('click', () => showView('main-instructors-frame'));
    document.getElementById('main-more-btn').addEventListener('click', () => showView('main-more-frame'));
}


export async function showView(viewId: string) {
    selectionChanged();
    switch (viewId) {
        case "main-schedule-frame":
            await prepareCalendarView();
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
