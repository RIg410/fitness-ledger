import { showHeader } from "../../main.js";

export async function show(frame: string) {
    console.log("Showing", frame);
    showHeader(true);
}
