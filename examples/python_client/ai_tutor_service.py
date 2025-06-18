#!/usr/bin/env python3
"""
AI Tutor Service Example

This is an example AI Tutor service that demonstrates how to:
1. Register with the AI Workflow System registry
2. Maintain heartbeat automatically
3. Provide tutoring capabilities via HTTP endpoints
4. Handle cross-system communication

Run this service alongside the AI Workflow System registry to see
cross-system integration in action.

Usage:
    python ai_tutor_service.py

Dependencies:
    pip install aiohttp pydantic fastapi uvicorn
"""

import asyncio
import logging
import os
import signal
import sys
from contextlib import asynccontextmanager
from typing import Dict, Any, List

from fastapi import FastAPI, HTTPException, Request
from pydantic import BaseModel
import uvicorn

# Import our AI Workflow client
from ai_workflow_client import AIWorkflowClient, ServiceConfig, AIWorkflowError

# Import correlation ID middleware
from correlation_middleware import (
    CorrelationIdMiddleware,
    configure_logging_with_correlation_id,
    get_correlation_id,
    inject_correlation_id_header
)


# Configure logging with correlation ID support
configure_logging_with_correlation_id(
    log_format='[%(asctime)s] [%(correlation_id)s] %(levelname)s - %(name)s - %(message)s',
    log_level=os.getenv('LOG_LEVEL', 'INFO')
)
logger = logging.getLogger(__name__)


# Request/Response models for the AI Tutor API
class TutoringRequest(BaseModel):
    """Request for tutoring assistance."""
    student_query: str
    subject: str = "general"
    difficulty_level: str = "intermediate"
    learning_style: str = "explanatory"
    context: Dict[str, Any] = {}


class TutoringResponse(BaseModel):
    """Response from tutoring service."""
    explanation: str
    examples: List[str] = []
    follow_up_questions: List[str] = []
    resources: List[str] = []
    confidence_score: float = 0.8
    metadata: Dict[str, Any] = {}


class HealthResponse(BaseModel):
    """Health check response."""
    status: str
    service: str
    version: str
    capabilities: List[str]
    registered: bool


# Global variables for service management
workflow_client: AIWorkflowClient = None
heartbeat_task: asyncio.Task = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Manage the service lifecycle."""
    global workflow_client, heartbeat_task
    
    # Startup
    logger.info("🚀 Starting AI Tutor Service...")
    
    # Configure the service for registration
    config = ServiceConfig(
        name=os.getenv("AI_TUTOR_NAME", "ai-tutor-service"),
        endpoint=os.getenv("AI_TUTOR_ENDPOINT", "http://localhost:3001"),
        capabilities=["tutoring", "education", "assistance", "explanation"],
        registry_endpoint=os.getenv("REGISTRY_ENDPOINT", "http://localhost:8080"),
        auth_token=os.getenv("AUTH_TOKEN"),
        heartbeat_interval=int(os.getenv("HEARTBEAT_INTERVAL", "60")),
        metadata={
            "version": "1.0.0",
            "language": "python",
            "framework": "fastapi",
            "subjects": ["math", "science", "programming", "general"],
            "max_concurrent_sessions": 50
        }
    )
    
    try:
        # Create and start the workflow client
        workflow_client = AIWorkflowClient(config)
        await workflow_client.start()
        
        # Register with the AI Workflow System
        registration_response = await workflow_client.register()
        logger.info(f"✅ Registered with AI Workflow System: {registration_response.id}")
        
        # Start heartbeat
        heartbeat_task = await workflow_client.start_heartbeat()
        logger.info("💓 Heartbeat started")
        
        yield  # Service is running
        
    except Exception as e:
        logger.error(f"❌ Failed to start service: {e}")
        raise
    finally:
        # Shutdown
        logger.info("🛑 Shutting down AI Tutor Service...")
        
        if heartbeat_task:
            heartbeat_task.cancel()
            
        if workflow_client:
            await workflow_client.stop()


# Create FastAPI app with lifecycle management
app = FastAPI(
    title="AI Tutor Service",
    description="An AI-powered tutoring service that integrates with the AI Workflow System",
    version="1.0.0",
    lifespan=lifespan
)

# Add correlation ID middleware
app.add_middleware(
    CorrelationIdMiddleware,
    header_name='X-Correlation-ID',
    validate=True
)


@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint (no authentication required)."""
    return HealthResponse(
        status="healthy",
        service="ai-tutor-service",
        version="1.0.0",
        capabilities=["tutoring", "education", "assistance", "explanation"],
        registered=workflow_client.is_registered if workflow_client else False
    )


