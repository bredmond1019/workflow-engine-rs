#!/usr/bin/env python3
"""
AI Workflow System Python Client

This module provides a Python client for integrating external services
with the AI Workflow System registry and cross-system communication.

Usage:
    from ai_workflow_client import AIWorkflowClient, ServiceConfig
    
    # Configure the service
    config = ServiceConfig(
        name="ai-tutor-service",
        endpoint="http://localhost:3001",
        capabilities=["tutoring", "education", "assistance"],
        registry_endpoint="http://localhost:8080"
    )
    
    # Create client and register
    client = AIWorkflowClient(config)
    await client.register()
    
    # Start heartbeat loop
    heartbeat_task = await client.start_heartbeat()
    
    # The service is now registered and will maintain its heartbeat
"""

import asyncio
import json
import logging
import time
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Dict, List, Optional, Any
from uuid import UUID, uuid4

import aiohttp
import pydantic
from pydantic import BaseModel

# Try to import correlation middleware (optional dependency)
try:
    from correlation_middleware import get_correlation_id, inject_correlation_id_header
    HAS_CORRELATION_SUPPORT = True
except ImportError:
    HAS_CORRELATION_SUPPORT = False
    def get_correlation_id():
        return None
    def inject_correlation_id_header(headers, correlation_id=None):
        return headers


# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class ServiceConfig(BaseModel):
    """Configuration for a service that registers with the AI Workflow System."""
    
    name: str
    endpoint: str
    capabilities: List[str]
    registry_endpoint: str
    auth_token: Optional[str] = None
    heartbeat_interval: int = 60
    metadata: Dict[str, Any] = field(default_factory=dict)
    service_type: str = "python"
    
    class Config:
        # Allow field assignment after initialization
        allow_mutation = True


class RegistrationRequest(BaseModel):
    """Request payload for service registration."""
    
    name: str
    endpoint: str
    capabilities: List[str]
    metadata: Dict[str, Any] = field(default_factory=dict)


class HeartbeatRequest(BaseModel):
    """Request payload for heartbeat."""
    
    timestamp: datetime
    status: Optional[str] = "active"
    metadata: Optional[Dict[str, Any]] = None


class RegistrationResponse(BaseModel):
    """Response from service registration."""
    
    id: str  # UUID as string
    name: str
    endpoint: str
    capabilities: List[str]
    status: str
    last_seen: datetime
    metadata: Dict[str, Any]
    created_at: datetime
    updated_at: datetime


class AIWorkflowError(Exception):
    """Base exception for AI Workflow client errors."""
    pass


class RegistrationError(AIWorkflowError):
    """Error during service registration."""
    pass


class HeartbeatError(AIWorkflowError):
    """Error during heartbeat."""
    pass


class ServiceDiscoveryError(AIWorkflowError):
    """Error during service discovery."""
    pass


