# Deployment Guide

## Quick Deployment for CEO Demo

### Option 1: Vercel (Recommended for Demo)
```bash
# Install Vercel CLI
npm install -g vercel

# Deploy
vercel

# Follow prompts, then access your live URL
```

### Option 2: Netlify
```bash
# Install Netlify CLI
npm install -g netlify-cli

# Build and deploy
npm run build
netlify deploy --prod --dir=dist
```

### Option 3: Local Demo
```bash
# Start backend server first
cd .. && cargo run --bin workflow-engine

# In another terminal, start frontend
cd frontend && npm run dev

# Access at http://localhost:5173
```

## Environment Setup for Production

Create `.env.production`:
```
VITE_API_URL=https://your-backend-api.com
VITE_APP_NAME=AI Workflow Engine
VITE_APP_VERSION=1.0.0
```

## Pre-Demo Checklist

✅ Backend API server is running
✅ Frontend builds successfully (`npm run build`)
✅ Environment variables are configured
✅ Demo scenarios are tested
✅ All dependencies are installed

## Demo Flow for CEO

1. **Login Page** - Use "Admin Demo" quick login
2. **Dashboard** - Show real-time metrics and charts
3. **Live Demos** - Run Customer Support demo
4. **Workflow Management** - Show instance monitoring
5. **Business Value** - Highlight ROI metrics

## Performance Notes

- Initial bundle size: ~2.5MB (acceptable for demo)
- Load time: <3 seconds on good connection
- All assets are optimized for production
- Charts render smoothly with sample data

## Troubleshooting

**Build Issues:**
- Run `npm run type-check` to identify TypeScript issues
- Check that all dependencies are installed

**Runtime Issues:**
- Verify backend API is accessible
- Check browser console for errors
- Ensure JWT authentication is working

**Demo Issues:**
- Use quick login buttons for immediate access
- Demo scenarios work with mock data if backend unavailable
- All major browsers supported (Chrome, Firefox, Safari, Edge)