@app.post("/tutor", response_model=TutoringResponse)
async def provide_tutoring(request: TutoringRequest, http_request: Request):
    """
    Main tutoring endpoint.
    
    This endpoint simulates an AI tutor providing educational assistance.
    In a real implementation, this would integrate with an actual AI model.
    """
    correlation_id = get_correlation_id()
    logger.info(f"📚 Tutoring request: {request.subject} - {request.student_query[:50]}...")
    
    try:
        # Simulate AI tutoring logic
        explanation = generate_explanation(request)
        examples = generate_examples(request)
        follow_up_questions = generate_follow_up_questions(request)
        resources = generate_resources(request)
        
        response = TutoringResponse(
            explanation=explanation,
            examples=examples,
            follow_up_questions=follow_up_questions,
            resources=resources,
            confidence_score=0.85,
            metadata={
                "processing_time_ms": 150,
                "model_used": "tutoring-gpt-4",
                "subject_detected": request.subject,
                "complexity_level": "intermediate",
                "correlation_id": correlation_id
            }
        )
        
        logger.info(f"✅ Tutoring response generated for: {request.subject}")
        return response
        
    except Exception as e:
        logger.error(f"❌ Error generating tutoring response: {e}")
        raise HTTPException(status_code=500, detail="Failed to generate tutoring response")


@app.post("/explain")
async def explain_concept(request: Dict[str, Any]):
    """
    Endpoint for explaining concepts.
    
    This is a simplified endpoint that other services can call for explanations.
    """
    concept = request.get("concept", "")
    context = request.get("context", {})
    
    if not concept:
        raise HTTPException(status_code=400, detail="Concept is required")
        
    logger.info(f"🔍 Explanation request for concept: {concept}")
    
    # Simulate explanation generation
    explanation = f"""
    **{concept}** is an important concept that can be understood as follows:
    
    {concept} involves several key aspects that are fundamental to understanding 
    the broader topic. Here's a clear explanation:
    
    1. **Definition**: {concept} refers to...
    2. **Key Components**: The main elements include...
    3. **Applications**: This concept is used in...
    4. **Common Misconceptions**: Students often think...
    
    This explanation is tailored to help you understand {concept} in a practical way.
    """
    
    return {
        "concept": concept,
        "explanation": explanation.strip(),
        "confidence": 0.9,
        "source": "ai-tutor-service"
    }


@app.get("/capabilities")
async def get_capabilities():
    """Get the capabilities of this service."""
    return {
        "service": "ai-tutor-service",
        "capabilities": ["tutoring", "education", "assistance", "explanation"],
        "subjects": ["math", "science", "programming", "general"],
        "endpoints": [
            "/tutor - Main tutoring endpoint",
            "/explain - Concept explanation endpoint",
            "/capabilities - This endpoint",
            "/health - Health check"
        ],
        "version": "1.0.0"
    }


