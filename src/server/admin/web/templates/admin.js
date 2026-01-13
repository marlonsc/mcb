/**
 * MCP Context Browser Admin - Centralized JavaScript Library
 *
 * Provides: API, toast, fmt, auth utilities for all admin pages.
 * Templates should use these instead of defining their own.
 */

// =============================================================================
// Authentication
// =============================================================================
const auth = {
    getToken() {
        return localStorage.getItem('auth_token');
    },
    setToken(token) {
        localStorage.setItem('auth_token', token);
    },
    clearToken() {
        localStorage.removeItem('auth_token');
    },
    isAuthenticated() {
        return !!this.getToken();
    },
    logout() {
        this.clearToken();
        window.location.href = '/login';
    }
};

// =============================================================================
// API Client
// =============================================================================
const API = {
    baseUrl: '/admin',

    async request(method, endpoint, data = null) {
        const url = endpoint.startsWith('/') ? endpoint : `${this.baseUrl}/${endpoint}`;
        const options = {
            method,
            headers: {
                'Content-Type': 'application/json',
            },
        };

        const token = auth.getToken();
        if (token) {
            options.headers['Authorization'] = `Bearer ${token}`;
        }

        if (data && (method === 'POST' || method === 'PUT' || method === 'PATCH')) {
            options.body = JSON.stringify(data);
        }

        const response = await fetch(url, options);

        if (response.status === 401) {
            auth.logout();
            throw new Error('Session expired. Please log in again.');
        }

        if (!response.ok) {
            const error = await response.json().catch(() => ({ message: response.statusText }));
            throw new Error(error.message || `HTTP error ${response.status}`);
        }

        // Handle empty responses
        const text = await response.text();
        return text ? JSON.parse(text) : null;
    },

    get(endpoint) {
        return this.request('GET', endpoint);
    },
    post(endpoint, data) {
        return this.request('POST', endpoint, data);
    },
    put(endpoint, data) {
        return this.request('PUT', endpoint, data);
    },
    patch(endpoint, data) {
        return this.request('PATCH', endpoint, data);
    },
    delete(endpoint) {
        return this.request('DELETE', endpoint);
    }
};

// =============================================================================
// Formatters
// =============================================================================
const fmt = {
    // Format numbers with locale-aware separators
    num(value, decimals = 0) {
        if (value === null || value === undefined) return '-';
        return Number(value).toLocaleString(undefined, {
            minimumFractionDigits: decimals,
            maximumFractionDigits: decimals
        });
    },

    // Format percentages
    percent(value, decimals = 1) {
        if (value === null || value === undefined) return '-';
        return `${this.num(value, decimals)}%`;
    },

    // Format bytes to human-readable
    bytes(bytes) {
        if (bytes === null || bytes === undefined || bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
    },

    // Format duration in seconds to human-readable
    duration(seconds) {
        if (!seconds || seconds < 0) return '0s';
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = Math.floor(seconds % 60);
        if (h > 0) return `${h}h ${m}m ${s}s`;
        if (m > 0) return `${m}m ${s}s`;
        return `${s}s`;
    },

    // Format milliseconds
    ms(value) {
        if (value === null || value === undefined) return '-';
        if (value < 1) return `${(value * 1000).toFixed(0)}μs`;
        if (value < 1000) return `${value.toFixed(1)}ms`;
        return `${(value / 1000).toFixed(2)}s`;
    },

    // Format dates
    date(value, options = {}) {
        if (!value) return '-';
        const date = new Date(value);
        return date.toLocaleDateString(undefined, {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
            ...options
        });
    },

    // Format time
    time(value) {
        if (!value) return '-';
        const date = new Date(value);
        return date.toLocaleTimeString(undefined, {
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit'
        });
    },

    // Format datetime
    datetime(value) {
        if (!value) return '-';
        return `${this.date(value)} ${this.time(value)}`;
    },

    // Format relative time (e.g., "2 hours ago")
    relative(value) {
        if (!value) return '-';
        const date = new Date(value);
        const now = new Date();
        const seconds = Math.floor((now - date) / 1000);

        if (seconds < 60) return 'just now';
        if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
        if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
        if (seconds < 604800) return `${Math.floor(seconds / 86400)}d ago`;
        return this.date(value);
    },

    // Truncate text
    truncate(text, length = 50) {
        if (!text || text.length <= length) return text;
        return text.substring(0, length) + '...';
    }
};

// =============================================================================
// Toast Notifications (Alpine.js store)
// =============================================================================
document.addEventListener('alpine:init', () => {
    // Modal store for managing modal state
    Alpine.store('modals', {
        // Track open state for each modal by name
        _open: {},

        isOpen(name) {
            return this._open[name] === true;
        },

        open(name) {
            this._open[name] = true;
        },

        close(name) {
            this._open[name] = false;
        },

        toggle(name) {
            this._open[name] = !this._open[name];
        }
    });

    Alpine.store('toast', {
        items: [],
        nextId: 0,

        add(message, type = 'info', title = null) {
            const id = this.nextId++;
            const defaultTitles = {
                success: 'Success',
                error: 'Error',
                warning: 'Warning',
                info: 'Info'
            };

            this.items.push({
                id,
                type,
                title: title || defaultTitles[type] || 'Notification',
                message,
                show: true
            });

            // Auto-remove after 5 seconds
            setTimeout(() => this.remove(id), 5000);
        },

        remove(id) {
            const index = this.items.findIndex(t => t.id === id);
            if (index > -1) {
                this.items.splice(index, 1);
            }
        },

        success(message, title) { this.add(message, 'success', title); },
        error(message, title) { this.add(message, 'error', title); },
        warning(message, title) { this.add(message, 'warning', title); },
        info(message, title) { this.add(message, 'info', title); }
    });
});

// Global toast shortcut
const toast = {
    success: (msg, title) => Alpine.store('toast').success(msg, title),
    error: (msg, title) => Alpine.store('toast').error(msg, title),
    warning: (msg, title) => Alpine.store('toast').warning(msg, title),
    info: (msg, title) => Alpine.store('toast').info(msg, title)
};

// =============================================================================
// Utility Functions
// =============================================================================
const utils = {
    // Escape HTML to prevent XSS
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    },

    // Debounce function calls
    debounce(fn, delay = 300) {
        let timeout;
        return (...args) => {
            clearTimeout(timeout);
            timeout = setTimeout(() => fn.apply(this, args), delay);
        };
    },

    // Throttle function calls
    throttle(fn, limit = 300) {
        let inThrottle;
        return (...args) => {
            if (!inThrottle) {
                fn.apply(this, args);
                inThrottle = true;
                setTimeout(() => inThrottle = false, limit);
            }
        };
    },

    // Copy text to clipboard
    async copyToClipboard(text) {
        try {
            await navigator.clipboard.writeText(text);
            toast.success('Copied to clipboard');
            return true;
        } catch (err) {
            toast.error('Failed to copy');
            return false;
        }
    },

    // Download data as file
    downloadFile(data, filename, type = 'application/json') {
        const blob = new Blob([typeof data === 'string' ? data : JSON.stringify(data, null, 2)], { type });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        a.click();
        URL.revokeObjectURL(url);
    },

    // Parse URL query parameters
    getQueryParams() {
        return Object.fromEntries(new URLSearchParams(window.location.search));
    },

    // Update URL without reload
    setQueryParam(key, value) {
        const url = new URL(window.location);
        if (value === null || value === undefined || value === '') {
            url.searchParams.delete(key);
        } else {
            url.searchParams.set(key, value);
        }
        window.history.pushState({}, '', url);
    }
};

