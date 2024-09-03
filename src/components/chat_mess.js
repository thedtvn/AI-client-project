import { invoke } from '@tauri-apps/api/tauri';
class Message extends HTMLElement {
    constructor() {
        super();
    }

    async init(message, user = false) {
        this.setAttribute("class", "markdown-body message");
        if (user) {
            this.setAttribute("style", "align-self: end;");
        }
        this.innerHTML = await invoke('md_to_html', { text: message });;
        return this;
    }

    async load() {
        this.setAttribute("class", "markdown-body message");
        this.innerHTML = "<div class=\"loader\"></div>";
        return this;
    }
}

window.customElements.define("chat-message", Message);