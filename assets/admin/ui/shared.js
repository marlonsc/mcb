/**
 * Admin UI Shared Utilities
 * Theme management, navigation helpers, and API utilities
 */

// ============================================================================
// Theme Management
// ============================================================================

/**
 * Get the current theme from localStorage or system preference
 * @returns {string} 'light' or 'dark'
 */
function getCurrentTheme() {
  const stored = localStorage.getItem('admin-theme');
  if (stored) {
    return stored;
  }
  
  // Check system preference
  if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    return 'dark';
  }
  
  return 'light';
}

/**
 * Set the theme and update the DOM
 * @param {string} theme - 'light' or 'dark'
 */
function setTheme(theme) {
  if (theme !== 'light' && theme !== 'dark') {
    console.warn(`Invalid theme: ${theme}. Using 'light'.`);
    theme = 'light';
  }
  
  // Update HTML element data-theme attribute
  document.documentElement.setAttribute('data-theme', theme);
  
  // Store preference
  localStorage.setItem('admin-theme', theme);
  
  // Dispatch custom event for other components to listen
  window.dispatchEvent(new CustomEvent('theme-changed', { detail: { theme } }));
}

/**
 * Toggle between light and dark themes
 * @returns {string} The new theme
 */
function toggleTheme() {
  const current = getCurrentTheme();
  const next = current === 'light' ? 'dark' : 'light';
  setTheme(next);
  return next;
}

/**
 * Initialize theme on page load
 */
function initializeTheme() {
  const theme = getCurrentTheme();
  setTheme(theme);
  
  // Listen for system theme changes
  if (window.matchMedia) {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
      if (!localStorage.getItem('admin-theme')) {
        setTheme(e.matches ? 'dark' : 'light');
      }
    });
  }
}

// ============================================================================
// Navigation Helpers
// ============================================================================

/**
 * Navigate to a route within the admin panel
 * @param {string} path - The path to navigate to (e.g., '/admin/dashboard')
 * @param {Object} options - Navigation options
 * @param {boolean} options.replace - Use replaceState instead of pushState
 * @param {Object} options.state - State object to pass
 */
function navigateTo(path, options = {}) {
  const { replace = false, state = {} } = options;
  
  if (replace) {
    window.history.replaceState(state, '', path);
  } else {
    window.history.pushState(state, '', path);
  }
  
  // Dispatch navigation event
  window.dispatchEvent(new CustomEvent('admin-navigate', { detail: { path, state } }));
}

/**
 * Get the current admin route
 * @returns {string} The current path
 */
function getCurrentRoute() {
  return window.location.pathname;
}

/**
 * Check if a route is active
 * @param {string} path - The path to check
 * @returns {boolean}
 */
function isRouteActive(path) {
  return getCurrentRoute() === path;
}

/**
 * Build an admin URL
 * @param {string} path - The path (e.g., 'dashboard', 'users/123')
 * @returns {string} The full URL
 */
function buildAdminUrl(path) {
  const basePath = '/admin';
  const cleanPath = path.startsWith('/') ? path : `/${path}`;
  return `${basePath}${cleanPath}`;
}

// ============================================================================
// Fetch Utilities
// ============================================================================

/**
 * Make an API request with error handling
 * @param {string} url - The URL to fetch
 * @param {Object} options - Fetch options
 * @returns {Promise<Object>} The parsed response
 * @throws {Error} If the request fails
 */
async function apiRequest(url, options = {}) {
  const {
    method = 'GET',
    headers = {},
    body = null,
    timeout = 30000,
  } = options;
  
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeout);
  
  try {
    const response = await fetch(url, {
      method,
      headers: {
        'Content-Type': 'application/json',
        ...headers,
      },
      body: body ? JSON.stringify(body) : null,
      signal: controller.signal,
    });
    
    clearTimeout(timeoutId);
    
    if (!response.ok) {
      const error = new Error(`HTTP ${response.status}: ${response.statusText}`);
      error.status = response.status;
      error.response = response;
      throw error;
    }
    
    const contentType = response.headers.get('content-type');
    if (contentType && contentType.includes('application/json')) {
      return await response.json();
    }
    
    return await response.text();
  } catch (error) {
    clearTimeout(timeoutId);
    
    if (error.name === 'AbortError') {
      throw new Error(`Request timeout after ${timeout}ms`);
    }
    
    throw error;
  }
}

