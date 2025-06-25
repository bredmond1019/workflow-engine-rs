#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}                 Docker Cleanup Script                          ${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Check Docker disk usage before cleanup
echo -e "\n${YELLOW}Current Docker disk usage:${NC}"
docker system df

# Stop all running containers
echo -e "\n${YELLOW}Stopping all running containers...${NC}"
docker stop $(docker ps -q) 2>/dev/null || true

# Remove all stopped containers
echo -e "\n${YELLOW}Removing stopped containers...${NC}"
docker container prune -f

# Remove all unused images
echo -e "\n${YELLOW}Removing unused images...${NC}"
docker image prune -a -f

# Remove all unused volumes
echo -e "\n${YELLOW}Removing unused volumes...${NC}"
docker volume prune -f

# Remove all unused networks
echo -e "\n${YELLOW}Removing unused networks...${NC}"
docker network prune -f

# Remove build cache
echo -e "\n${YELLOW}Removing build cache...${NC}"
docker builder prune -a -f

# Show disk usage after cleanup
echo -e "\n${GREEN}Docker disk usage after cleanup:${NC}"
docker system df

echo -e "\n${GREEN}✓ Docker cleanup complete!${NC}"
echo -e "${YELLOW}Note: You may need to rebuild images after this cleanup.${NC}"