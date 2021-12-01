import { writable } from "svelte/store";
import type { Writable } from "svelte/store";
import MainForm from "./formData";

export const formData: Writable<MainForm> = writable(new MainForm());
export const jobId: Writable<number | null> = writable(null);
