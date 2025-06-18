#!/usr/bin/env python3
"""
Example 1: Blog Content Pipeline
Demonstrates a complete content creation workflow using AI Workflow System.

This example shows how to:
1. Research a topic using AI
2. Generate a blog post outline
3. Create full content
4. Save to Notion
"""

import os
import json
import time
import requests
from typing import Dict, Any

# Configuration
API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8080/api/v1")
AUTH_TOKEN = os.getenv("AUTH_TOKEN", "your-jwt-token")

# Headers for API requests
headers = {
    "Authorization": f"Bearer {AUTH_TOKEN}",
    "Content-Type": "application/json"
}


def trigger_workflow(workflow_name: str, inputs: Dict[str, Any]) -> str:
    """Trigger a workflow and return the instance ID."""
    response = requests.post(
        f"{API_BASE_URL}/workflows/trigger",
        headers=headers,
        json={
            "workflow_name": workflow_name,
            "inputs": inputs
        }
    )
    response.raise_for_status()
    result = response.json()
    print(f"Triggered workflow: {result['instance_id']}")
    return result["instance_id"]


def wait_for_completion(instance_id: str, timeout: int = 300) -> Dict[str, Any]:
    """Wait for workflow to complete and return the final status."""
    start_time = time.time()
    
    while time.time() - start_time < timeout:
        response = requests.get(
            f"{API_BASE_URL}/workflows/status/{instance_id}",
            headers=headers
        )
        response.raise_for_status()
        status = response.json()
        
        print(f"Status: {status['status']} - Progress: {status['progress']['percentage']}%")
        
        if status["status"] in ["completed", "failed"]:
            return status
        
        time.sleep(5)  # Poll every 5 seconds
    
    raise TimeoutError(f"Workflow {instance_id} did not complete within {timeout} seconds")


def create_blog_content_pipeline():
    """Main example: Create a complete blog post from topic to publication."""
    print("=== Blog Content Pipeline Example ===\n")
    
    # Step 1: Research the topic
    print("Step 1: Researching topic...")
    research_id = trigger_workflow(
        "research_to_documentation",
        {
            "topic": "The Impact of AI on Software Development",
            "difficulty": "intermediate"
        }
    )
    
    research_result = wait_for_completion(research_id)
    
    if research_result["status"] != "completed":
        print(f"Research failed: {research_result.get('error')}")
        return
    
    research_output = research_result["outputs"]
    print(f"Research completed. Summary: {research_output.get('summary', '')[:200]}...")
    
    # Step 2: Generate blog content
    print("\nStep 2: Generating blog content...")
    content_id = trigger_workflow(
        "ai_content_generation",
        {
            "content_type": "blog_post",
            "topic": "The Impact of AI on Software Development",
            "target_audience": "software developers",
            "word_count": 1500,
            "tone": "professional yet engaging",
            "research_data": research_output
        }
    )
    
    content_result = wait_for_completion(content_id)
    
    if content_result["status"] != "completed":
        print(f"Content generation failed: {content_result.get('error')}")
        return
    
    blog_content = content_result["outputs"]
    print(f"Blog post generated. Title: {blog_content.get('title', 'Untitled')}")
    
    # Step 3: Save to Notion (if available)
    print("\nStep 3: Saving to Notion...")
    
    # First, check if Notion agent is available
    agents_response = requests.get(
        f"{API_BASE_URL}/registry/agents",
        headers=headers
    )
    agents = agents_response.json().get("agents", [])
    notion_available = any(agent["name"] == "notion-agent" for agent in agents)
    
    if notion_available:
        publish_id = trigger_workflow(
            "publish_to_notion",
            {
                "title": blog_content.get("title", "AI Impact on Software Development"),
                "content": blog_content.get("content", ""),
                "metadata": {
                    "author": "AI Workflow System",
                    "category": "Technology",
                    "tags": ["AI", "Software Development", "Automation"]
                }
            }
        )
        
        publish_result = wait_for_completion(publish_id)
        
        if publish_result["status"] == "completed":
            print(f"Successfully published to Notion!")
            print(f"Page URL: {publish_result['outputs'].get('page_url', 'N/A')}")
        else:
            print(f"Publishing failed: {publish_result.get('error')}")
    else:
        print("Notion agent not available. Saving content locally...")
        
        # Save to local file
        filename = f"blog_post_{int(time.time())}.md"
        with open(filename, "w") as f:
            f.write(f"# {blog_content.get('title', 'Untitled')}\n\n")
            f.write(blog_content.get("content", ""))
        
        print(f"Content saved to: {filename}")
    
    print("\n=== Pipeline Complete ===")
    
    # Display summary
    print("\nSummary:")
    print(f"- Research completed in: {research_result['progress']['completed_steps']} steps")
    print(f"- Blog post word count: {blog_content.get('word_count', 'N/A')}")
    print(f"- Total time: {time.time() - start_time:.2f} seconds")


def demonstrate_template_usage():
    """Demonstrate using workflow templates."""
    print("\n=== Template Usage Example ===\n")
    
    # List available templates
    response = requests.get(
        f"{API_BASE_URL}/templates",
        headers=headers
    )
    response.raise_for_status()
    templates = response.json()["templates"]
    
    print("Available templates:")
    for template in templates:
        print(f"- {template['id']}: {template['name']} ({template['complexity']})")
    
    # Search for content generation templates
    print("\nSearching for content generation templates...")
    response = requests.get(
        f"{API_BASE_URL}/templates/search",
        headers=headers,
        params={"category": "content_generation"}
    )
    response.raise_for_status()
    content_templates = response.json()["templates"]
    
    if content_templates:
        template = content_templates[0]
        print(f"\nUsing template: {template['name']}")
        print(f"Description: {template['description']}")
        
        # Trigger template
        response = requests.post(
            f"{API_BASE_URL}/templates/trigger",
            headers=headers,
            json={
                "template_id": template["id"],
                "inputs": {
                    "topic": "Best Practices for API Design",
                    "format": "tutorial"
                }
            }
        )
        response.raise_for_status()
        result = response.json()
        print(f"Template triggered: {result['instance_id']}")


if __name__ == "__main__":
    # Run the main example
    create_blog_content_pipeline()
    
    # Demonstrate template usage
    demonstrate_template_usage()