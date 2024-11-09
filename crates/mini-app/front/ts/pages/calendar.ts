import { currentWeek, nextWeek, prevWeek } from "../ledger/calendar.js";
import { selectionChanged } from "../tg.js";

var selectedWeek: Date = currentWeek();
var selectedDay: Date = new Date();
var millesInDay: number = 24 * 60 * 60 * 1000;

export async function prepareCalendarView() {
    console.log("Preparing calendar view...");
    renderCalendar();
    document.getElementById("calendar-prev").addEventListener("click", () => selectPrevWeek());
    document.getElementById("calendar-next").addEventListener("click", () => selectNextWeek());
}


async function selectNextWeek() {
    selectionChanged();
    selectedWeek = nextWeek(selectedWeek);
    selectedDay = new Date(selectedDay.getTime() + 7 * 24 * 60 * 60 * 1000)
    renderCalendar();
}

async function selectPrevWeek() {
    selectionChanged();
    selectedWeek = prevWeek(selectedWeek);
    selectedDay = new Date(selectedDay.getTime() - 7 * 24 * 60 * 60 * 1000)
    renderCalendar();
}

function renderCalendar() {
    console.log("Rendering calendar...", selectedWeek);
    setMonthAndYear();
    setDays();
    setEvents();
    renderDays();
}

function setMonthAndYear() {
    let month = selectedWeek.toLocaleString('default', { month: 'long' });
    let year = selectedWeek.getFullYear();
    document.getElementById("calendar-month-val").innerText = month + " " + year;
}

function setDays() {
    let mon = selectedWeek.getDate();
    document.getElementById("mon-num").innerText = mon.toString();
    document.getElementById("tue-num").innerText = (mon + 1).toString();
    document.getElementById("wed-num").innerText = (mon + 2).toString();
    document.getElementById("thu-num").innerText = (mon + 3).toString();
    document.getElementById("fri-num").innerText = (mon + 4).toString();
    document.getElementById("sat-num").innerText = (mon + 5).toString();
    document.getElementById("sun-num").innerText = (mon + 6).toString();
}

function setEvents() {
    document.getElementById("mon-day").addEventListener("click", () => selectDay(0));
    document.getElementById("tue-day").addEventListener("click", () => selectDay(1));
    document.getElementById("wed-day").addEventListener("click", () => selectDay(2));
    document.getElementById("thu-day").addEventListener("click", () => selectDay(3));
    document.getElementById("fri-day").addEventListener("click", () => selectDay(4));
    document.getElementById("sat-day").addEventListener("click", () => selectDay(5));
    document.getElementById("sun-day").addEventListener("click", () => selectDay(6));
}

function renderDays() {
    let selectedWeekDay = selectedDay.getDay();
    let today = new Date().getTime();
    let mon = selectedWeek.getTime() + millesInDay;

    renderDay("mon-day", selectedWeekDay == 1, mon < today);
    renderDay("tue-day", selectedWeekDay == 2, mon + 1 * millesInDay < today);
    renderDay("wed-day", selectedWeekDay == 3, mon + 2 * millesInDay < today);
    renderDay("thu-day", selectedWeekDay == 4, mon + 3 * millesInDay < today);
    renderDay("fri-day", selectedWeekDay == 5, mon + 4 * millesInDay < today);
    renderDay("sat-day", selectedWeekDay == 6, mon + 5 * millesInDay < today);
    renderDay("sun-day", selectedWeekDay == 0, mon + 6 * millesInDay < today);
}

function renderDay(dayId: string, selected: boolean, inPast: boolean) {
    let day = document.getElementById(dayId);
    day.setAttribute("class", "");

    if (selected) {
        if (inPast) {
            day.classList.add("past-day-selected");
        } else {
            day.classList.add("day-selected");
        }
    } else {
        if (inPast) {
            day.classList.add("past-day");
        } else {
            day.classList.add("day");
        }
    }
}

function selectDay(dayOffset: number) {
    let newSelectedDay = new Date(new Date(selectedWeek).setDate(selectedWeek.getDate() + dayOffset));
    if (newSelectedDay === selectedDay) {
        return;
    }
    selectedDay = newSelectedDay;
    selectionChanged();
    renderDays();

}
