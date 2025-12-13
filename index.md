---
layout: home
title: "Solana MCP Server Documentation"
description: "A comprehensive Model Context Protocol server for seamless Solana blockchain integration with AI assistants like Claude"
---

<div class="hero-section">
  <h1 class="hero-title">Solana MCP Server</h1>
  <p class="hero-subtitle">
    A comprehensive Model Context Protocol server that provides seamless access to Solana blockchain data through AI assistants like Claude. Query blockchain information with natural language.
  </p>
  <div class="hero-actions">
    <a href="{{ '/docs/onboarding/' | relative_url }}" class="btn btn-primary">Get Started</a>
    <a href="{{ '/docs/api-reference/' | relative_url }}" class="btn">API Reference</a>
    <a href="{{ site.github.repository_url }}" class="btn" target="_blank" rel="noopener noreferrer">GitHub</a>
  </div>
</div>

<div class="features-grid">
  <div class="feature-card">
    <span class="feature-icon">🚀</span>
    <h3 class="feature-title">Comprehensive RPC Coverage</h3>
    <p class="feature-description">
      73+ Solana RPC methods across all major categories including accounts, blocks, transactions, tokens, and system operations. Real-time blockchain data with configurable commitment levels plus full WebSocket subscription support.
    </p>
  </div>
  
  <div class="feature-card">
    <span class="feature-icon">🌐</span>
    <h3 class="feature-title">Multi-Network Support</h3>
    <p class="feature-description">
      Query multiple SVM-compatible networks simultaneously. Dynamic network configuration and management with parallel execution and result aggregation.
    </p>
  </div>
  
  <div class="feature-card">
    <span class="feature-icon">🛠️</span>
    <h3 class="feature-title">Flexible Deployment</h3>
    <p class="feature-description">
      Deploy locally with Claude Desktop, as HTTP endpoints, serverless functions (AWS Lambda, Vercel, Google Cloud), or container orchestration (Docker, Kubernetes).
    </p>
  </div>
  
  <div class="feature-card">
    <span class="feature-icon">⚡</span>
    <h3 class="feature-title">High Performance</h3>
    <p class="feature-description">
      Connection pooling, persistent RPC clients, configurable caching layers, parallel multi-network queries, and built-in rate limiting with error recovery.
    </p>
  </div>
  
  <div class="feature-card">
    <span class="feature-icon">📊</span>
    <h3 class="feature-title">Monitoring & Scaling</h3>
    <p class="feature-description">
      Prometheus metrics, Kubernetes HPA support, health checks, and comprehensive logging for production-ready deployments with autoscaling capabilities.
    </p>
  </div>
  
  <div class="feature-card">
    <span class="feature-icon">🔒</span>
    <h3 class="feature-title">Security First</h3>
    <p class="feature-description">
      Regular security audits with cargo audit, dependency updates, vulnerability monitoring, and comprehensive security documentation.
    </p>
  </div>
</div>

<section class="docs-section">
  <h2>📚 Documentation</h2>
  
  <div class="docs-grid">
    <a href="{{ '/docs/onboarding/' | relative_url }}" class="docs-card">
      <span class="docs-card-icon">🚀</span>
      <h3 class="docs-card-title">Quick Start</h3>
      <p class="docs-card-description">
        Get up and running quickly with step-by-step setup instructions, configuration, and your first queries.
      </p>
    </a>
    
    <a href="{{ '/docs/architecture/' | relative_url }}" class="docs-card">
      <span class="docs-card-icon">🏗️</span>
      <h3 class="docs-card-title">Architecture</h3>
      <p class="docs-card-description">
        Understand the system design, component interactions, and internal workings of the MCP server.
      </p>
    </a>
    
    <a href="{{ '/docs/api-reference/' | relative_url }}" class="docs-card">
      <span class="docs-card-icon">📖</span>
      <h3 class="docs-card-title">API Reference</h3>
      <p class="docs-card-description">
        Complete documentation of all 73+ RPC methods with parameters, examples, and response formats including JSON-RPC API support.
      </p>
    </a>
    
    <a href="{{ '/docs/deployment/' | relative_url }}" class="docs-card">
      <span class="docs-card-icon">🚀</span>
      <h3 class="docs-card-title">Deployment</h3>
      <p class="docs-card-description">
        Deploy locally, to cloud platforms, serverless functions, or container orchestration systems.
      </p>
    </a>
    
    <a href="{{ '/docs/configuration/' | relative_url }}" class="docs-card">
      <span class="docs-card-icon">⚙️</span>
      <h3 class="docs-card-title">Configuration</h3>
      <p class="docs-card-description">
        Comprehensive guide to configuration options, environment variables, and customization.
      </p>
    </a>
    
    <a href="{{ '/docs/examples/' | relative_url }}" class="docs-card">
      <span class="docs-card-icon">💡</span>
      <h3 class="docs-card-title">Examples</h3>
      <p class="docs-card-description">
        Real-world examples, use cases, and practical applications with sample queries and responses.
      </p>
    </a>
    
    <a href="{{ '/docs/web-service/' | relative_url }}" class="docs-card">
      <span class="docs-card-icon">🌐</span>
      <h3 class="docs-card-title">Web Service & JSON-RPC</h3>
      <p class="docs-card-description">
        HTTP API mode with JSON-RPC 2.0 endpoints, WebSocket subscriptions, health checks, and Prometheus metrics integration.
      </p>
    </a>
    
    <a href="{{ '/docs/metrics/' | relative_url }}" class="docs-card">
      <span class="docs-card-icon">📊</span>
      <h3 class="docs-card-title">Metrics & Monitoring</h3>
      <p class="docs-card-description">
        Prometheus metrics, Kubernetes autoscaling, performance monitoring, and observability setup.
      </p>
    </a>
    
    <a href="{{ '/docs/security-audit/' | relative_url }}" class="docs-card">
      <span class="docs-card-icon">🔒</span>
      <h3 class="docs-card-title">Security</h3>
      <p class="docs-card-description">
        Security audit reports, vulnerability assessments, best practices, and compliance information.
      </p>
    </a>
  </div>