// =============================================================================
// CSS Class Helpers
// =============================================================================
const css = {
    // Get status badge class
    statusBadge(status) {
        const map = {
            'healthy': 'badge-success',
            'running': 'badge-success',
            'active': 'badge-success',
            'available': 'badge-success',
            'success': 'badge-success',
            'degraded': 'badge-warning',
            'warning': 'badge-warning',
            'paused': 'badge-warning',
            'pending': 'badge-warning',
            'unhealthy': 'badge-danger',
            'error': 'badge-danger',
            'failed': 'badge-danger',
            'unavailable': 'badge-danger',
            'stopped': 'badge-gray',
            'inactive': 'badge-gray',
            'unknown': 'badge-gray'
        };
        return map[status?.toLowerCase()] || 'badge-gray';
    },

    // Get icon box class based on type
    iconBox(type) {
        const map = {
            'primary': 'icon-box-primary',
            'success': 'icon-box-success',
            'warning': 'icon-box-warning',
            'danger': 'icon-box-danger',
            'info': 'icon-box-primary',
            'gradient': 'icon-box-gradient'
        };
        return map[type] || 'icon-box-primary';
    },

    // Get alert class based on type
    alertClass(type) {
        const map = {
            'success': 'alert-success',
            'error': 'alert-danger',
            'danger': 'alert-danger',
            'warning': 'alert-warning',
            'info': 'alert-info'
        };
        return map[type?.toLowerCase()] || '';
    },

    // Get button class based on variant
    buttonClass(variant) {
        const map = {
            'primary': 'btn btn-primary',
            'secondary': 'btn btn-secondary',
            'ghost': 'btn btn-ghost',
            'danger': 'btn btn-danger'
        };
        return map[variant?.toLowerCase()] || 'btn btn-primary';
    }
};

// =============================================================================
// HTMX Integration
// =============================================================================
document.addEventListener('htmx:afterSwap', (event) => {
    // Re-initialize any Alpine.js components in swapped content
    if (typeof Alpine !== 'undefined') {
        Alpine.initTree(event.detail.target);
    }
});

document.addEventListener('htmx:responseError', (event) => {
    toast.error(`Request failed: ${event.detail.xhr.statusText}`);
});

// =============================================================================
// Export for use in modules (if needed)
// =============================================================================
if (typeof window !== 'undefined') {
    window.auth = auth;
    window.API = API;
    window.fmt = fmt;
    window.toast = toast;
    window.utils = utils;
    window.css = css;
}
