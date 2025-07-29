# 📚 Solana MCP Server Documentation Site

This repository hosts a modern, e-ink inspired documentation website for the Solana MCP Server project, built with Jekyll and deployed on GitHub Pages.

## 🎨 Features

### Modern E-ink Design
- **Grayscale color palette** inspired by e-reader aesthetics
- **Clean typography** using Inter and JetBrains Mono fonts
- **Minimalist interface** focusing on content readability
- **Responsive design** that works on all devices

### Advanced Functionality
- **🔍 Full-text search** powered by Lunr.js
- **📖 Bookmark system** with local storage persistence
- **🧭 Smart navigation** with breadcrumbs and page navigation
- **📱 Mobile-optimized** responsive design
- **♿ Accessibility features** including skip links and ARIA labels

### Content Organization
- **Comprehensive documentation** covering all 47 RPC methods
- **Step-by-step guides** for deployment and configuration
- **Interactive examples** with code snippets
- **Architecture diagrams** and system overviews
- **Security audit reports** and best practices

## 🛠️ Technical Stack

- **Static Site Generator**: Jekyll 4.3+
- **Styling**: Custom CSS with CSS Grid and Flexbox
- **Search**: Lunr.js full-text search
- **Icons**: Custom SVG icons and emojis
- **Deployment**: GitHub Actions → GitHub Pages
- **Performance**: Optimized CSS/JS, lazy loading, service worker

## 🚀 Local Development

### Prerequisites
- Ruby 3.1+
- Bundler
- Git

### Setup
```bash
# Clone the repository
git clone https://github.com/openSVM/solana-mcp-server.git
cd solana-mcp-server

# Install dependencies
bundle install

# Serve locally
bundle exec jekyll serve

# View at http://localhost:4000
```

### Development Commands
```bash
# Build the site
bundle exec jekyll build

# Serve with live reload
bundle exec jekyll serve --livereload

# Build for production
JEKYLL_ENV=production bundle exec jekyll build
```

## 📁 Site Structure

```
├── _config.yml              # Jekyll configuration
├── _layouts/                 # Page templates
│   ├── default.html         # Base layout with navigation
│   ├── home.html            # Homepage layout
│   └── docs.html            # Documentation layout
├── _docs/                   # Documentation collection
│   ├── onboarding.md        # Quick start guide
│   ├── api-reference.md     # Complete API docs
│   ├── architecture.md      # System architecture
│   ├── deployment.md        # Deployment guides
│   └── ...
├── assets/
│   ├── css/main.css         # E-ink themed styles
│   ├── js/main.js          # Search & bookmark functionality
│   └── images/             # Logos and icons
├── index.md                # Homepage content
├── search.md               # Search page
├── bookmarks.md            # Bookmarks page
└── Gemfile                 # Ruby dependencies
```

## 🎯 Content Guidelines

### Writing Style
- **Clear and concise** technical writing
- **Step-by-step instructions** with code examples
- **Consistent terminology** across all pages
- **Practical examples** that users can follow

### Documentation Organization
- **Progressive disclosure** from basic to advanced concepts
- **Cross-references** between related topics
- **Code examples** with proper syntax highlighting
- **Visual aids** like diagrams and screenshots

### Markdown Conventions
```markdown
---
layout: docs
title: "Page Title"
description: "Brief description"
order: 1
category: getting_started
---

# Page Title

Brief introduction paragraph.

## Section Header

Content with `inline code` and:

```bash
# Code blocks with language specification
command --with --arguments
```

> **Note**: Important callouts in blockquotes

- Bullet points for lists
- Use emojis sparingly for visual interest
```

## 🔧 Customization

### Theme Colors
Edit CSS custom properties in `/assets/css/main.css`:
```css
:root {
  --eink-white: #ffffff;
  --eink-paper: #fafafa;
  --eink-light-gray: #f5f5f5;
  --accent-teal: #2d5f5f;
  /* ... */
}
```

### Navigation
Update navigation in `_config.yml`:
```yaml
nav_links:
  - title: "Documentation"
    url: "/docs/"
  - title: "API Reference"
    url: "/docs/api-reference/"
```

### Search Configuration
Search is automatically configured but can be customized in `/assets/js/main.js`.

## 📊 Analytics & Performance

### Built-in Features
- **Performance monitoring** with Core Web Vitals
- **Search analytics** tracking user engagement
- **Bookmark usage** metrics
- **Mobile responsiveness** testing

### Optimization
- **CSS minification** in production builds
- **Image optimization** with lazy loading
- **Service worker** for offline functionality
- **CDN delivery** via GitHub Pages

## 🚀 Deployment

### Automatic Deployment
The site automatically deploys to GitHub Pages when changes are pushed to the main branch via GitHub Actions.

### Manual Deployment
```bash
# Build for production
JEKYLL_ENV=production bundle exec jekyll build

# Deploy to GitHub Pages
# (Handled automatically by GitHub Actions)
```

### Custom Domain
To use a custom domain:
1. Add CNAME file with your domain
2. Configure DNS settings
3. Enable HTTPS in repository settings

## 🤝 Contributing

### Content Updates
1. Edit markdown files in `_docs/` directory
2. Follow existing naming conventions
3. Add appropriate front matter
4. Test locally before submitting PR

### Design Changes
1. Modify CSS in `/assets/css/main.css`
2. Maintain grayscale e-ink aesthetic
3. Test across different screen sizes
4. Ensure accessibility compliance

### New Features  
1. Add JavaScript to `/assets/js/main.js`
2. Update templates in `_layouts/` if needed
3. Document new features in this README
4. Test thoroughly across browsers

## 📝 License

This documentation site is part of the Solana MCP Server project and is licensed under the MIT License.

## 🔗 Links

- **Live Site**: https://opensvm.github.io/solana-mcp-server/
- **Main Repository**: https://github.com/openSVM/solana-mcp-server
- **Issues**: https://github.com/openSVM/solana-mcp-server/issues
- **Jekyll Documentation**: https://jekyllrb.com/docs/