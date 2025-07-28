---
layout: default
title: "Bookmarks"
description: "Your bookmarked documentation pages"
---

<div class="bookmarks-page">
  <header class="page-header">
    <h1 class="page-title">📖 Your Bookmarks</h1>
    <p class="page-description">
      Quick access to your saved documentation pages.
    </p>
  </header>

  <div class="bookmarks-actions">
    <button id="clear-all-bookmarks" class="btn" style="background-color: var(--eink-white); border: 1px solid var(--eink-medium-gray);">
      Clear All Bookmarks
    </button>
    <button id="export-bookmarks" class="btn" style="background-color: var(--eink-white); border: 1px solid var(--eink-medium-gray);">
      Export Bookmarks
    </button>
  </div>

  <div id="bookmarks-container" class="bookmarks-container">
    <!-- Bookmarks will be populated here -->
  </div>

  <div id="empty-state" class="empty-state" style="display: none;">
    <div class="empty-icon">📚</div>
    <h3>No bookmarks yet</h3>
    <p>Start exploring the documentation and bookmark pages you want to reference later.</p>
    <a href="{{ '/' | relative_url }}" class="btn btn-primary">Browse Documentation</a>
  </div>
</div>

<style>
.bookmarks-page {
  max-width: 800px;
  margin: 0 auto;
}

.bookmarks-actions {
  display: flex;
  gap: 1rem;
  margin: 2rem 0;
  flex-wrap: wrap;
}

.bookmarks-actions .btn {
  font-size: var(--font-size-sm);
  padding: var(--spacing-sm) var(--spacing-md);
}

.bookmarks-container {
  margin: 2rem 0;
}

.bookmark-item {
  background-color: var(--eink-white);
  border: 1px solid var(--eink-medium-gray);
  border-radius: var(--border-radius-md);
  padding: 1.5rem;
  margin-bottom: 1rem;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  transition: all var(--transition-fast);
}

.bookmark-item:hover {
  border-color: var(--accent-teal);
  box-shadow: var(--shadow-sm);
}

.bookmark-content {
  flex: 1;
  min-width: 0;
}

.bookmark-title {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--eink-black);
  margin-bottom: 0.5rem;
  text-decoration: none;
  display: block;
}

.bookmark-title:hover {
  color: var(--accent-teal);
}

.bookmark-url {
  font-size: var(--font-size-sm);
  color: var(--accent-teal);
  margin-bottom: 0.5rem;
  font-family: var(--font-family-mono);
}

.bookmark-date {
  font-size: var(--font-size-xs);
  color: var(--eink-charcoal);
}

.bookmark-actions {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  flex-shrink: 0;
}

.bookmark-action {
  background: none;
  border: 1px solid var(--eink-medium-gray);
  color: var(--eink-charcoal);
  padding: 0.25rem 0.5rem;
  border-radius: var(--border-radius-sm);
  font-size: var(--font-size-xs);
  cursor: pointer;
  transition: all var(--transition-fast);
  min-width: 60px;
}

.bookmark-action:hover {
  background-color: var(--eink-light-gray);
  color: var(--eink-black);
}

.bookmark-action.remove:hover {
  background-color: #fee;
  border-color: #fcc;
  color: #c33;
}

