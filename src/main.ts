import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";

// Platform-specific styling (e.g. the macOS title bar inset in app.css).
document.documentElement.dataset.platform = /Mac/.test(navigator.userAgent) ? "mac" : "other";

const app = mount(App, { target: document.getElementById("app")! });

export default app;
