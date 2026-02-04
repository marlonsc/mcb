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

    document.body.addEventListener('htmx:configRequest', function (evt) {
        const key = ensureAdminKey();
        if (key !== '') {
            evt.detail.headers['X-Admin-Key'] = key;
        }
    });

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
        div.textContent = text;
        return div.innerHTML;
    };

    window.formatUptime = function (seconds) {
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        const mins = Math.floor((seconds % 3600) / 60);
        const secs = Math.floor(seconds % 60);
        if (days > 0) return `${days}d ${hours}h ${mins}m`;
        if (hours > 0) return `${hours}h ${mins}m ${secs}s`;
        if (mins > 0) return `${mins}m ${secs}s`;
        return `${secs}s`;
    };
})();
