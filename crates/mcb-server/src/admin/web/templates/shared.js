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
})();
