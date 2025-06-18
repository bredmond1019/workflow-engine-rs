#!/usr/bin/env python3
"""
Example 3: Multi-Source Knowledge Base Search
Demonstrates searching across multiple knowledge sources using AI Workflow System.

This example shows how to:
1. Search across Notion, Slack, and HelpScout
2. Aggregate and rank results
3. Generate unified responses
4. Cache results for performance
"""

import os
import json
import time
import hashlib
from typing import Dict, Any, List, Optional
from datetime import datetime, timedelta
import requests

# Configuration
API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8080/api/v1")
AUTH_TOKEN = os.getenv("AUTH_TOKEN", "your-jwt-token")

# Headers for API requests
headers = {
    "Authorization": f"Bearer {AUTH_TOKEN}",
    "Content-Type": "application/json"
}


class KnowledgeBaseSearch:
    """Multi-source knowledge base search system."""
    
    def __init__(self):
        self.search_cache = {}
        self.cache_ttl = 3600  # 1 hour cache TTL
        self.available_sources = self._check_available_sources()
    
    def _check_available_sources(self) -> List[str]:
        """Check which knowledge sources are available."""
        try:
            response = requests.get(
                f"{API_BASE_URL}/registry/agents",
                headers=headers
            )
            response.raise_for_status()
            agents = response.json().get("agents", [])
            
            # Map agent names to knowledge sources
            source_mapping = {
                "notion-agent": "notion",
                "slack-agent": "slack",
                "helpscout-agent": "helpscout"
            }
            
            available = []
            for agent in agents:
                if agent["status"] == "active" and agent["name"] in source_mapping:
                    available.append(source_mapping[agent["name"]])
            
            print(f"Available knowledge sources: {', '.join(available)}")
            return available
            
        except Exception as e:
            print(f"Error checking sources: {e}")
            return []
    
    def _get_cache_key(self, query: str, sources: List[str]) -> str:
        """Generate cache key for a search query."""
        content = f"{query}:{','.join(sorted(sources))}"
        return hashlib.md5(content.encode()).hexdigest()
    
    def _is_cache_valid(self, cache_entry: Dict[str, Any]) -> bool:
        """Check if a cache entry is still valid."""
        if not cache_entry:
            return False
        
        cached_time = datetime.fromisoformat(cache_entry["timestamp"])
        return datetime.utcnow() - cached_time < timedelta(seconds=self.cache_ttl)
    
    def search(
        self,
        query: str,
        sources: Optional[List[str]] = None,
        filters: Optional[Dict[str, Any]] = None,
        use_cache: bool = True
    ) -> Dict[str, Any]:
        """Search across multiple knowledge sources."""
        print(f"\n🔍 Searching for: '{query}'")
        
        # Use all available sources if not specified
        if sources is None:
            sources = self.available_sources
        else:
            # Filter to only available sources
            sources = [s for s in sources if s in self.available_sources]
        
        if not sources:
            print("No knowledge sources available")
            return {"results": [], "error": "No knowledge sources available"}
        
        print(f"Searching in: {', '.join(sources)}")
        
        # Check cache
        cache_key = self._get_cache_key(query, sources)
        if use_cache and cache_key in self.search_cache:
            cache_entry = self.search_cache[cache_key]
            if self._is_cache_valid(cache_entry):
                print("✅ Returning cached results")
                return cache_entry["results"]
        
        # Trigger knowledge base search workflow
        response = requests.post(
            f"{API_BASE_URL}/workflows/trigger",
            headers=headers,
            json={
                "workflow_name": "knowledge_base_search",
                "inputs": {
                    "query": query,
                    "sources": sources,
                    "filters": filters or {},
                    "max_results_per_source": 5,
                    "include_relevance_scores": True
                }
            }
        )
        response.raise_for_status()
        instance_id = response.json()["instance_id"]
        
        # Wait for results
        results = self._wait_for_results(instance_id)
        
        if results:
            # Cache results
            if use_cache:
                self.search_cache[cache_key] = {
                    "timestamp": datetime.utcnow().isoformat(),
                    "results": results
                }
            
            return results
        
        return {"results": [], "error": "Search failed"}
    
    def _wait_for_results(self, instance_id: str, timeout: int = 60) -> Optional[Dict[str, Any]]:
        """Wait for workflow to complete and return results."""
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            response = requests.get(
                f"{API_BASE_URL}/workflows/status/{instance_id}",
                headers=headers
            )
            response.raise_for_status()
            status = response.json()
            
            if status["status"] == "completed":
                outputs = status["outputs"]
                
                # Process and return results
                results = {
                    "total_results": outputs.get("total_results", 0),
                    "sources_searched": outputs.get("sources_searched", []),
                    "results": outputs.get("aggregated_results", []),
                    "search_time": time.time() - start_time
                }
                
                return results
            
            elif status["status"] == "failed":
                print(f"Search failed: {status.get('error')}")
                return None
            
            time.sleep(1)
        
        print("Search timed out")
        return None
    
    def display_results(self, results: Dict[str, Any]):
        """Display search results in a formatted way."""
        print(f"\n📊 Search Results ({results['total_results']} total)")
        print(f"Sources searched: {', '.join(results['sources_searched'])}")
        print(f"Search time: {results['search_time']:.2f} seconds\n")
        
        for i, result in enumerate(results["results"], 1):
            print(f"{i}. {result['title']}")
            print(f"   Source: {result['source']} | Relevance: {result['relevance_score']:.2f}")
            print(f"   {result['summary'][:150]}...")
            
            if result.get("url"):
                print(f"   URL: {result['url']}")
            
            print()
    
    def generate_unified_response(self, query: str, search_results: Dict[str, Any]) -> str:
        """Generate a unified response based on search results."""
        print("\n🤖 Generating unified response...")
        
        if search_results["total_results"] == 0:
            return "No relevant information found for your query."
        
        # Trigger response generation workflow
        response = requests.post(
            f"{API_BASE_URL}/workflows/trigger",
            headers=headers,
            json={
                "workflow_name": "generate_knowledge_response",
                "inputs": {
                    "query": query,
                    "search_results": search_results["results"],
                    "response_style": "comprehensive",
                    "max_length": 500
                }
            }
        )
        response.raise_for_status()
        instance_id = response.json()["instance_id"]
        
        # Wait for response
        start_time = time.time()
        timeout = 30
        
        while time.time() - start_time < timeout:
            response = requests.get(
                f"{API_BASE_URL}/workflows/status/{instance_id}",
                headers=headers
            )
            response.raise_for_status()
            status = response.json()
            
            if status["status"] == "completed":
                return status["outputs"].get("response", "Failed to generate response")
            elif status["status"] == "failed":
                return "Failed to generate response"
            
            time.sleep(1)
        
        return "Response generation timed out"


