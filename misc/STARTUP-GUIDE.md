# 🚀 AI Workflow Engine - Startup Guide

This guide provides instructions for running the AI Workflow Engine using either local services or Docker containers.

## 📋 Prerequisites

### For Local Development
- **Rust** (latest stable)
- **Node.js** 18+
- **Python** 3.9+
- **PostgreSQL** 14+
- **Redis** 6+

### For Docker
- **Docker** 20.10+
- **Docker Compose** 2.0+

## 🏃 Quick Start

### Option 1: Local Development (No Docker)

```bash
# Start all services locally
./start-local.sh

# Stop all services
./stop-local.sh
```

### Option 2: Docker (Recommended for Production)

```bash
# Start minimal Docker setup (PostgreSQL, Redis, API, Frontend)
./start-docker.sh

# Start full Docker setup with monitoring (Prometheus, Grafana)
./start-docker.sh --full

# Stop all services
./stop-docker.sh

# Note: MCP servers need to be run locally even with Docker:
cd scripts && ./start_test_servers.sh
```

## 📖 Detailed Instructions

### Local Development Setup

The `start-local.sh` script will:

1. **Check Prerequisites** - Verify all required tools are installed
2. **Start Databases** - Ensure PostgreSQL and Redis are running
3. **Initialize Database** - Create database and run migrations
4. **Start MCP Servers** - Launch Python MCP servers on ports 8001-8003
5. **Build Backend** - Compile Rust backend in release mode
6. **Start API Server** - Launch main API on port 8080
7. **Start Frontend** - Launch React dev server on port 5173

#### Environment Variables (Local)

Create a `.env` file in the root directory:

```bash
# Database
DATABASE_URL=postgresql://localhost/ai_workflow_db

# JWT Secret
JWT_SECRET=your-secure-jwt-secret-key-change-in-production

# Logging
RUST_LOG=info,workflow_engine=debug

# Optional: AI Provider Keys
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# Optional: MCP Server Keys
HELPSCOUT_API_KEY=...
NOTION_API_KEY=...
SLACK_BOT_TOKEN=xoxb-...
```

#### Starting Individual Services

```bash
# Start only MCP servers
cd mcp-servers && source venv/bin/activate
python -m servers.helpscout --port 8001
python -m servers.notion --port 8002
python -m servers.slack --port 8003

# Start only backend
cargo run --release --bin workflow-engine

# Start only frontend
cd frontend && npm run dev
```

### Docker Setup

The `start-docker.sh` script will:

1. **Check Docker** - Verify Docker is installed and running
2. **Create .env** - Generate default environment file
3. **Build Images** - Build all Docker images in parallel
4. **Start Infrastructure** - Launch PostgreSQL and Redis
5. **Initialize Database** - Run database migrations
6. **Start Services** - Launch all application services
7. **Start Monitoring** - Launch Prometheus and Grafana
8. **Health Checks** - Verify all services are running

#### Docker Services

| Service | Port | Description |
|---------|------|-------------|
| Frontend | 5173 | React development server |
| API Server | 8080 | Main Rust API backend |
| PostgreSQL | 5432 | Primary database |
| Redis | 6379 | Cache and queue |
| MCP HelpScout | 8001 | Customer support integration |
| MCP Notion | 8002 | Knowledge base integration |
| MCP Slack | 8003 | Team communication |
| Prometheus | 9090 | Metrics collection |
| Grafana | 3000 | Monitoring dashboards |

#### Docker Commands

```bash
# View logs for a specific service
docker-compose logs -f ai-workflow-system

# View all running services
docker-compose ps

# Restart a specific service
docker-compose restart frontend

# Execute commands in a container
docker-compose exec ai-workflow-system /bin/bash

# Remove all containers and volumes
docker-compose down -v
```

## 🔧 Troubleshooting

### Local Development Issues

#### PostgreSQL Not Running
```bash
# macOS
brew services start postgresql@14

# Linux
sudo systemctl start postgresql

# Create database manually
createdb ai_workflow_db
```

#### Port Already in Use
```bash
# Find process using port
lsof -i :8080

# Kill process
kill -9 <PID>
```

#### Python Virtual Environment
```bash
# Recreate virtual environment
cd mcp-servers
rm -rf venv
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### Docker Issues

#### Docker Daemon Not Running
- **macOS/Windows**: Start Docker Desktop
- **Linux**: `sudo systemctl start docker`

#### Permission Denied
```bash
# Add user to docker group
sudo usermod -aG docker $USER
# Log out and back in
```

#### Container Fails to Start
```bash
# Check logs
docker-compose -f docker-compose.simple.yml logs <service-name>

# Rebuild specific service
docker-compose -f docker-compose.simple.yml build --no-cache <service-name>

# Reset everything
docker-compose -f docker-compose.simple.yml down -v
docker system prune -a
```

#### Microservices Build Errors
If you see errors about missing `services/shared` directory:
- The project uses a simplified docker-compose that excludes microservices
- To include microservices, ensure they're properly configured in your environment

## 📊 Monitoring

### Local Development
- **Logs**: Check `logs/` directory for service logs
- **PIDs**: Check `pids/` directory for process IDs
- **Health**: `curl http://localhost:8080/health`

### Docker
- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090
- **Container Stats**: `docker stats`

## 🛠️ Advanced Configuration

### Enable Microservices (Local)

```bash
# Start with microservices enabled
START_MICROSERVICES=true ./start-local.sh
```

### Custom Ports

Edit the scripts to change default ports:
- Frontend: 5173
- API: 8080
- MCP Servers: 8001-8003

### Production Deployment

For production, use Docker with:
1. Real SSL certificates
2. Secure JWT secrets
3. Production database credentials
4. API rate limiting
5. Monitoring and alerting

## 📚 Additional Resources

- [Main Documentation](CLAUDE.md)
- [Frontend Documentation](frontend/CLAUDE.md)
- [API Documentation](http://localhost:8080/swagger-ui/)
- [E2E Test Overview](frontend/E2E-TEST-OVERVIEW.md)

## 🆘 Getting Help

1. Check service logs in `logs/` directory
2. Run health checks: `curl http://localhost:8080/health`
3. Review error messages in browser console
4. Check Docker logs: `docker-compose logs -f`
5. File issues on GitHub with error details