</section>

<section style="margin: 4rem 0; padding: 3rem; background-color: var(--bg-tertiary); border: 2px solid var(--border-primary);">
  <h2 style="text-align: center; margin-bottom: 2rem; font-size: 1.5rem;">🎯 Usage Examples</h2>
  
  <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 2rem;">
    <div>
      <h3 style="color: var(--accent-primary); margin-bottom: 1rem;">💰 Basic Queries</h3>
      <ul style="list-style-type: none; padding: 0;">
        <li style="margin-bottom: 0.5rem; padding: 0.5rem; background: var(--bg-secondary); border: 1px solid var(--border-secondary);">
          "What's the SOL balance of address Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr?"
        </li>
        <li style="margin-bottom: 0.5rem; padding: 0.5rem; background: var(--bg-secondary); border: 1px solid var(--border-secondary);">
          "Show me the current slot number"
        </li>
        <li style="margin-bottom: 0.5rem; padding: 0.5rem; background: var(--bg-secondary); border: 1px solid var(--border-secondary);">
          "Get information about the latest block"
        </li>
      </ul>
    </div>
    
    <div>
      <h3 style="color: var(--accent-primary); margin-bottom: 1rem;">🌐 Multi-Network</h3>
      <ul style="list-style-type: none; padding: 0;">
        <li style="margin-bottom: 0.5rem; padding: 0.5rem; background: var(--bg-secondary); border: 1px solid var(--border-secondary);">
          "List all available SVM networks"
        </li>
        <li style="margin-bottom: 0.5rem; padding: 0.5rem; background: var(--bg-secondary); border: 1px solid var(--border-secondary);">
          "Enable Eclipse mainnet for queries"
        </li>
        <li style="margin-bottom: 0.5rem; padding: 0.5rem; background: var(--bg-secondary); border: 1px solid var(--border-secondary);">
          "Check SOL balance on all enabled networks"
        </li>
      </ul>
    </div>
    
    <div>
      <h3 style="color: var(--accent-primary); margin-bottom: 1rem;">🔧 Advanced Operations</h3>
      <ul style="list-style-type: none; padding: 0;">
        <li style="margin-bottom: 0.5rem; padding: 0.5rem; background: var(--bg-secondary); border: 1px solid var(--border-secondary);">
          "Show me the largest USDC token accounts"
        </li>
        <li style="margin-bottom: 0.5rem; padding: 0.5rem; background: var(--bg-secondary); border: 1px solid var(--border-secondary);">
          "Get the leader schedule for the current epoch"
        </li>
        <li style="margin-bottom: 0.5rem; padding: 0.5rem; background: var(--bg-secondary); border: 1px solid var(--border-secondary);">
          "Check block production stats for a validator"
        </li>
      </ul>
    </div>
  </div>
</section>

<section style="text-align: center; margin: 4rem 0;">
  <h2 style="margin-bottom: 1.5rem; font-size: 1.5rem;">🔗 Links & Resources</h2>
  <div style="display: flex; justify-content: center; gap: 1rem; flex-wrap: wrap;">
    <a href="{{ site.github.repository_url }}" class="btn" target="_blank" rel="noopener noreferrer">
      GitHub Repository
    </a>
    <a href="{{ site.github.repository_url }}/releases" class="btn" target="_blank" rel="noopener noreferrer">
      Releases
    </a>
    <a href="{{ site.github.repository_url }}/issues" class="btn" target="_blank" rel="noopener noreferrer">
      Report Issues
    </a>
    <a href="{{ '/search/' | relative_url }}" class="btn">
      Search Documentation
    </a>
  </div>
</section>

<script>
// Add some interactive enhancements
document.addEventListener('DOMContentLoaded', function() {
    // Animate feature cards on scroll
    const featureCards = document.querySelectorAll('.feature-card, .docs-card');
    
    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
            }
        });
    }, { 
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    });
    
    featureCards.forEach(card => {
        card.style.opacity = '0';
        card.style.transform = 'translateY(20px)';
        card.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
        observer.observe(card);
    });
    
    // Add hover effects to cards
    featureCards.forEach(card => {
        card.addEventListener('mouseenter', function() {
            this.style.transform = 'translateY(-5px)';
            this.style.boxShadow = '0 10px 25px rgba(0, 0, 0, 0.1)';
        });
        
        card.addEventListener('mouseleave', function() {
            this.style.transform = 'translateY(0)';
            this.style.boxShadow = '';
        });
    });
});
</script>