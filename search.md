---
layout: default
title: "Search Documentation"
description: "Search through all Solana MCP Server documentation"
---

<div class="search-page">
  <header class="page-header">
    <h1 class="page-title">Search Documentation</h1>
    <p class="page-description">
      Find information quickly across all documentation pages.
    </p>
  </header>

  <div class="search-form">
    <div class="search-input-wrapper">
      <input type="text" id="page-search-input" placeholder="Enter search terms..." 
             autocomplete="off" class="search-input-large">
      <button type="button" id="clear-search" class="search-clear" aria-label="Clear search">×</button>
    </div>
    <p class="search-help">
      <strong>Tips:</strong> Try searching for specific RPC methods, configuration options, or deployment platforms.
    </p>
  </div>

  <div class="search-results-container">
    <div id="page-search-results" class="search-results-list"></div>
    <div id="search-status" class="search-status">
      Enter search terms above to find relevant documentation.
    </div>
  </div>

  <div class="search-suggestions">
    <h3>Popular Searches</h3>
    <div class="suggestion-tags">
      <button class="suggestion-tag" data-query="getAccountInfo">getAccountInfo</button>
      <button class="suggestion-tag" data-query="deployment">Deployment</button>
      <button class="suggestion-tag" data-query="configuration">Configuration</button>
      <button class="suggestion-tag" data-query="docker">Docker</button>
      <button class="suggestion-tag" data-query="kubernetes">Kubernetes</button>
      <button class="suggestion-tag" data-query="tokens">Token Methods</button>
      <button class="suggestion-tag" data-query="metrics">Metrics</button>
      <button class="suggestion-tag" data-query="security">Security</button>
    </div>
  </div>
</div>

<style>
.search-page {
  max-width: 800px;
  margin: 0 auto;
}

.search-form {
  margin: 2rem 0;
}

.search-input-wrapper {
  position: relative;
  margin-bottom: 1rem;
}

.search-input-large {
  width: 100%;
  padding: 1rem 3rem 1rem 1rem;
  font-size: 1.125rem;
  border: 2px solid var(--eink-medium-gray);
  border-radius: var(--border-radius-md);
  background-color: var(--eink-white);
  color: var(--eink-black);
  font-family: inherit;
  transition: border-color var(--transition-fast);
}

.search-input-large:focus {
  outline: none;
  border-color: var(--accent-teal);
  box-shadow: 0 0 0 3px rgba(45, 95, 95, 0.1);
}

.search-clear {
  position: absolute;
  right: 1rem;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  font-size: 1.5rem;
  color: var(--eink-charcoal);
  cursor: pointer;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--border-radius-sm);
  transition: all var(--transition-fast);
}

.search-clear:hover {
  background-color: var(--eink-light-gray);
  color: var(--eink-black);
}

.search-help {
  font-size: var(--font-size-sm);
  color: var(--eink-charcoal);
  margin: 0;
}

.search-results-container {
  margin: 2rem 0;
}

.search-results-list {
  margin-bottom: 2rem;
}

.search-status {
  text-align: center;
  color: var(--eink-charcoal);
  font-style: italic;
  padding: 2rem;
  background-color: var(--eink-light-gray);
  border-radius: var(--border-radius-md);
}

.search-suggestions h3 {
  margin-bottom: 1rem;
  color: var(--eink-black);
}

.suggestion-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.suggestion-tag {
  background-color: var(--eink-white);
  border: 1px solid var(--eink-medium-gray);
  color: var(--eink-charcoal);
  padding: 0.5rem 1rem;
  border-radius: var(--border-radius-sm);
  cursor: pointer;
  font-size: var(--font-size-sm);
  transition: all var(--transition-fast);
}

.suggestion-tag:hover {
  background-color: var(--accent-teal);
  color: var(--eink-white);
  border-color: var(--accent-teal);
}

.search-result-item {
  background-color: var(--eink-white);
  border: 1px solid var(--eink-medium-gray);
  border-radius: var(--border-radius-md);
  padding: 1.5rem;
  margin-bottom: 1rem;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.search-result-item:hover {
  border-color: var(--accent-teal);
  box-shadow: var(--shadow-sm);
}

.search-result-title {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--eink-black);
  margin-bottom: 0.5rem;
}

.search-result-url {
  font-size: var(--font-size-xs);
  color: var(--accent-teal);
  margin-bottom: 0.5rem;
}

