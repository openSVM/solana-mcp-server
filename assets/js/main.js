/**
 * Solana MCP Server Documentation Site
 * JavaScript for search, bookmarks, and navigation
 */

(function() {
    'use strict';

    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    function init() {
        initializeTheme();
        initializeSearch();
        initializeBookmarks();
        initializeNavigation();
        initializeScrollSpy();
        initializeTableOfContents();
        initializeMicroAnimations();
        initializeLLMsButton();
    }

    // Theme functionality
    function initializeTheme() {
        const themeToggle = document.getElementById('theme-toggle');
        const themeDropdown = document.getElementById('theme-dropdown');
        const themeOptions = document.querySelectorAll('.theme-option');
        
        if (!themeToggle || !themeDropdown) return;
        
        // Check if localStorage is available
        const storageAvailable = typeof(Storage) !== 'undefined';
        
        // Load saved theme or default to light
        let savedTheme = 'light';
        if (storageAvailable) {
            try {
                savedTheme = localStorage.getItem('solana-mcp-theme') || 'light';
            } catch (e) {
                console.warn('localStorage not accessible, using default theme');
            }
        }
        applyTheme(savedTheme);
        
        // Toggle dropdown
        themeToggle.addEventListener('click', function(e) {
            e.stopPropagation();
            themeDropdown.classList.toggle('active');
        });
        
        // Close dropdown when clicking outside
        document.addEventListener('click', function(e) {
            if (!themeToggle.contains(e.target) && !themeDropdown.contains(e.target)) {
                themeDropdown.classList.remove('active');
            }
        });
        
        // Theme selection
        themeOptions.forEach(option => {
            option.addEventListener('click', function() {
                const theme = this.getAttribute('data-theme');
                applyTheme(theme);
                if (storageAvailable) {
                    try {
                        localStorage.setItem('solana-mcp-theme', theme);
                    } catch (e) {
                        console.warn('Cannot save theme preference');
                    }
                }
                themeDropdown.classList.remove('active');
            });
        });
        
        function applyTheme(theme) {
            document.documentElement.setAttribute('data-theme', theme);
            
            // Update active state in dropdown
            themeOptions.forEach(opt => {
                if (opt.getAttribute('data-theme') === theme) {
                    opt.classList.add('active');
                } else {
                    opt.classList.remove('active');
                }
            });
        }
    }

    // Global functions that need to be accessible outside their initialization scopes
    let performSearch = null;
    let toggleBookmark = null;

    // Search functionality
    function initializeSearch() {
        const searchToggle = document.getElementById('search-toggle');
        const searchOverlay = document.getElementById('search-overlay');
        const searchInput = document.getElementById('search-input');
        const searchClose = document.getElementById('search-close');
        const searchResults = document.getElementById('search-results');

        if (!searchToggle || !searchOverlay || !searchInput) return;

        let searchIndex;
        let searchData = [];

        // Initialize Lunr search index
        if (window.searchData) {
            searchData = window.searchData;
            searchIndex = lunr(function() {
                this.ref('id');
                this.field('title', { boost: 10 });
                this.field('content');

                searchData.forEach((doc) => {
                    this.add(doc);
                });
            });
        }

        // Toggle search overlay
        searchToggle.addEventListener('click', () => {
            searchOverlay.classList.add('active');
            searchInput.focus();
            searchToggle.setAttribute('aria-expanded', 'true');
        });

        // Close search overlay
        function closeSearch() {
            searchOverlay.classList.remove('active');
            searchInput.value = '';
            searchResults.innerHTML = '';
            searchToggle.setAttribute('aria-expanded', 'false');
        }

        searchClose.addEventListener('click', closeSearch);

        // Close on escape key
        searchOverlay.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                closeSearch();
            }
        });

        // Close on overlay click
        searchOverlay.addEventListener('click', (e) => {
            if (e.target === searchOverlay) {
                closeSearch();
            }
        });

        // Search input handler
        let searchTimeout;
        searchInput.addEventListener('input', (e) => {
            clearTimeout(searchTimeout);
            const query = e.target.value.trim();

            if (query.length < 2) {
                searchResults.innerHTML = '';
                return;
            }

            searchTimeout = setTimeout(() => {
                performSearch(query);
            }, 150);
        });

        // Define performSearch and assign to global variable
        performSearch = function(query) {
            if (!searchIndex) {
                searchResults.innerHTML = '<div class="search-result"><div class="search-result-title">Search not available</div></div>';
                return;
            }

            try {
                const results = searchIndex.search(query);
                displaySearchResults(results, query);
            } catch (e) {
                console.error('Search error:', e);
                searchResults.innerHTML = '<div class="search-result"><div class="search-result-title">Search error occurred</div></div>';
            }
        };

        function displaySearchResults(results, query) {
            if (results.length === 0) {
                searchResults.innerHTML = '<div class="search-result"><div class="search-result-title">No results found</div></div>';
                return;
            }

            const html = results.slice(0, 8).map(result => {
                const doc = searchData.find(d => d.id === result.ref);
                if (!doc) return '';

                const title = highlightMatch(doc.title, query);
                const snippet = generateSnippet(doc.content, query);

                return `
                    <div class="search-result" onclick="navigateToResult('${doc.url}')">
                        <div class="search-result-title">${title}</div>
                        <div class="search-result-snippet">${snippet}</div>
                    </div>
                `;
            }).join('');

            searchResults.innerHTML = html;
        }

        function highlightMatch(text, query) {
            const regex = new RegExp(`(${escapeRegex(query)})`, 'gi');
            return text.replace(regex, '<mark>$1</mark>');
        }

        function generateSnippet(content, query, length = 150) {
            const words = content.toLowerCase().split(/\s+/);
            const queryWords = query.toLowerCase().split(/\s+/);
            
            let bestIndex = 0;
            let bestScore = 0;

            // Find the best position to show snippet
            for (let i = 0; i < words.length - 10; i++) {
                let score = 0;
                for (let j = i; j < Math.min(i + 20, words.length); j++) {
                    if (queryWords.some(qw => words[j].includes(qw))) {
                        score += 1;
                    }
                }
                if (score > bestScore) {
                    bestScore = score;
                    bestIndex = i;
                }
            }

            const snippetWords = content.split(/\s+/).slice(bestIndex, bestIndex + 25);
            let snippet = snippetWords.join(' ');
            
            if (snippet.length > length) {
                snippet = snippet.substring(0, length) + '...';
            }
            
            return highlightMatch(snippet, query);
        }

        function escapeRegex(string) {
            return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        }

        // Make navigateToResult globally available
        window.navigateToResult = function(url) {
            window.location.href = url;
        };
    }

    // Bookmark functionality
    function initializeBookmarks() {
        const bookmarkToggle = document.getElementById('bookmark-toggle');
        if (!bookmarkToggle) return;

        const currentUrl = window.location.pathname;
        const currentTitle = bookmarkToggle.getAttribute('data-title');

        // Check if current page is bookmarked
        updateBookmarkIcon();

        bookmarkToggle.addEventListener('click', () => {
            toggleBookmark(currentUrl, currentTitle);
        });

        // Define toggleBookmark and assign to global variable
        toggleBookmark = function(url, title) {
            let bookmarks = JSON.parse(localStorage.getItem('solana-mcp-bookmarks') || '[]');
            const existingIndex = bookmarks.findIndex(b => b.url === url);

            if (existingIndex > -1) {
                // Remove bookmark
                bookmarks.splice(existingIndex, 1);
                bookmarkToggle.classList.remove('bookmarked');
                bookmarkToggle.setAttribute('aria-label', 'Bookmark this page');
            } else {
                // Add bookmark
                bookmarks.push({
                    url: url,
                    title: title,
                    timestamp: Date.now()
                });
                bookmarkToggle.classList.add('bookmarked');
                bookmarkToggle.setAttribute('aria-label', 'Remove bookmark');
            }

            localStorage.setItem('solana-mcp-bookmarks', JSON.stringify(bookmarks));
            window.bookmarks = bookmarks;
        };

        function updateBookmarkIcon() {
            const bookmarks = JSON.parse(localStorage.getItem('solana-mcp-bookmarks') || '[]');
            const isBookmarked = bookmarks.some(b => b.url === currentUrl);
            
            if (isBookmarked) {
                bookmarkToggle.classList.add('bookmarked');
                bookmarkToggle.setAttribute('aria-label', 'Remove bookmark');
            } else {
                bookmarkToggle.classList.remove('bookmarked');
                bookmarkToggle.setAttribute('aria-label', 'Bookmark this page');
            }
        }
    }

    // Navigation functionality
    function initializeNavigation() {
        const menuToggle = document.getElementById('menu-toggle');
        const mainNav = document.querySelector('.main-nav');
        
        if (!menuToggle || !mainNav) return;

        menuToggle.addEventListener('click', () => {
            const isExpanded = menuToggle.getAttribute('aria-expanded') === 'true';
            
            menuToggle.setAttribute('aria-expanded', !isExpanded);
            mainNav.classList.toggle('active');
        });

        // Close menu when clicking outside
        document.addEventListener('click', (e) => {
            if (!menuToggle.contains(e.target) && !mainNav.contains(e.target)) {
                menuToggle.setAttribute('aria-expanded', 'false');
                mainNav.classList.remove('active');
            }
        });

        // Close menu on escape key
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                menuToggle.setAttribute('aria-expanded', 'false');
                mainNav.classList.remove('active');
            }
        });

        // Handle responsive navigation
        window.addEventListener('resize', () => {
            if (window.innerWidth > 768) {
                menuToggle.setAttribute('aria-expanded', 'false');
                mainNav.classList.remove('active');
            }
        });
    }

    // Scroll spy for documentation navigation
    function initializeScrollSpy() {
        const headings = document.querySelectorAll('.content h1, .content h2, .content h3');
        const navLinks = document.querySelectorAll('.docs-nav-link');
        
        if (headings.length === 0 || navLinks.length === 0) return;

        const observer = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    const id = entry.target.id;
                    if (id) {
                        // Update active navigation link
                        navLinks.forEach(link => {
                            link.classList.remove('active');
                            if (link.getAttribute('href').includes(id)) {
                                link.classList.add('active');
                            }
                        });
                    }
                }
            });
        }, {
            rootMargin: '-20% 0px -70% 0px'
        });

        headings.forEach(heading => {
            if (heading.id) {
                observer.observe(heading);
            }
        });
    }

    // Generate table of contents for long documents
    function initializeTableOfContents() {
        const content = document.querySelector('.content');
        if (!content) return;

        const headings = content.querySelectorAll('h2, h3, h4');
        if (headings.length < 3) return; // Only show TOC for documents with multiple headings

        const toc = document.createElement('div');
        toc.className = 'table-of-contents';
        toc.innerHTML = '<h3>Table of Contents</h3>';

        const tocNav = document.createElement('nav');
        tocNav.setAttribute('aria-label', 'Table of contents');
        
        const tocList = document.createElement('ul');
        tocList.className = 'toc-list';

        headings.forEach((heading, index) => {
            // Generate ID if it doesn't exist
            if (!heading.id) {
                heading.id = `heading-${index}`;
            }

            const li = document.createElement('li');
            li.className = `toc-item toc-${heading.tagName.toLowerCase()}`;
            
            const link = document.createElement('a');
            link.href = `#${heading.id}`;
            link.textContent = heading.textContent;
            link.className = 'toc-link';
            
            // Smooth scroll to heading
            link.addEventListener('click', (e) => {
                e.preventDefault();
                heading.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
                
                // Update URL without triggering page reload
                history.pushState(null, null, `#${heading.id}`);
            });
            
            li.appendChild(link);
            tocList.appendChild(li);
        });

        tocNav.appendChild(tocList);
        toc.appendChild(tocNav);

        // Insert TOC after the first paragraph or at the beginning
        const firstParagraph = content.querySelector('p');
        if (firstParagraph) {
            firstParagraph.parentNode.insertBefore(toc, firstParagraph.nextSibling);
        } else {
            content.insertBefore(toc, content.firstChild);
        }
    }

    // Utility functions
    function debounce(func, wait) {
        let timeout;
        return function executedFunction(...args) {
            const later = () => {
                clearTimeout(timeout);
                func(...args);
            };
            clearTimeout(timeout);
            timeout = setTimeout(later, wait);
        };
    }

    function throttle(func, limit) {
        let inThrottle;
        return function() {
            const args = arguments;
            const context = this;
            if (!inThrottle) {
                func.apply(context, args);
                inThrottle = true;
                setTimeout(() => inThrottle = false, limit);
            }
        };
    }

    // Copy code functionality
    function initializeCodeCopy() {
        const codeBlocks = document.querySelectorAll('pre code');
        
        codeBlocks.forEach(block => {
            const button = document.createElement('button');
            button.className = 'copy-code-btn';
            button.textContent = 'Copy';
            button.setAttribute('aria-label', 'Copy code to clipboard');
            
            button.addEventListener('click', async () => {
                try {
                    await navigator.clipboard.writeText(block.textContent);
                    button.textContent = 'Copied!';
                    button.classList.add('copied');
                    
                    setTimeout(() => {
                        button.textContent = 'Copy';
                        button.classList.remove('copied');
                    }, 2000);
                } catch (err) {
                    console.error('Failed to copy code:', err);
                    button.textContent = 'Failed';
                    setTimeout(() => {
                        button.textContent = 'Copy';
                    }, 2000);
                }
            });
            
            const wrapper = document.createElement('div');
            wrapper.className = 'code-block-wrapper';
            block.parentNode.insertBefore(wrapper, block);
            wrapper.appendChild(block.parentNode);
            wrapper.appendChild(button);
        });
    }

    // Initialize code copy when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initializeCodeCopy);
    } else {
        initializeCodeCopy();
    }

    // Lazy loading for images
    function initializeLazyLoading() {
        const images = document.querySelectorAll('img[data-src]');
        
        if ('IntersectionObserver' in window) {
            const imageObserver = new IntersectionObserver((entries) => {
                entries.forEach(entry => {
                    if (entry.isIntersecting) {
                        const img = entry.target;
                        img.src = img.dataset.src;
                        img.removeAttribute('data-src');
                        img.classList.remove('lazy');
                        imageObserver.unobserve(img);
                    }
                });
            });
            
            images.forEach(img => imageObserver.observe(img));
        } else {
            // Fallback for browsers without IntersectionObserver
            images.forEach(img => {
                img.src = img.dataset.src;
                img.removeAttribute('data-src');
            });
        }
    }

    // Theme preference handling
    function initializeThemeToggle() {
        const themeToggle = document.getElementById('theme-toggle');
        if (!themeToggle) return;

        const preferredTheme = localStorage.getItem('preferred-theme') || 'eink';
        document.body.className = `${preferredTheme}-theme`;

        themeToggle.addEventListener('click', () => {
            const currentTheme = document.body.className.includes('dark') ? 'dark' : 'eink';
            const newTheme = currentTheme === 'eink' ? 'dark' : 'eink';
            
            document.body.className = `${newTheme}-theme`;
            localStorage.setItem('preferred-theme', newTheme);
            
            themeToggle.setAttribute('aria-label', 
                newTheme === 'eink' ? 'Switch to dark theme' : 'Switch to e-ink theme'
            );
        });
    }

    // Analytics and performance monitoring
    function initializeAnalytics() {
        // Track page views
        if (typeof gtag !== 'undefined') {
            gtag('config', 'GA_MEASUREMENT_ID', {
                page_title: document.title,
                page_location: window.location.href
            });
        }

        // Track search usage
        const searchInput = document.getElementById('search-input');
        if (searchInput) {
            let searchStartTime;
            
            searchInput.addEventListener('focus', () => {
                searchStartTime = Date.now();
            });
            
            searchInput.addEventListener('blur', () => {
                if (searchStartTime) {
                    const searchDuration = Date.now() - searchStartTime;
                    // Track search engagement time
                    if (typeof gtag !== 'undefined') {
                        gtag('event', 'search_engagement', {
                            event_category: 'Search',
                            event_label: 'Duration',
                            value: Math.round(searchDuration / 1000)
                        });
                    }
                }
            });
        }

        // Track bookmark usage
        const bookmarkToggle = document.getElementById('bookmark-toggle');
        if (bookmarkToggle) {
            bookmarkToggle.addEventListener('click', () => {
                const isBookmarked = bookmarkToggle.classList.contains('bookmarked');
                if (typeof gtag !== 'undefined') {
                    gtag('event', isBookmarked ? 'bookmark_remove' : 'bookmark_add', {
                        event_category: 'Engagement',
                        event_label: window.location.pathname
                    });
                }
            });
        }
    }

    // Initialize additional features
    document.addEventListener('DOMContentLoaded', () => {
        initializeLazyLoading();
        initializeThemeToggle();
        initializeAnalytics();
    });

    // Service Worker registration for offline functionality
    if ('serviceWorker' in navigator) {
        window.addEventListener('load', () => {
            navigator.serviceWorker.register('/sw.js')
                .then(registration => {
                    console.log('SW registered: ', registration);
                })
                .catch(registrationError => {
                    console.log('SW registration failed: ', registrationError);
                });
        });
    }

    // Export for global access
    window.SolanaMCPDocs = {
        search: (query) => performSearch && performSearch(query),
        toggleBookmark: (url, title) => toggleBookmark && toggleBookmark(url, title),
        getBookmarks: () => JSON.parse(localStorage.getItem('solana-mcp-bookmarks') || '[]'),
        clearBookmarks: () => {
            localStorage.removeItem('solana-mcp-bookmarks');
            window.bookmarks = [];
        }
    };

    // Also expose performSearch globally for backward compatibility
    window.performSearch = (query) => performSearch && performSearch(query);

    // Micro-animations and interactive enhancements
    function initializeMicroAnimations() {
        // Enhanced button interactions
        const buttons = document.querySelectorAll('.btn, .search-toggle, .bookmark-toggle, .menu-toggle');
        buttons.forEach(button => {
            // Add ripple effect on click
            button.addEventListener('click', function(e) {
                const ripple = document.createElement('span');
                const rect = this.getBoundingClientRect();
                const size = Math.max(rect.width, rect.height);
                const x = e.clientX - rect.left - size / 2;
                const y = e.clientY - rect.top - size / 2;
                
                ripple.style.cssText = `
                    position: absolute;
                    border-radius: 50%;
                    background: rgba(255, 255, 255, 0.3);
                    transform: scale(0);
                    animation: ripple 0.6s linear;
                    width: ${size}px;
                    height: ${size}px;
                    left: ${x}px;
                    top: ${y}px;
                    pointer-events: none;
                `;
                
                // Ensure button has relative positioning for ripple
                const computedStyle = window.getComputedStyle(this);
                if (computedStyle.position === 'static') {
                    this.style.position = 'relative';
                }
                
                this.appendChild(ripple);
                
                // Remove ripple after animation
                setTimeout(() => {
                    if (ripple.parentNode) {
                        ripple.parentNode.removeChild(ripple);
                    }
                }, 600);
            });
            
            // Add loading state for primary buttons
            if (button.classList.contains('btn-primary')) {
                button.addEventListener('click', function() {
                    if (this.dataset.loading === 'true') return;
                    
                    const originalText = this.textContent;
                    this.dataset.loading = 'true';
                    this.textContent = 'Loading...';
                    this.classList.add('loading');
                    
                    // Reset after 2 seconds (simulating async operation)
                    setTimeout(() => {
                        this.dataset.loading = 'false';
                        this.textContent = originalText;
                        this.classList.remove('loading');
                    }, 2000);
                });
            }
        });

        // Enhanced search input interactions
        const searchInput = document.getElementById('search-input');
        if (searchInput) {
            let isSearching = false;
            
            searchInput.addEventListener('input', function() {
                if (!isSearching && this.value.length > 0) {
                    isSearching = true;
                    this.classList.add('searching');
                }
            });
            
            // Clear searching state when no results or empty
            const originalSearch = window.performSearch;
            if (originalSearch) {
                window.performSearch = function(query) {
                    originalSearch(query);
                    if (searchInput) {
                        searchInput.classList.remove('searching');
                        isSearching = false;
                    }
                };
            }
        }

        // Enhanced card hover effects with stagger animation
        const cards = document.querySelectorAll('.feature-card, .docs-card');
        const observerOptions = {
            threshold: 0.1,
            rootMargin: '0px 0px -50px 0px'
        };

        const cardObserver = new IntersectionObserver((entries) => {
            entries.forEach((entry, index) => {
                if (entry.isIntersecting) {
                    // Stagger the animation
                    setTimeout(() => {
                        entry.target.classList.add('fade-in');
                        entry.target.style.animationDelay = `${index * 0.1}s`;
                    }, index * 50);
                    cardObserver.unobserve(entry.target);
                }
            });
        }, observerOptions);

        cards.forEach((card, index) => {
            // Initial state
            card.style.opacity = '0';
            card.style.transform = 'translateY(20px)';
            
            cardObserver.observe(card);
            
            // Add magnetic effect on mouse move
            card.addEventListener('mousemove', function(e) {
                const rect = this.getBoundingClientRect();
                const x = e.clientX - rect.left - rect.width / 2;
                const y = e.clientY - rect.top - rect.height / 2;
                
                const moveX = x * 0.1;
                const moveY = y * 0.1;
                
                this.style.transform = `translateY(-8px) rotateX(${-moveY}deg) rotateY(${moveX}deg)`;
            });
            
            card.addEventListener('mouseleave', function() {
                this.style.transform = 'translateY(0) rotateX(0) rotateY(0)';
            });
        });

        // Enhanced navigation link interactions
        const navLinks = document.querySelectorAll('.nav-link, .docs-nav-link');
        navLinks.forEach(link => {
            link.addEventListener('mouseenter', function() {
                // Add pulse effect to active indicator
                if (this.classList.contains('active')) {
                    const indicator = this.querySelector('::after');
                    this.style.setProperty('--pulse-animation', 'pulse 0.5s ease');
                }
            });
        });

        // Smooth scroll for all internal links
        const internalLinks = document.querySelectorAll('a[href^="#"], a[href^="/"]');
        internalLinks.forEach(link => {
            if (link.hostname === window.location.hostname) {
                link.addEventListener('click', function(e) {
                    const href = this.getAttribute('href');
                    if (href.startsWith('#')) {
                        e.preventDefault();
                        const target = document.querySelector(href);
                        if (target) {
                            target.scrollIntoView({
                                behavior: 'smooth',
                                block: 'start'
                            });
                            
                            // Add highlight effect to target
                            target.classList.add('highlight-target');
                            setTimeout(() => {
                                target.classList.remove('highlight-target');
                            }, 2000);
                        }
                    }
                });
            }
        });

        // Add progressive enhancement for form elements
        const inputs = document.querySelectorAll('input, textarea, select');
        inputs.forEach(input => {
            // Floating label effect
            const label = input.previousElementSibling;
            if (label && label.tagName === 'LABEL') {
                input.addEventListener('focus', () => {
                    label.classList.add('focused');
                });
                
                input.addEventListener('blur', () => {
                    if (!input.value) {
                        label.classList.remove('focused');
                    }
                });
                
                // Check initial state
                if (input.value) {
                    label.classList.add('focused');
                }
            }
            
            // Add validation feedback
            input.addEventListener('invalid', function() {
                this.classList.add('error');
                this.style.animation = 'shake 0.5s ease-in-out';
            });
            
            input.addEventListener('input', function() {
                this.classList.remove('error');
                this.style.animation = '';
            });
        });

        // Enhanced scroll indicators
        const scrollIndicator = document.createElement('div');
        scrollIndicator.className = 'scroll-indicator';
        scrollIndicator.innerHTML = '<div class="scroll-progress"></div>';
        document.body.appendChild(scrollIndicator);
        
        const scrollProgress = scrollIndicator.querySelector('.scroll-progress');
        
        window.addEventListener('scroll', throttle(() => {
            const winScroll = document.body.scrollTop || document.documentElement.scrollTop;
            const height = document.documentElement.scrollHeight - document.documentElement.clientHeight;
            const scrolled = (winScroll / height) * 100;
            
            scrollProgress.style.width = scrolled + '%';
            
            // Add scroll-based animations
            if (winScroll > 100) {
                document.body.classList.add('scrolled');
            } else {
                document.body.classList.remove('scrolled');
            }
        }, 10));

        // Add CSS for animations
        const animationStyles = document.createElement('style');
        animationStyles.textContent = `
            @keyframes ripple {
                to {
                    transform: scale(4);
                    opacity: 0;
                }
            }
            
            @keyframes fadeInUp {
                from {
                    opacity: 0;
                    transform: translateY(30px);
                }
                to {
                    opacity: 1;
                    transform: translateY(0);
                }
            }
            
            .fade-in {
                animation: fadeInUp 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards;
            }
            
            .searching {
                position: relative;
            }
            
            .searching::after {
                content: '';
                position: absolute;
                right: 10px;
                top: 50%;
                transform: translateY(-50%);
                width: 16px;
                height: 16px;
                border: 2px solid var(--eink-light-gray);
                border-top: 2px solid var(--accent-teal);
                border-radius: 50%;
                animation: spinOnce 1s linear infinite;
            }
            
            .highlight-target {
                background-color: rgba(45, 95, 95, 0.1);
                transition: background-color 0.3s ease;
                border-radius: 4px;
                padding: 0.5rem;
                margin: -0.5rem;
            }
            
            .scroll-indicator {
                position: fixed;
                top: 0;
                left: 0;
                width: 100%;
                height: 3px;
                background-color: var(--eink-light-gray);
                z-index: 1000;
                opacity: 0;
                transition: opacity 0.3s ease;
            }
            
            .scrolled .scroll-indicator {
                opacity: 1;
            }
            
            .scroll-progress {
                height: 100%;
                background: linear-gradient(90deg, var(--accent-teal), var(--accent-teal-light));
                width: 0%;
                transition: width 0.1s ease;
            }
            
            .floating-label {
                position: relative;
            }
            
            .floating-label label {
                position: absolute;
                top: 50%;
                left: 10px;
                transform: translateY(-50%);
                transition: all 0.3s ease;
                pointer-events: none;
                color: var(--eink-charcoal);
            }
            
            .floating-label label.focused {
                top: -10px;
                font-size: 0.75rem;
                color: var(--accent-teal);
                background: var(--eink-white);
                padding: 0 5px;
            }
            
            .error {
                border-color: #e74c3c !important;
                box-shadow: 0 0 0 2px rgba(231, 76, 60, 0.2) !important;
            }
        `;
        document.head.appendChild(animationStyles);
    }

    // LLMs.txt floating button functionality
    function initializeLLMsButton() {
        const llmsButton = document.getElementById('llms-floating-btn');
        if (!llmsButton) return;
        
        let llmsContent = null;
        
        // Fetch the llms.txt content
        async function fetchLLMsContent() {
            if (llmsContent) return llmsContent;
            
            try {
                const response = await fetch('/llms.txt');
                if (response.ok) {
                    llmsContent = await response.text();
                    return llmsContent;
                } else {
                    throw new Error('Failed to fetch llms.txt');
                }
            } catch (error) {
                console.error('Error fetching llms.txt:', error);
                return 'LLMs.txt content not available';
            }
        }
        
        // Copy to clipboard functionality
        async function copyToClipboard(text) {
            try {
                if (navigator.clipboard && window.isSecureContext) {
                    await navigator.clipboard.writeText(text);
                    return true;
                } else {
                    // Fallback for older browsers or non-secure contexts
                    const textArea = document.createElement('textarea');
                    textArea.value = text;
                    textArea.style.position = 'fixed';
                    textArea.style.left = '-999999px';
                    textArea.style.top = '-999999px';
                    document.body.appendChild(textArea);
                    textArea.focus();
                    textArea.select();
                    
                    const success = document.execCommand('copy');
                    document.body.removeChild(textArea);
                    return success;
                }
            } catch (err) {
                console.error('Failed to copy text: ', err);
                return false;
            }
        }
        
        // Button click handler
        llmsButton.addEventListener('click', async function() {
            // Prevent multiple clicks
            if (this.classList.contains('copying') || this.classList.contains('copied')) {
                return;
            }
            
            // Show copying state
            this.classList.add('copying');
            const originalText = this.querySelector('.llms-text').textContent;
            this.querySelector('.llms-text').textContent = 'Copying...';
            
            try {
                // Fetch and copy content
                const content = await fetchLLMsContent();
                const success = await copyToClipboard(content);
                
                // Update button state
                this.classList.remove('copying');
                
                if (success) {
                    this.classList.add('copied');
                    this.querySelector('.llms-text').textContent = 'Copied!';
                    
                    // Show success toast
                    showToast('LLMs.txt content copied to clipboard!', 'success');
                    
                    // Track the action
                    if (typeof gtag !== 'undefined') {
                        gtag('event', 'llms_copy', {
                            event_category: 'Engagement',
                            event_label: 'llms.txt',
                            value: content.length
                        });
                    }
                } else {
                    this.querySelector('.llms-text').textContent = 'Failed';
                    showToast('Failed to copy content. Please try again.', 'error');
                }
                
                // Reset button after 2 seconds
                setTimeout(() => {
                    this.classList.remove('copied');
                    this.querySelector('.llms-text').textContent = originalText;
                }, 2000);
                
            } catch (error) {
                console.error('Error copying LLMs content:', error);
                this.classList.remove('copying');
                this.querySelector('.llms-text').textContent = 'Error';
                showToast('Error occurred while copying. Please try again.', 'error');
                
                setTimeout(() => {
                    this.querySelector('.llms-text').textContent = originalText;
                }, 2000);
            }
        });
        
        // Add hover effects
        llmsButton.addEventListener('mouseenter', function() {
            if (!this.classList.contains('copying') && !this.classList.contains('copied')) {
                this.style.transform = 'translateY(-2px) scale(1.05)';
            }
        });
        
        llmsButton.addEventListener('mouseleave', function() {
            if (!this.classList.contains('copying') && !this.classList.contains('copied')) {
                this.style.transform = 'translateY(0) scale(1)';
            }
        });
        
        // Keyboard accessibility
        llmsButton.addEventListener('keydown', function(e) {
            if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                this.click();
            }
        });
    }
    
    // Toast notification function
    function showToast(message, type = 'info') {
        // Remove existing toast if any
        const existingToast = document.querySelector('.toast-notification');
        if (existingToast) {
            existingToast.remove();
        }
        
        // Create toast element
        const toast = document.createElement('div');
        toast.className = `toast-notification toast-${type}`;
        toast.innerHTML = `
            <div class="toast-content">
                <span class="toast-icon">${type === 'success' ? '✓' : type === 'error' ? '✗' : 'ℹ'}</span>
                <span class="toast-message">${message}</span>
            </div>
        `;
        
        // Add toast styles
        const toastStyles = `
            .toast-notification {
                position: fixed;
                top: 20px;
                right: 20px;
                background: var(--eink-white);
                border: 1px solid var(--eink-medium-gray);
                border-radius: var(--border-radius-md);
                padding: var(--spacing-md);
                box-shadow: var(--shadow-lg);
                z-index: 10000;
                opacity: 0;
                transform: translateX(100%);
                transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
                max-width: 300px;
                font-family: var(--font-family-sans);
            }
            
            .toast-notification.show {
                opacity: 1;
                transform: translateX(0);
            }
            
            .toast-success {
                border-left: 4px solid #22c55e;
            }
            
            .toast-error {
                border-left: 4px solid #ef4444;
            }
            
            .toast-info {
                border-left: 4px solid var(--accent-teal);
            }
            
            .toast-content {
                display: flex;
                align-items: center;
                gap: var(--spacing-sm);
            }
            
            .toast-icon {
                font-weight: bold;
                font-size: var(--font-size-lg);
            }
            
            .toast-success .toast-icon {
                color: #22c55e;
            }
            
            .toast-error .toast-icon {
                color: #ef4444;
            }
            
            .toast-info .toast-icon {
                color: var(--accent-teal);
            }
            
            .toast-message {
                font-size: var(--font-size-sm);
                color: var(--eink-dark-charcoal);
                line-height: var(--line-height-normal);
            }
            
            @media (max-width: 768px) {
                .toast-notification {
                    top: 10px;
                    right: 10px;
                    left: 10px;
                    max-width: none;
                }
            }
        `;
        
        // Add styles if not already added
        if (!document.querySelector('#toast-styles')) {
            const styleElement = document.createElement('style');
            styleElement.id = 'toast-styles';
            styleElement.textContent = toastStyles;
            document.head.appendChild(styleElement);
        }
        
        // Add to document
        document.body.appendChild(toast);
        
        // Trigger animation
        requestAnimationFrame(() => {
            toast.classList.add('show');
        });
        
        // Auto remove after 3 seconds
        setTimeout(() => {
            toast.classList.remove('show');
            setTimeout(() => {
                if (toast.parentNode) {
                    toast.parentNode.removeChild(toast);
                }
            }, 300);
        }, 3000);
        
        // Allow manual dismiss by clicking
        toast.addEventListener('click', () => {
            toast.classList.remove('show');
            setTimeout(() => {
                if (toast.parentNode) {
                    toast.parentNode.removeChild(toast);
                }
            }, 300);
        });
    }

})();