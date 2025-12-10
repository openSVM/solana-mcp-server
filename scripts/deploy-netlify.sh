#!/bin/bash
# Deployment script for Netlify
# This script prepares and deploys the Solana MCP Server documentation to Netlify

set -e

echo "🚀 Deploying Solana MCP Server Documentation to Netlify..."

# Check if Netlify CLI is installed
if ! command -v netlify &> /dev/null; then
    echo "📦 Installing Netlify CLI..."
    npm install -g netlify-cli
fi

# Check if Ruby and Bundle are installed
if ! command -v bundle &> /dev/null; then
    echo "⚠️  Bundle not found. Please install Ruby and Bundler first:"
    echo "    gem install bundler"
    exit 1
fi

# Install dependencies
echo "📦 Installing Ruby dependencies..."
bundle install

# Build the Jekyll site
echo "🔨 Building Jekyll site..."
JEKYLL_ENV=production bundle exec jekyll build

# Deploy to Netlify
echo "🌐 Deploying to Netlify..."
if [ "$1" == "--prod" ]; then
    echo "🚀 Deploying to production..."
    netlify deploy --prod --dir=_site
else
    echo "🔍 Deploying preview..."
    netlify deploy --dir=_site
fi

echo "✅ Deployment complete!"
echo ""
echo "To deploy to production, run:"
echo "  ./scripts/deploy-netlify.sh --prod"
echo ""
echo "To check deployment status:"
echo "  netlify status"
echo ""
echo "To open the site:"
echo "  netlify open:site"
