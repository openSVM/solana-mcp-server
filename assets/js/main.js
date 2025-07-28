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
        initializeSearch();
        initializeBookmarks();
        initializeNavigation();
        initializeScrollSpy();
        initializeTableOfContents();
    }

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

        function performSearch(query) {
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
        }

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

        function toggleBookmark(url, title) {
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
        }

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
        search: performSearch,
        toggleBookmark: (url, title) => toggleBookmark(url, title),
        getBookmarks: () => JSON.parse(localStorage.getItem('solana-mcp-bookmarks') || '[]'),
        clearBookmarks: () => {
            localStorage.removeItem('solana-mcp-bookmarks');
            window.bookmarks = [];
        }
    };

})();