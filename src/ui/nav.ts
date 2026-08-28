export type Route =
    | { page: "hotkeys" }
    | { page: "editor"; id: string | null };

const PAGE_IDS = ["hotkeys", "editor"] as const;

export function parseRoute(): Route {
    const raw = location.hash.replace(/^#\/?/, "");
    if (raw === "editor" || raw.startsWith("editor/")) {
        const encoded = raw === "editor" ? "" : raw.slice("editor/".length);
        if (!encoded) return {page: "editor", id: null};
        try {
            return {page: "editor", id: decodeURIComponent(encoded)};
        } catch {
            return {page: "editor", id: encoded};
        }
    }
    return {page: "hotkeys"};
}

export function hrefFor(route: Route): string {
    if (route.page === "editor") {
        return route.id ? `#editor/${encodeURIComponent(route.id)}` : "#editor";
    }
    return "#hotkeys";
}

export function go(route: Route) {
    const next = hrefFor(route);
    const current = location.hash.startsWith("#") ? location.hash : `#${location.hash}`;
    if (current === next) return;
    location.hash = next.slice(1);
}

export function mountNav(onChange?: (route: Route) => void) {
    let current: string | null = null;

    const apply = () => {
        const route = parseRoute();

        for (const id of PAGE_IDS) {
            const active = route.page === id;
            document.querySelector(`[data-page="${id}"]`)?.classList.toggle("is-active", active);
        }

        document.querySelector("[data-add-hotkey]")?.classList.toggle("is-hidden", route.page !== "hotkeys");

        const key = route.page === "editor" ? `editor:${route.id ?? ""}` : route.page;
        if (current !== key) {
            current = key;
            onChange?.(route);
        }
    };

    window.addEventListener("hashchange", apply);
    apply();
}
