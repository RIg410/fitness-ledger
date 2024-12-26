import { headerId, showHeader } from "../../main.js";


export async function show(frame: string) {
    console.log("Showing", frame);
    console.log("Header ID", headerId());
    showHeader(false);
}
