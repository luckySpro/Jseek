import { createApp } from "vue";
import App from "./App.vue";
import Panel from "./Panel.vue";
import { native } from "./api";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "./style.css";

// One bundle, two windows: "main" (search) and "panel" (menu-bar control panel).
const label = native ? getCurrentWebviewWindow().label : location.hash === "#panel" ? "panel" : "main";
createApp(label === "panel" ? Panel : App).mount("#app");