def demonstrate_basic_search():
    """Demonstrate basic knowledge base search."""
    print("=== Basic Knowledge Base Search Demo ===\n")
    
    kb = KnowledgeBaseSearch()
    
    # Simple search
    results = kb.search("How to set up authentication in our API?")
    kb.display_results(results)
    
    # Generate unified response
    response = kb.generate_unified_response(
        "How to set up authentication in our API?",
        results
    )
    print(f"Unified Response:\n{response}")


def demonstrate_filtered_search():
    """Demonstrate search with filters and specific sources."""
    print("\n=== Filtered Search Demo ===\n")
    
    kb = KnowledgeBaseSearch()
    
    # Search only in Notion with date filter
    results = kb.search(
        query="Recent product updates",
        sources=["notion"],
        filters={
            "date_range": "last_30_days",
            "content_type": "documentation"
        }
    )
    kb.display_results(results)
    
    # Search in Slack for discussions
    results = kb.search(
        query="bug reports",
        sources=["slack"],
        filters={
            "channels": ["#engineering", "#support"],
            "date_range": "last_7_days"
        }
    )
    kb.display_results(results)


def demonstrate_complex_queries():
    """Demonstrate handling complex, multi-part queries."""
    print("\n=== Complex Query Demo ===\n")
    
    kb = KnowledgeBaseSearch()
    
    complex_queries = [
        "What are the differences between our Pro and Enterprise plans, and how do users upgrade?",
        "Find all discussions about performance issues in the last month and their resolutions",
        "Show me the API documentation for user management and any related support tickets"
    ]
    
    for query in complex_queries:
        print(f"\n{'='*60}")
        results = kb.search(query)
        kb.display_results(results)
        
        # Generate comprehensive response
        response = kb.generate_unified_response(query, results)
        print(f"\n📝 Unified Response:\n{response}")
        
        # Show cache hit on second search
        print("\n🔄 Searching again (should use cache)...")
        start = time.time()
        cached_results = kb.search(query)
        print(f"Cache lookup time: {time.time() - start:.3f} seconds")


def demonstrate_real_time_search():
    """Demonstrate real-time search with WebSocket updates."""
    print("\n=== Real-Time Search Demo ===\n")
    
    kb = KnowledgeBaseSearch()
    
    # Simulate real-time search scenario
    print("Simulating real-time search updates...")
    
    queries = [
        "error handling best practices",
        "deployment procedures",
        "customer onboarding process"
    ]
    
    for query in queries:
        print(f"\n🔍 New search: '{query}'")
        
        # Trigger search
        response = requests.post(
            f"{API_BASE_URL}/workflows/trigger",
            headers=headers,
            json={
                "workflow_name": "knowledge_base_search",
                "inputs": {
                    "query": query,
                    "sources": kb.available_sources,
                    "real_time": True
                }
            }
        )
        response.raise_for_status()
        instance_id = response.json()["instance_id"]
        
        # Simulate receiving real-time updates
        print("Receiving updates:")
        for i in range(3):
            time.sleep(1)
            print(f"  ✓ Searched {kb.available_sources[i % len(kb.available_sources)]}")
        
        # Get final results
        results = kb._wait_for_results(instance_id)
        if results:
            print(f"  ✓ Found {results['total_results']} results")


def create_search_analytics():
    """Create analytics for search patterns."""
    print("\n=== Search Analytics Demo ===\n")
    
    # Simulate search history
    search_history = [
        {"query": "authentication", "count": 45, "avg_results": 12},
        {"query": "api documentation", "count": 38, "avg_results": 8},
        {"query": "error handling", "count": 31, "avg_results": 15},
        {"query": "deployment", "count": 28, "avg_results": 6},
        {"query": "billing", "count": 24, "avg_results": 4},
    ]
    
    print("Top Search Queries (Last 30 Days):")
    print("-" * 50)
    print(f"{'Query':<25} {'Count':<10} {'Avg Results'}")
    print("-" * 50)
    
    for item in search_history:
        print(f"{item['query']:<25} {item['count']:<10} {item['avg_results']}")
    
    # Search performance metrics
    print("\n\nSearch Performance Metrics:")
    print("-" * 50)
    print(f"Average search time: 2.3 seconds")
    print(f"Cache hit rate: 68%")
    print(f"Failed searches: 2%")
    print(f"Total searches: 531")


if __name__ == "__main__":
    # Run all demonstrations
    demonstrate_basic_search()
    demonstrate_filtered_search()
    demonstrate_complex_queries()
    demonstrate_real_time_search()
    create_search_analytics()