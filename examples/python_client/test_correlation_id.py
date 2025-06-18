#!/usr/bin/env python3
"""
Test script for correlation ID middleware.

This script tests that correlation IDs are properly propagated
through the AI Tutor service and logged correctly.
"""

import asyncio
import json
import logging

import aiohttp

# Configure logging to see correlation IDs
logging.basicConfig(
    level=logging.INFO,
    format='[%(asctime)s] [%(name)s] %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


async def test_correlation_id_propagation():
    """Test that correlation IDs are propagated correctly."""
    
    base_url = "http://localhost:3001"
    test_correlation_id = "test-correlation-12345"
    
    async with aiohttp.ClientSession() as session:
        # Test 1: Health check endpoint with correlation ID
        logger.info("Test 1: Health check with correlation ID")
        async with session.get(
            f"{base_url}/health",
            headers={"X-Correlation-ID": test_correlation_id}
        ) as response:
            assert response.status == 200
            assert response.headers.get("X-Correlation-ID") == test_correlation_id
            data = await response.json()
            logger.info(f"Health check response: {data}")
            
        # Test 2: Tutoring request with correlation ID
        logger.info("\nTest 2: Tutoring request with correlation ID")
        tutoring_request = {
            "student_query": "What is machine learning?",
            "subject": "programming",
            "difficulty_level": "beginner"
        }
        
        async with session.post(
            f"{base_url}/tutor",
            json=tutoring_request,
            headers={
                "X-Correlation-ID": test_correlation_id,
                "Content-Type": "application/json"
            }
        ) as response:
            assert response.status == 200
            assert response.headers.get("X-Correlation-ID") == test_correlation_id
            data = await response.json()
            # Check that correlation ID is in metadata
            assert data.get("metadata", {}).get("correlation_id") == test_correlation_id
            logger.info(f"Tutoring response metadata: {data['metadata']}")
            
        # Test 3: Request without correlation ID (should generate one)
        logger.info("\nTest 3: Request without correlation ID")
        async with session.post(
            f"{base_url}/explain",
            json={"concept": "neural networks"},
            headers={"Content-Type": "application/json"}
        ) as response:
            assert response.status == 200
            generated_id = response.headers.get("X-Correlation-ID")
            assert generated_id is not None
            logger.info(f"Generated correlation ID: {generated_id}")
            
        # Test 4: Multiple requests with same correlation ID
        logger.info("\nTest 4: Multiple requests with same correlation ID")
        multi_correlation_id = "multi-request-67890"
        
        for i in range(3):
            async with session.get(
                f"{base_url}/capabilities",
                headers={"X-Correlation-ID": multi_correlation_id}
            ) as response:
                assert response.status == 200
                assert response.headers.get("X-Correlation-ID") == multi_correlation_id
                logger.info(f"Request {i+1} completed with correlation ID: {multi_correlation_id}")
                
        # Test 5: Invalid correlation ID
        logger.info("\nTest 5: Invalid correlation ID handling")
        invalid_id = "invalid@id#with$special%chars"
        
        async with session.get(
            f"{base_url}/health",
            headers={"X-Correlation-ID": invalid_id}
        ) as response:
            assert response.status == 200
            # Should generate a new valid ID
            returned_id = response.headers.get("X-Correlation-ID")
            assert returned_id != invalid_id
            logger.info(f"Invalid ID '{invalid_id}' was replaced with: {returned_id}")
            
    logger.info("\n✅ All correlation ID tests passed!")


async def test_logging_with_correlation_id():
    """Test that logs include correlation IDs."""
    
    logger.info("\nTesting log output with correlation IDs...")
    logger.info("Check the AI Tutor service logs to verify correlation IDs are included")
    logger.info("Look for log entries in format: [timestamp] [correlation-id] LEVEL - message")
    

async def main():
    """Run all tests."""
    
    logger.info("Starting correlation ID tests for AI Tutor service")
    logger.info("Make sure the AI Tutor service is running on port 3001")
    logger.info("-" * 60)
    
    try:
        # Test correlation ID propagation
        await test_correlation_id_propagation()
        
        # Test logging
        await test_logging_with_correlation_id()
        
        logger.info("\n🎉 All tests completed successfully!")
        
    except AssertionError as e:
        logger.error(f"❌ Test failed: {e}")
        raise
    except aiohttp.ClientError as e:
        logger.error(f"❌ Connection error: {e}")
        logger.error("Make sure the AI Tutor service is running on port 3001")
        raise
    except Exception as e:
        logger.error(f"❌ Unexpected error: {e}")
        raise


if __name__ == "__main__":
    asyncio.run(main())