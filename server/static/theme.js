// theme.js — theme switcher for Slate Show admin UI
// Cycles through: light → dark → monokai → light
// Persists the chosen theme in localStorage so it survives page reloads

const THEMES = [
    { key: 'light',    label: '🌙 Dark',     next: 'dark'     },
    { key: 'dark',     label: '🎨 Monokai',  next: 'monokai'  },
    { key: 'monokai',  label: '☀️ Light',    next: 'light'    },
];

// Apply a theme by setting data-theme on <body> and updating the button label
function applyTheme(key) {
    // light theme has no data-theme attribute — it's the CSS default
    if (key === 'light') {
        document.body.removeAttribute('data-theme');
    } else {
        document.body.setAttribute('data-theme', key);
    }

    // Find the theme config and update the button to show what clicking will do next
    const theme = THEMES.find(t => t.key === key);
    const btn = document.getElementById('theme-toggle');
    if (btn && theme) btn.textContent = theme.label;
}

// Cycle to the next theme and save to localStorage
function cycleTheme() {
    const current = localStorage.getItem('theme') || 'light';
    const theme = THEMES.find(t => t.key === current) || THEMES[0];
    const next = theme.next;
    localStorage.setItem('theme', next);
    applyTheme(next);
}

// On load, restore the saved theme
(function () {
    const saved = localStorage.getItem('theme') || 'light';
    applyTheme(saved);
})();