@app.post("/discover")
async def discover_related_services():
    """
    Discover other services in the AI Workflow System.
    
    This demonstrates cross-system service discovery.
    """
    if not workflow_client:
        raise HTTPException(status_code=503, detail="Service not connected to registry")
        
    try:
        # Discover services with different capabilities
        education_services = await workflow_client.discover_services("education")
        workflow_services = await workflow_client.discover_services("workflow")
        documentation_services = await workflow_client.discover_services("documentation")
        
        return {
            "discovered_services": {
                "education": education_services,
                "workflow": workflow_services,
                "documentation": documentation_services
            },
            "total_services": len(education_services) + len(workflow_services) + len(documentation_services),
            "discovery_time": "2024-01-01T12:00:00Z"
        }
        
    except AIWorkflowError as e:
        logger.error(f"Service discovery failed: {e}")
        raise HTTPException(status_code=503, detail=f"Service discovery failed: {e}")


def generate_explanation(request: TutoringRequest) -> str:
    """Generate an explanation for the tutoring request."""
    
    subject_specific_intro = {
        "math": "Let's solve this step-by-step using mathematical principles.",
        "science": "Let's explore this scientific concept with clear explanations.",
        "programming": "Let's break down this programming concept with practical examples.",
        "general": "Let's approach this topic systematically."
    }
    
    intro = subject_specific_intro.get(request.subject, subject_specific_intro["general"])
    
    explanation = f"""
    {intro}
    
    Regarding your question: "{request.student_query}"
    
    Here's a comprehensive explanation tailored to the {request.difficulty_level} level:
    
    **Core Concept**: The fundamental idea here is that {request.student_query.lower()} 
    involves understanding several key principles that work together.
    
    **Detailed Explanation**: 
    When we examine this topic, we need to consider multiple perspectives. 
    The concept builds upon foundational knowledge and extends into practical applications.
    
    **Key Points to Remember**:
    • This concept connects to broader themes in {request.subject}
    • Understanding this helps with more advanced topics
    • Practical application is important for retention
    
    **Why This Matters**: 
    Understanding this concept is crucial because it forms the basis for more advanced 
    learning in {request.subject} and provides practical skills you can apply.
    """
    
    return explanation.strip()


def generate_examples(request: TutoringRequest) -> List[str]:
    """Generate examples for the tutoring request."""
    
    examples = [
        f"Example 1: A practical application of this concept in {request.subject}",
        f"Example 2: How this relates to everyday situations you might encounter",
        f"Example 3: A step-by-step walkthrough of the process"
    ]
    
    if request.subject == "math":
        examples.extend([
            "Example 4: Numerical calculation demonstrating the principle",
            "Example 5: Word problem application"
        ])
    elif request.subject == "programming":
        examples.extend([
            "Example 4: Code snippet showing implementation",
            "Example 5: Common use case in software development"
        ])
        
    return examples[:3]  # Return first 3 examples


def generate_follow_up_questions(request: TutoringRequest) -> List[str]:
    """Generate follow-up questions for deeper learning."""
    
    return [
        f"How might this concept apply to other areas of {request.subject}?",
        "What would happen if we modified one of the key parameters?",
        "Can you think of a real-world situation where this would be useful?",
        "What's the next logical step in learning about this topic?"
    ]


def generate_resources(request: TutoringRequest) -> List[str]:
    """Generate learning resources for the topic."""
    
    return [
        f"Khan Academy: {request.subject.title()} Fundamentals",
        f"Interactive tutorial: Understanding {request.student_query}",
        f"Practice problems: {request.subject.title()} exercises",
        "Video explanation: Visual learning approach"
    ]


def signal_handler(signum, frame):
    """Handle shutdown signals gracefully."""
    logger.info(f"Received signal {signum}. Shutting down gracefully...")
    sys.exit(0)


if __name__ == "__main__":
    # Set up signal handlers for graceful shutdown
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # Get configuration from environment
    host = os.getenv("AI_TUTOR_HOST", "0.0.0.0")
    port = int(os.getenv("AI_TUTOR_PORT", "3001"))
    
    logger.info(f"🎓 Starting AI Tutor Service on {host}:{port}")
    
    # Run the service
    uvicorn.run(
        app,
        host=host,
        port=port,
        log_level="info",
        access_log=True
    )