class AIWorkflowClient:
    """
    Python client for integrating with the AI Workflow System.
    
    This client provides functionality for:
    - Service registration
    - Automatic heartbeat maintenance  
    - Service discovery
    - Cross-system communication
    """
    
    def __init__(self, config: ServiceConfig):
        """
        Initialize the AI Workflow client.
        
        Args:
            config: Service configuration
        """
        self.config = config
        self.agent_id: Optional[UUID] = None
        self.session: Optional[aiohttp.ClientSession] = None
        self.heartbeat_task: Optional[asyncio.Task] = None
        self._registered = False
        
    async def __aenter__(self):
        """Async context manager entry."""
        await self.start()
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self.stop()
        
    async def start(self):
        """Start the client session."""
        if self.session is None:
            timeout = aiohttp.ClientTimeout(total=30)
            self.session = aiohttp.ClientSession(timeout=timeout)
            
    async def stop(self):
        """Stop the client and cleanup resources."""
        if self.heartbeat_task:
            self.heartbeat_task.cancel()
            try:
                await self.heartbeat_task
            except asyncio.CancelledError:
                pass
                
        if self.session:
            await self.session.close()
            self.session = None
            
    async def register(self) -> RegistrationResponse:
        """
        Register this service with the AI Workflow System registry.
        
        Returns:
            RegistrationResponse: Registration details including agent ID
            
        Raises:
            RegistrationError: If registration fails
        """
        if not self.session:
            await self.start()
            
        registration = RegistrationRequest(
            name=self.config.name,
            endpoint=self.config.endpoint,
            capabilities=self.config.capabilities,
            metadata=self.config.metadata
        )
        
        url = f"{self.config.registry_endpoint}/registry/agents"
        headers = {"Content-Type": "application/json"}
        
        # Add correlation ID if available
        headers = inject_correlation_id_header(headers)
        
        if self.config.auth_token:
            headers["Authorization"] = f"Bearer {self.config.auth_token}"
            
        try:
            async with self.session.post(
                url,
                json=registration.dict(),
                headers=headers
            ) as response:
                if response.status == 200 or response.status == 201:
                    data = await response.json()
                    registration_response = RegistrationResponse(**data)
                    self.agent_id = UUID(registration_response.id)
                    self._registered = True
                    
                    logger.info(
                        f"✅ Service '{self.config.name}' registered successfully "
                        f"with ID: {self.agent_id}"
                    )
                    
                    return registration_response
                else:
                    error_text = await response.text()
                    raise RegistrationError(
                        f"Registration failed with status {response.status}: {error_text}"
                    )
                    
        except aiohttp.ClientError as e:
            raise RegistrationError(f"Network error during registration: {e}")
            
    async def send_heartbeat(self) -> bool:
        """
        Send a heartbeat to the registry.
        
        Returns:
            bool: True if heartbeat was successful
            
        Raises:
            HeartbeatError: If heartbeat fails
        """
        if not self.agent_id:
            raise HeartbeatError("Service not registered - cannot send heartbeat")
            
        if not self.session:
            await self.start()
            
        heartbeat = HeartbeatRequest(
            timestamp=datetime.now(timezone.utc),
            status="active",
            metadata=self.config.metadata
        )
        
        url = f"{self.config.registry_endpoint}/registry/agents/{self.agent_id}/heartbeat"
        headers = {"Content-Type": "application/json"}
        
        # Add correlation ID if available
        headers = inject_correlation_id_header(headers)
        
        if self.config.auth_token:
            headers["Authorization"] = f"Bearer {self.config.auth_token}"
            
        try:
            async with self.session.post(
                url,
                json=heartbeat.dict(),
                headers=headers
            ) as response:
                if response.status == 200:
                    logger.debug(f"💓 Heartbeat sent for service '{self.config.name}'")
                    return True
                else:
                    error_text = await response.text()
                    logger.error(
                        f"❌ Heartbeat failed for service '{self.config.name}': "
                        f"HTTP {response.status} - {error_text}"
                    )
                    return False
                    
        except aiohttp.ClientError as e:
            logger.error(f"❌ Heartbeat error for service '{self.config.name}': {e}")
            return False
            
    async def start_heartbeat(self) -> asyncio.Task:
        """
        Start the automatic heartbeat loop.
        
        Returns:
            asyncio.Task: The heartbeat task
        """
        if not self._registered:
            raise HeartbeatError("Service must be registered before starting heartbeat")
            
        async def heartbeat_loop():
            """Background heartbeat loop."""
            while True:
                try:
                    await asyncio.sleep(self.config.heartbeat_interval)
                    await self.send_heartbeat()
                except asyncio.CancelledError:
                    logger.info(f"Heartbeat loop cancelled for service '{self.config.name}'")
                    break
                except Exception as e:
                    logger.error(f"Error in heartbeat loop: {e}")
                    # Continue the loop even if individual heartbeats fail
                    
        self.heartbeat_task = asyncio.create_task(heartbeat_loop())
        logger.info(
            f"🔄 Started heartbeat loop for service '{self.config.name}' "
            f"(interval: {self.config.heartbeat_interval}s)"
        )
        
        return self.heartbeat_task
        
    async def discover_services(self, capability: str) -> List[str]:
        """
        Discover services by capability.
        
        Args:
            capability: The capability to search for
            
        Returns:
            List of service names that have the specified capability
            
        Raises:
            ServiceDiscoveryError: If discovery fails
        """
        if not self.session:
            await self.start()
            
        url = f"{self.config.registry_endpoint}/registry/agents/discover"
        params = {"capability": capability}
        headers = {}
        
        # Add correlation ID if available
        headers = inject_correlation_id_header(headers)
        
        if self.config.auth_token:
            headers["Authorization"] = f"Bearer {self.config.auth_token}"
            
        try:
            async with self.session.get(
                url,
                params=params,
                headers=headers
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    agents = data.get("agents", [])
                    service_names = [agent["name"] for agent in agents]
                    
                    logger.debug(
                        f"🔍 Discovered {len(service_names)} services with capability '{capability}': "
                        f"{service_names}"
                    )
                    
                    return service_names
                else:
                    error_text = await response.text()
                    raise ServiceDiscoveryError(
                        f"Service discovery failed with status {response.status}: {error_text}"
                    )
                    
        except aiohttp.ClientError as e:
            raise ServiceDiscoveryError(f"Network error during service discovery: {e}")
            
    async def call_service(
        self,
        service_name: str,
        method: str,
        payload: Dict[str, Any]
    ) -> Dict[str, Any]:
        """
        Make a cross-system call to another service.
        
        Args:
            service_name: Name of the service to call
            method: Method/endpoint to call on the service
            payload: Request payload
            
        Returns:
            Response data from the service
            
        Raises:
            ServiceDiscoveryError: If the service is not found
            AIWorkflowError: If the service call fails
        """
        if not self.session:
            await self.start()
            
        # First get the service endpoint
        endpoint = await self._get_service_endpoint(service_name)
        
        # Make the service call
        url = f"{endpoint}/{method}"
        headers = {"Content-Type": "application/json"}
        
        # Add correlation ID if available
        headers = inject_correlation_id_header(headers)
        
        if self.config.auth_token:
            headers["Authorization"] = f"Bearer {self.config.auth_token}"
            
        try:
            async with self.session.post(
                url,
                json=payload,
                headers=headers
            ) as response:
                if response.status == 200:
                    result = await response.json()
                    logger.debug(
                        f"✅ Successfully called {service_name}/{method}"
                    )
                    return result
                else:
                    error_text = await response.text()
                    raise AIWorkflowError(
                        f"Service call failed: {service_name}/{method} "
                        f"returned {response.status}: {error_text}"
                    )
                    
        except aiohttp.ClientError as e:
            raise AIWorkflowError(f"Network error calling {service_name}/{method}: {e}")
            
    async def _get_service_endpoint(self, service_name: str) -> str:
        """Get the endpoint for a service by name."""
        url = f"{self.config.registry_endpoint}/registry/agents"
        headers = {}
        
        # Add correlation ID if available
        headers = inject_correlation_id_header(headers)
        
        if self.config.auth_token:
            headers["Authorization"] = f"Bearer {self.config.auth_token}"
            
        async with self.session.get(url, headers=headers) as response:
            if response.status == 200:
                data = await response.json()
                agents = data.get("agents", [])
                
                for agent in agents:
                    if agent.get("name") == service_name:
                        return agent.get("endpoint")
                        
                raise ServiceDiscoveryError(f"Service '{service_name}' not found")
            else:
                error_text = await response.text()
                raise ServiceDiscoveryError(
                    f"Failed to get service list: HTTP {response.status} - {error_text}"
                )
                
    @property
    def is_registered(self) -> bool:
        """Check if the service is registered."""
        return self._registered
        
    @property 
    def agent_id_str(self) -> Optional[str]:
        """Get the agent ID as a string."""
        return str(self.agent_id) if self.agent_id else None


# Example usage
async def main():
    """Example usage of the AI Workflow client."""
    
    # Configure the AI Tutor service
    config = ServiceConfig(
        name="ai-tutor-service",
        endpoint="http://localhost:3001",
        capabilities=["tutoring", "education", "assistance"],
        registry_endpoint="http://localhost:8080",
        heartbeat_interval=30,
        metadata={
            "version": "1.0.0",
            "language": "python",
            "framework": "fastapi"
        }
    )
    
    # Create and use the client
    async with AIWorkflowClient(config) as client:
        try:
            # Register the service
            response = await client.register()
            print(f"Registered with ID: {response.id}")
            
            # Start heartbeat
            heartbeat_task = await client.start_heartbeat()
            
            # Discover other services
            tutoring_services = await client.discover_services("tutoring")
            print(f"Found tutoring services: {tutoring_services}")
            
            # Keep the service running
            print("Service is running. Press Ctrl+C to stop.")
            await asyncio.sleep(60)  # Run for 1 minute
            
        except KeyboardInterrupt:
            print("Shutting down...")
        except Exception as e:
            print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main())