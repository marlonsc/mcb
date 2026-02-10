// Shared admin web UI utilities.
(function () {
    'use strict';

    function getAdminKey() {
        const key = localStorage.getItem('admin_api_key');
        if (key && key.trim() !== '') {
            return key;
        }
        return '';
    }

    function ensureAdminKey() {
        let key = getAdminKey();
        if (key !== '') {
            return key;
        }
        key = window.prompt('Enter admin API key:') || '';
        if (key.trim() !== '') {
            localStorage.setItem('admin_api_key', key.trim());
            return key.trim();
        }
        return '';
    }

    function bindHtmxAdminHeader() {
        if (!document.body) {
            return;
        }
        document.body.addEventListener('htmx:configRequest', function (evt) {
            const key = ensureAdminKey();
            if (key !== '') {
                evt.detail.headers['X-Admin-Key'] = key;
            }
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', bindHtmxAdminHeader);
    } else {
        bindHtmxAdminHeader();
    }

    window.adminFetch = async function (url, options) {
        const opts = options || {};
        const headers = Object.assign({}, opts.headers || {});
        const key = ensureAdminKey();
        if (key !== '') {
            headers['X-Admin-Key'] = key;
        }
        if (opts.body && !headers['Content-Type']) {
            headers['Content-Type'] = 'application/json';
        }
        return fetch(url, Object.assign({}, opts, { headers: headers }));
    };

    window.escapeHtml = function (text) {
        const div = document.createElement('div');
        div.textContent = text == null ? '' : String(text);
        return div.innerHTML;
    };

    window.parseJsonSafe = function (raw, fallback) {
        try {
            return JSON.parse(raw);
        } catch {
            return fallback;
        }
    };

    window.toNumberSafe = function (value, fallback) {
        const n = Number(value);
        return Number.isFinite(n) ? n : fallback;
    };

    window.toIntegerSafe = function (value, fallback) {
        const n = Number(value);
        return Number.isInteger(n) ? n : fallback;
    };

    window.toArraySafe = function (value) {
        return Array.isArray(value) ? value : [];
    };

    window.formatUptime = function (seconds) {
        const safe = window.toIntegerSafe(seconds, 0);
        const days = Math.floor(safe / 86400);
        const hours = Math.floor((safe % 86400) / 3600);
        const mins = Math.floor((safe % 3600) / 60);
        const secs = Math.floor(safe % 60);
        if (days > 0) return `${days}d ${hours}h ${mins}m`;
        if (hours > 0) return `${hours}h ${mins}m ${secs}s`;
        if (mins > 0) return `${mins}m ${secs}s`;
        return `${secs}s`;
    };

    var NAV_ITEMS = [
        { href: '/',            label: 'Dashboard' },
        { href: '/ui/config',   label: 'Config' },
        { href: '/ui/health',   label: 'Health' },
        { href: '/ui/jobs',      label: 'Jobs' },
        { href: '/ui/browse',   label: 'Browse' }
    ];

    function currentNavPath() {
        var p = window.location.pathname.replace(/\/+$/, '') || '/';
        if (p === '/ui') return '/';
        return p;
    }

    function isNavActive(href) {
        var cur = currentNavPath();
        if (href === '/') return cur === '/' || cur === '/ui';
        return cur.indexOf(href) === 0;
    }

    function getTheme() {
        return localStorage.getItem('mcb-theme') || 'auto';
    }

    function cycleTheme() {
        var order = { auto: 'light', light: 'dark', dark: 'auto' };
        var next = order[getTheme()] || 'auto';
        if (next === 'auto') {
            localStorage.removeItem('mcb-theme');
            document.documentElement.removeAttribute('data-theme');
        } else {
            localStorage.setItem('mcb-theme', next);
            document.documentElement.setAttribute('data-theme', next);
        }
        var btn = document.getElementById('mcb-theme-toggle');
        if (btn) btn.textContent = next.charAt(0).toUpperCase() + next.slice(1);
    }

    window.mcbInjectShell = function () {
        var shell = document.getElementById('app-shell');
        if (!shell) return;

        var cur = currentNavPath();

        var links = NAV_ITEMS.map(function (item) {
            var cls = 'nav-link' + (isNavActive(item.href) ? ' active' : '');
            return '<a href="' + item.href + '" class="' + cls + '">' + item.label + '</a>';
        }).join('');

        var themeLabel = getTheme();
        themeLabel = themeLabel.charAt(0).toUpperCase() + themeLabel.slice(1);

        var nav = '<nav class="app-nav">' +
            '<div class="nav-inner">' +
                '<div class="nav-brand">' +
                    '<span class="nav-brand-title">MCP Context Browser</span>' +
                    '<span class="nav-brand-sub">Admin</span>' +
                '</div>' +
                '<div class="nav-links">' +
                    links +
                    '<button id="mcb-theme-toggle" class="nav-theme-btn" title="Toggle Theme" aria-label="Toggle Theme" onclick="cycleTheme()">' + themeLabel + '</button>' +
                '</div>' +
            '</div>' +
        '</nav>';

        var content = shell.innerHTML;
        shell.innerHTML = '';
        shell.className = 'app-shell';

        var navDiv = document.createElement('div');
        navDiv.innerHTML = nav;
        shell.insertBefore(navDiv.firstChild, null);

        var main = document.createElement('main');
        main.className = 'app-main';
        main.innerHTML = content;
        shell.appendChild(main);

        var footer = document.createElement('footer');
        footer.className = 'app-footer';
        footer.innerHTML = '<div class="footer-inner">MCP Context Browser v0.1.5 | Admin Panel</div>';
        shell.appendChild(footer);
    };

    window.cycleTheme = cycleTheme;

    (function () {
        var saved = localStorage.getItem('mcb-theme');
        if (saved) document.documentElement.setAttribute('data-theme', saved);
    })();

    function initShell() {
        if (document.getElementById('app-shell')) {
            window.mcbInjectShell();
        }
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initShell);
    } else {
        initShell();
    }
})();
