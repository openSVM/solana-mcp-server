# Netlify Deployment Guide

This guide explains how to deploy the Solana MCP Server documentation to Netlify.

## What Gets Deployed

The Netlify configuration deploys:
- **Jekyll Documentation Site**: Complete documentation with search, navigation, and examples
- **Static Assets**: CSS, JavaScript, and images with optimized caching
- **API Documentation**: RPC method references and guides

## Prerequisites

Before deploying, ensure you have:

1. **Ruby & Bundler**: Required for building the Jekyll site
   ```bash
   gem install bundler
   ```

2. **Netlify CLI** (optional, for command-line deployment):
   ```bash
   npm install -g netlify-cli
   netlify login
   ```

## Deployment Methods

### Method 1: Automatic Deployment via GitHub (Recommended)

1. **Connect Repository to Netlify**:
   - Go to [Netlify](https://app.netlify.com/)
   - Click "Add new site" → "Import an existing project"
   - Choose "GitHub" and select the `solana-mcp-server` repository
   - Netlify will automatically detect the `netlify.toml` configuration

2. **Configure Build Settings** (auto-detected from netlify.toml):
   - Build command: `bundle install && bundle exec jekyll build`
   - Publish directory: `_site`
   - Ruby version: `3.2.0`

3. **Deploy**:
   - Click "Deploy site"
   - Netlify will automatically build and deploy on every push to the main branch

### Method 2: Manual Deployment via CLI

1. **Build and Deploy**:
   ```bash
   # Deploy preview
   ./scripts/deploy-netlify.sh
   
   # Deploy to production
   ./scripts/deploy-netlify.sh --prod
   ```

2. **Link to Netlify Site** (first time only):
   ```bash
   netlify link
   ```

### Method 3: One-Click Deploy

[![Deploy to Netlify](https://www.netlify.com/img/deploy/button.svg)](https://app.netlify.com/start/deploy?repository=https://github.com/openSVM/solana-mcp-server)

## Configuration Details

The `netlify.toml` file includes:

### Build Configuration
- **Jekyll build**: Compiles Markdown documentation to HTML
- **Asset optimization**: Minifies CSS/JS, compresses images
- **Environment**: Production settings for optimal performance

### Headers
- **Security headers**: XSS protection, frame options, content type
- **Cache headers**: Optimized caching for static assets (1 year) and HTML (1 hour)

### Redirects
- Clean URL redirects for better SEO
- Legacy path compatibility (e.g., `/docs` → `/docs/`)
- Custom 404 page handling

### Performance
- Lighthouse plugin for performance audits
- Image optimization and compression
- CSS/JS bundling and minification

## Environment Variables

Set these in Netlify dashboard under Site settings → Environment variables:

```bash
JEKYLL_ENV=production
RUBY_VERSION=3.2.0
```

## Deployment Contexts

### Production
- URL: `https://your-site.netlify.app`
- Built from: `main` branch
- Settings: Production optimizations enabled

### Deploy Previews
- URL: `https://deploy-preview-XX--your-site.netlify.app`
- Built from: Pull requests
- Settings: Includes drafts and future posts

### Branch Deploys
- URL: `https://branch-name--your-site.netlify.app`
- Built from: Feature branches
- Settings: Same as deploy previews

## Custom Domain

To use a custom domain:

1. Go to Site settings → Domain management
2. Add your custom domain
3. Configure DNS settings as instructed by Netlify
4. SSL certificate is automatically provisioned

## Monitoring & Analytics

### Built-in Features
- **Netlify Analytics**: Page views, popular pages, traffic sources
- **Deploy logs**: Real-time build and deploy logs
- **Performance metrics**: Lighthouse scores and Core Web Vitals

### Access Logs
View deployment logs:
```bash
netlify logs
```

## Troubleshooting

### Build Failures

**Jekyll dependency issues:**
```bash
bundle update
bundle exec jekyll build
```

**Ruby version mismatch:**
Update `netlify.toml`:
```toml
[build.environment]
  RUBY_VERSION = "3.2.0"  # Match your local version
```

### Deployment Issues

**CLI not authenticated:**
```bash
netlify login
netlify link
```

**Build cache issues:**
- Go to Site settings → Build & deploy → Clear cache and retry deploy

### Preview Not Working

Check that your branch is configured:
- Site settings → Build & deploy → Deploy contexts
- Enable "Branch deploys" for your branches

## Advanced Configuration

### Custom Functions (Future Enhancement)

To add serverless functions for API endpoints:

1. Create `netlify/functions/` directory
2. Add JavaScript/TypeScript functions
3. Functions will be available at `/.netlify/functions/function-name`

Example structure:
```
netlify/
  functions/
    mcp-api.js        # MCP API endpoint
    health.js         # Health check
```

### Edge Functions

For edge computing capabilities:
```toml
[[edge_functions]]
  path = "/api/*"
  function = "mcp-handler"
```

## Continuous Deployment

### Automatic Deploys
- **Main branch**: Automatically deploys to production
- **Pull requests**: Creates deploy preview
- **Other branches**: Optional branch deploys

### Deploy Hooks
Generate deploy hooks for external triggers:
- Site settings → Build & deploy → Deploy hooks
- Use webhook URL to trigger builds from external services

## Performance Optimization

The configuration includes:
- ✅ Asset minification (CSS, JS, HTML)
- ✅ Image compression
- ✅ Smart caching headers
- ✅ Prerendering for better SEO
- ✅ HTTP/2 and HTTP/3 support
- ✅ Global CDN distribution

## Security Features

- ✅ Automatic HTTPS/SSL
- ✅ Security headers (XSS, frame options, etc.)
- ✅ DDoS protection
- ✅ Form spam filtering
- ✅ Branch deploy access control

## Cost Considerations

**Free Tier includes:**
- 100 GB bandwidth/month
- 300 build minutes/month
- Unlimited sites
- Deploy previews
- HTTPS/SSL

**Paid plans** offer more bandwidth, build minutes, and advanced features.

## Support

For issues or questions:
- **Netlify Documentation**: https://docs.netlify.com
- **Netlify Support**: https://support.netlify.com
- **Repository Issues**: https://github.com/openSVM/solana-mcp-server/issues

## Next Steps

After deployment:
1. ✅ Verify site loads correctly
2. ✅ Test all documentation links
3. ✅ Check performance with Lighthouse
4. ✅ Set up custom domain (optional)
5. ✅ Enable analytics
6. ✅ Configure branch deploys for your workflow