.search-result-snippet {
  color: var(--eink-charcoal);
  line-height: var(--line-height-relaxed);
}

.search-result-snippet mark {
  background-color: var(--accent-amber);
  color: var(--eink-black);
  padding: 0.1em 0.2em;
  border-radius: 2px;
}

@media (max-width: 768px) {
  .search-page {
    margin: 0;
  }
  
  .suggestion-tags {
    justify-content: center;
  }
}
</style>

<script>
document.addEventListener('DOMContentLoaded', function() {
  const searchInput = document.getElementById('page-search-input');
  const clearButton = document.getElementById('clear-search');
  const resultsContainer = document.getElementById('page-search-results');
  const statusElement = document.getElementById('search-status');
  const suggestionTags = document.querySelectorAll('.suggestion-tag');
  
  let searchIndex;
  let searchData = [];
  
  // Initialize search
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
  
  // Search input handler
  let searchTimeout;
  searchInput.addEventListener('input', function() {
    clearTimeout(searchTimeout);
    const query = this.value.trim();
    
    if (query.length === 0) {
      resultsContainer.innerHTML = '';
      statusElement.textContent = 'Enter search terms above to find relevant documentation.';
      statusElement.style.display = 'block';
      return;
    }
    
    if (query.length < 2) {
      statusElement.textContent = 'Enter at least 2 characters to search.';
      statusElement.style.display = 'block';
      resultsContainer.innerHTML = '';
      return;
    }
    
    statusElement.textContent = 'Searching...';
    statusElement.style.display = 'block';
    
    searchTimeout = setTimeout(() => {
      performSearch(query);
    }, 200);
  });
  
  // Clear button
  clearButton.addEventListener('click', function() {
    searchInput.value = '';
    resultsContainer.innerHTML = '';
    statusElement.textContent = 'Enter search terms above to find relevant documentation.';
    statusElement.style.display = 'block';
    searchInput.focus();
  });
  
  // Suggestion tags
  suggestionTags.forEach(tag => {
    tag.addEventListener('click', function() {
      const query = this.getAttribute('data-query');
      searchInput.value = query;
      performSearch(query);
    });
  });
  
  function performSearch(query) {
    if (!searchIndex) {
      statusElement.textContent = 'Search not available.';
      return;
    }
    
    try {
      const results = searchIndex.search(query);
      displayResults(results, query);
    } catch (e) {
      console.error('Search error:', e);
      statusElement.textContent = 'Search error occurred.';
    }
  }
  
  function displayResults(results, query) {
    if (results.length === 0) {
      resultsContainer.innerHTML = '';
      statusElement.textContent = `No results found for "${query}".`;
      statusElement.style.display = 'block';
      return;
    }
    
    statusElement.style.display = 'none';
    
    const html = results.slice(0, 10).map(result => {
      const doc = searchData.find(d => d.id === result.ref);
      if (!doc) return '';
      
      const title = highlightMatch(doc.title, query);
      const snippet = generateSnippet(doc.content, query);
      
      return `
        <div class="search-result-item" onclick="window.location.href='${doc.url}'">
          <div class="search-result-title">${title}</div>
          <div class="search-result-url">${doc.url}</div>
          <div class="search-result-snippet">${snippet}</div>
        </div>
      `;
    }).join('');
    
    resultsContainer.innerHTML = html;
  }
  
  function highlightMatch(text, query) {
    const regex = new RegExp(`(${escapeRegex(query)})`, 'gi');
    return text.replace(regex, '<mark>$1</mark>');
  }
  
  function generateSnippet(content, query, length = 200) {
    const words = content.toLowerCase().split(/\s+/);
    const queryWords = query.toLowerCase().split(/\s+/);
    
    let bestIndex = 0;
    let bestScore = 0;
    
    for (let i = 0; i < words.length - 20; i++) {
      let score = 0;
      for (let j = i; j < Math.min(i + 30, words.length); j++) {
        if (queryWords.some(qw => words[j].includes(qw))) {
          score += 1;
        }
      }
      if (score > bestScore) {
        bestScore = score;
        bestIndex = i;
      }
    }
    
    const snippetWords = content.split(/\s+/).slice(bestIndex, bestIndex + 30);
    let snippet = snippetWords.join(' ');
    
    if (snippet.length > length) {
      snippet = snippet.substring(0, length) + '...';
    }
    
    return highlightMatch(snippet, query);
  }
  
  function escapeRegex(string) {
    return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }
});
</script>