/**
 * Make a GET request
 * @param {string} url - The URL to fetch
 * @param {Object} options - Additional options
 * @returns {Promise<Object>}
 */
function apiGet(url, options = {}) {
  return apiRequest(url, { ...options, method: 'GET' });
}

/**
 * Make a POST request
 * @param {string} url - The URL to fetch
 * @param {Object} body - The request body
 * @param {Object} options - Additional options
 * @returns {Promise<Object>}
 */
function apiPost(url, body, options = {}) {
  return apiRequest(url, { ...options, method: 'POST', body });
}

/**
 * Make a PUT request
 * @param {string} url - The URL to fetch
 * @param {Object} body - The request body
 * @param {Object} options - Additional options
 * @returns {Promise<Object>}
 */
function apiPut(url, body, options = {}) {
  return apiRequest(url, { ...options, method: 'PUT', body });
}

/**
 * Make a DELETE request
 * @param {string} url - The URL to fetch
 * @param {Object} options - Additional options
 * @returns {Promise<Object>}
 */
function apiDelete(url, options = {}) {
  return apiRequest(url, { ...options, method: 'DELETE' });
}

/**
 * Make a PATCH request
 * @param {string} url - The URL to fetch
 * @param {Object} body - The request body
 * @param {Object} options - Additional options
 * @returns {Promise<Object>}
 */
function apiPatch(url, body, options = {}) {
  return apiRequest(url, { ...options, method: 'PATCH', body });
}

// ============================================================================
// Utility Helpers
// ============================================================================

/**
 * Format a date for display
 * @param {Date|string|number} date - The date to format
 * @param {Object} options - Intl.DateTimeFormat options
 * @returns {string}
 */
function formatDate(date, options = {}) {
  const d = new Date(date);
  const defaultOptions = {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    ...options,
  };
  return new Intl.DateTimeFormat('en-US', defaultOptions).format(d);
}

/**
 * Debounce a function
 * @param {Function} fn - The function to debounce
 * @param {number} delay - The delay in milliseconds
 * @returns {Function}
 */
function debounce(fn, delay = 300) {
  let timeoutId;
  return function debounced(...args) {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn.apply(this, args), delay);
  };
}

/**
 * Throttle a function
 * @param {Function} fn - The function to throttle
 * @param {number} limit - The time limit in milliseconds
 * @returns {Function}
 */
function throttle(fn, limit = 300) {
  let inThrottle;
  return function throttled(...args) {
    if (!inThrottle) {
      fn.apply(this, args);
      inThrottle = true;
      setTimeout(() => {
        inThrottle = false;
      }, limit);
    }
  };
}

/**
 * Check if an element is visible in the viewport
 * @param {Element} element - The element to check
 * @returns {boolean}
 */
function isElementInViewport(element) {
  const rect = element.getBoundingClientRect();
  return (
    rect.top >= 0 &&
    rect.left >= 0 &&
    rect.bottom <= (window.innerHeight || document.documentElement.clientHeight) &&
    rect.right <= (window.innerWidth || document.documentElement.clientWidth)
  );
}

/**
 * Add a CSS class to an element
 * @param {Element} element - The element
 * @param {string} className - The class name
 */
function addClass(element, className) {
  element.classList.add(className);
}

/**
 * Remove a CSS class from an element
 * @param {Element} element - The element
 * @param {string} className - The class name
 */
function removeClass(element, className) {
  element.classList.remove(className);
}

/**
 * Toggle a CSS class on an element
 * @param {Element} element - The element
 * @param {string} className - The class name
 * @returns {boolean} True if the class was added, false if removed
 */
function toggleClass(element, className) {
  return element.classList.toggle(className);
}

// ============================================================================
// Initialization
// ============================================================================

// Initialize theme when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initializeTheme);
} else {
  initializeTheme();
}

// Export functions for use in other scripts
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    // Theme
    getCurrentTheme,
    setTheme,
    toggleTheme,
    initializeTheme,
    // Navigation
    navigateTo,
    getCurrentRoute,
    isRouteActive,
    buildAdminUrl,
    // API
    apiRequest,
    apiGet,
    apiPost,
    apiPut,
    apiDelete,
    apiPatch,
    // Utilities
    formatDate,
    debounce,
    throttle,
    isElementInViewport,
    addClass,
    removeClass,
    toggleClass,
  };
}
