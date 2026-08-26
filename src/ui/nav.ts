export type Route =
    | { page: "hotkeys" }
    | { page: "settings" }
    | { page: "editor"; id: string | null };

const PAGE_IDS = ["hotkeys", "settings", "editor"] as const;

function navTarget(route: Route): "hotkeys" | "settings" {
    return route.page === "settings" ? "settings" : "hotkeys";
}

export function parseRoute(): Route {
    const raw = location.hash.replace(/^#\/?/, "");
    if (raw === "settings") return {page: "settings"};
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
    if (route.page === "settings") return "#settings";
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
        const activeNav = navTarget(route);

        for (const id of PAGE_IDS) {
            const active = route.page === id;
            document.querySelector(`[data-page="${id}"]`)?.classList.toggle("is-active", active);
            document.querySelector(`[data-page-desc="${id}"]`)?.classList.toggle("is-active", active);
        }

        for (const id of ["hotkeys", "settings"] as const) {
            const nav = document.querySelector(`[data-nav="${id}"]`);
            const active = id === activeNav;
            nav?.classList.toggle("is-active", active);
            if (nav instanceof HTMLElement) {
                if (active) nav.setAttribute("aria-current", "page");
                else nav.removeAttribute("aria-current");
            }
        }

        const key = route.page === "editor" ? `editor:${route.id ?? ""}` : route.page;
        if (current !== key) {
            current = key;
            onChange?.(route);
        }
    };

    window.addEventListener("hashchange", apply);
    apply();
}