.empty-state {
  text-align: center;
  padding: 4rem 2rem;
  background-color: var(--eink-white);
  border: 1px solid var(--eink-medium-gray);
  border-radius: var(--border-radius-lg);
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

.empty-state h3 {
  font-size: var(--font-size-xl);
  color: var(--eink-black);
  margin-bottom: 1rem;
}

.empty-state p {
  color: var(--eink-charcoal);
  margin-bottom: 2rem;
  max-width: 400px;
  margin-left: auto;
  margin-right: auto;
}

.bookmark-stats {
  background-color: var(--eink-light-gray);
  padding: 1rem;
  border-radius: var(--border-radius-md);
  margin-bottom: 2rem;
  text-align: center;
}

.stats-item {
  display: inline-block;
  margin: 0 1rem;
  font-size: var(--font-size-sm);
  color: var(--eink-charcoal);
}

.stats-number {
  font-weight: var(--font-weight-semibold);
  color: var(--accent-teal);
  font-size: var(--font-size-lg);
}

@media (max-width: 768px) {
  .bookmarks-page {
    margin: 0;
  }
  
  .bookmark-item {
    flex-direction: column;
    align-items: stretch;
  }
  
  .bookmark-actions {
    flex-direction: row;
    justify-content: flex-end;
  }
  
  .bookmarks-actions {
    justify-content: center;
  }
}
</style>

<script>
document.addEventListener('DOMContentLoaded', function() {
  const bookmarksContainer = document.getElementById('bookmarks-container');
  const emptyState = document.getElementById('empty-state');
  const clearAllBtn = document.getElementById('clear-all-bookmarks');
  const exportBtn = document.getElementById('export-bookmarks');
  
  function loadBookmarks() {
    const bookmarks = JSON.parse(localStorage.getItem('solana-mcp-bookmarks') || '[]');
    
    if (bookmarks.length === 0) {
      bookmarksContainer.style.display = 'none';
      emptyState.style.display = 'block';
      clearAllBtn.style.display = 'none';
      exportBtn.style.display = 'none';
      return;
    }
    
    bookmarksContainer.style.display = 'block';
    emptyState.style.display = 'none';
    clearAllBtn.style.display = 'inline-block';
    exportBtn.style.display = 'inline-block';
    
    // Sort bookmarks by date (newest first)
    bookmarks.sort((a, b) => (b.timestamp || 0) - (a.timestamp || 0));
    
    // Add stats
    const statsHtml = `
      <div class="bookmark-stats">
        <div class="stats-item">
          <div class="stats-number">${bookmarks.length}</div>
          <div>Bookmarked Pages</div>
        </div>
        <div class="stats-item">
          <div class="stats-number">${getUniqueCategories(bookmarks).length}</div>
          <div>Categories</div>
        </div>
      </div>
    `;
    
    const bookmarksHtml = bookmarks.map((bookmark, index) => {
      const date = bookmark.timestamp ? new Date(bookmark.timestamp).toLocaleDateString() : 'Unknown';
      
      return `
        <div class="bookmark-item" data-index="${index}">
          <div class="bookmark-content">
            <a href="${bookmark.url}" class="bookmark-title">${bookmark.title}</a>
            <div class="bookmark-url">${bookmark.url}</div>
            <div class="bookmark-date">Bookmarked on ${date}</div>
          </div>
          <div class="bookmark-actions">
            <button class="bookmark-action visit" onclick="window.location.href='${bookmark.url}'">Visit</button>
            <button class="bookmark-action remove" onclick="removeBookmark(${index})">Remove</button>
          </div>
        </div>
      `;
    }).join('');
    
    bookmarksContainer.innerHTML = statsHtml + bookmarksHtml;
  }
  
  function getUniqueCategories(bookmarks) {
    const categories = new Set();
    bookmarks.forEach(bookmark => {
      if (bookmark.url.includes('/docs/')) {
        const parts = bookmark.url.split('/');
        const docType = parts[parts.length - 2] || 'general';
        categories.add(docType);
      }
    });
    return Array.from(categories);
  }
  
  window.removeBookmark = function(index) {
    if (confirm('Are you sure you want to remove this bookmark?')) {
      let bookmarks = JSON.parse(localStorage.getItem('solana-mcp-bookmarks') || '[]');
      bookmarks.splice(index, 1);
      localStorage.setItem('solana-mcp-bookmarks', JSON.stringify(bookmarks));
      window.bookmarks = bookmarks;
      loadBookmarks();
      
      // Update bookmark icon on current page if it matches
      const currentUrl = window.location.pathname;
      const bookmarkToggle = document.getElementById('bookmark-toggle');
      if (bookmarkToggle && !bookmarks.some(b => b.url === currentUrl)) {
        bookmarkToggle.classList.remove('bookmarked');
        bookmarkToggle.setAttribute('aria-label', 'Bookmark this page');
      }
    }
  };
  
  clearAllBtn.addEventListener('click', function() {
    if (confirm('Are you sure you want to clear all bookmarks? This action cannot be undone.')) {
      localStorage.removeItem('solana-mcp-bookmarks');
      window.bookmarks = [];
      loadBookmarks();
      
      // Update bookmark icon on current page
      const bookmarkToggle = document.getElementById('bookmark-toggle');
      if (bookmarkToggle) {
        bookmarkToggle.classList.remove('bookmarked');
        bookmarkToggle.setAttribute('aria-label', 'Bookmark this page');
      }
    }
  });
  
  exportBtn.addEventListener('click', function() {
    const bookmarks = JSON.parse(localStorage.getItem('solana-mcp-bookmarks') || '[]');
    
    const exportData = {
      title: 'Solana MCP Server Documentation Bookmarks',
      exported: new Date().toISOString(),
      bookmarks: bookmarks
    };
    
    const dataStr = JSON.stringify(exportData, null, 2);
    const dataBlob = new Blob([dataStr], { type: 'application/json' });
    
    const link = document.createElement('a');
    link.href = URL.createObjectURL(dataBlob);
    link.download = `solana-mcp-bookmarks-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  });
  
  // Initial load
  loadBookmarks();
  
  // Listen for storage changes from other tabs
  window.addEventListener('storage', function(e) {
    if (e.key === 'solana-mcp-bookmarks') {
      loadBookmarks();
    }
  });
});
</script>