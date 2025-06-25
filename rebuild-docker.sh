#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}              Force Rebuild Docker Images                       ${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Stop all containers
echo -e "${YELLOW}Stopping all containers...${NC}"
docker-compose -f docker-compose.minimal.yml down -v

# Remove the specific image to force rebuild
echo -e "${YELLOW}Removing old images...${NC}"
docker rmi ai-workflow-system:latest 2>/dev/null || true
docker rmi $(docker images -q -f "dangling=true") 2>/dev/null || true

# Clean build cache
echo -e "${YELLOW}Cleaning Docker build cache...${NC}"
docker builder prune -f

# Rebuild with no cache
echo -e "${YELLOW}Rebuilding with no cache...${NC}"
docker-compose -f docker-compose.minimal.yml build --no-cache ai-workflow-system

echo -e "${GREEN}✓ Rebuild complete!${NC}"
echo -e "${GREEN}Run ./start-docker.sh to start the services${NC}"