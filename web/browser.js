// Original PCBcraft platform adapter. No project data is uploaded to a server.
export function open_project(maxBytes) {
    return new Promise((resolve, reject) => {
        const input = document.createElement("input");
        input.type = "file";
        input.accept = ".pc,application/json";
        input.hidden = true;
        document.body.append(input);
        const finish = value => { input.remove(); resolve(value); };
        input.addEventListener("cancel", () => finish(null), { once: true });
        input.addEventListener("change", async () => {
            try {
                const file = input.files?.[0];
                if (!file) { finish(null); return; }
                if (file.size > maxBytes) throw new Error("Project exceeds the 4 MiB limit.");
                const bytes = new Uint8Array(await file.arrayBuffer());
                if (bytes.byteLength > maxBytes) throw new Error("Project exceeds the 4 MiB limit.");
                finish([file.name, bytes]);
            } catch (error) { input.remove(); reject(String(error)); }
        }, { once: true });
        try { input.click(); } catch (error) { input.remove(); reject(String(error)); }
    });
}

export function download_project(name, bytes) {
    const blob = new Blob([bytes], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = name.replace(/[\\/:*?"<>|\x00-\x1f]/g, "_");
    if (!anchor.download.toLowerCase().endsWith(".pc")) anchor.download += ".pc";
    document.body.append(anchor);
    try { anchor.click(); } finally {
        anchor.remove();
        // Allow the browser to consume the blob before releasing it.
        setTimeout(() => URL.revokeObjectURL(url), 60_000);
    }
}

let dirty = false;
export function set_dirty(value) { dirty = value; }
window.addEventListener("beforeunload", event => {
    if (dirty) { event.preventDefault(); event.returnValue = ""; }
});
export function ready() {
    document.getElementById("loading")?.remove();
    document.getElementById("pcbcraft")?.focus();
    document.documentElement.dataset.pcbcraft = "ready";
}
export function failed(message) {
    const status = document.getElementById("loading");
    if (status) status.textContent = message;
    document.documentElement.dataset.pcbcraft = "failed";
}
