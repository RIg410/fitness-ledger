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
}

class TrainingInfo {
    id: string;
    name: string;
    start_at: Date;
    duration_nim: number;
    couch: string;
    free_places: number;
    total_places: number;
}

export async function loadWeek(date: Date, force: boolean): Promise<Day | null> {
    let week = getFromCache(date);

    return null
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

function makeWeekId(date: Date): Date {
    let day = date.getDay();
    let diff = date.getDate() - day + (day == 0 ? -6 : 1);
    date.setHours(0, 0, 0, 0);
    return new Date(date.setDate(diff));
}