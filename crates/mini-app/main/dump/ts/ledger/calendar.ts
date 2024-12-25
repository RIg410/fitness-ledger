import { get_token } from "../auth.js";

const DAY_TTL = 60 * 1000; // 1 minute

enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

class Day {
    id: string;
    dateTime: Date;
    weekday: Weekday;
    training: TrainingInfo[];

    constructor(id: string, date: Date, weekday: Weekday, training: TrainingInfo[]) {
        this.id = id;
        this.dateTime = date;
        this.weekday = weekday;
        this.training = training;
    }
}

class TrainingInfo {
    id: string;
    name: string;
    start_at: Date;
    duration_nim: number;
    couch: string;
    free_places: number;
    total_places: number;

    constructor(id: string, name: string, start_at: Date, duration_nim: number, couch: string, free_places: number, total_places: number) {
        this.id = id;
        this.name = name;
        this.start_at = start_at;
        this.duration_nim = duration_nim;
        this.couch = couch;
        this.free_places = free_places;
        this.total_places = total_places;
    }
}

export async function loadDate(date: Date, force: boolean): Promise<Day | null> {
    let week = getFromCache(date);

    return 
}

function getFromCache(date: Date): Day | null {
    try {
        let weekId = makeWeekId(date);
        let week = localStorage.getItem("week-" + weekId.toISOString());
        if (week == null) {
            return null;
        }
        return JSON.parse(week);
    } catch (e) {
        return null;
    }
}

export function makeWeekId(date: Date): Date {
    let day = date.getDay();
    let diff = date.getDate() - day + (day == 0 ? -6 : 1);
    date.setHours(0, 0, 0, 0);
    return new Date(date.setDate(diff));
}

export function nextWeek(date: Date): Date {
    let next = new Date(date);
    next.setDate(next.getDate() + 7);
    return next;
}

export function prevWeek(date: Date): Date {
    let prev = new Date(date);
    prev.setDate(prev.getDate() - 7);
    return prev;
}

export function currentWeek(): Date {
    let date = new Date();
    return makeWeekId(date);
}