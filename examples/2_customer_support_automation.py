#!/usr/bin/env python3
"""
Example 2: Customer Support Automation
Demonstrates automated customer support ticket handling using AI Workflow System.

This example shows how to:
1. Process incoming support tickets
2. Categorize and route tickets
3. Generate automated responses
4. Escalate when necessary
"""

import os
import json
import time
import asyncio
import websockets
from typing import Dict, Any, List
from datetime import datetime
import requests

# Configuration
API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8080/api/v1")
WS_URL = os.getenv("WS_URL", "ws://localhost:8080/ws")
AUTH_TOKEN = os.getenv("AUTH_TOKEN", "your-jwt-token")

# Headers for API requests
headers = {
    "Authorization": f"Bearer {AUTH_TOKEN}",
    "Content-Type": "application/json"
}

# Sample support tickets
SAMPLE_TICKETS = [
    {
        "id": "TICKET-001",
        "customer_email": "user1@example.com",
        "subject": "Cannot login to my account",
        "message": "I've been trying to login for the past hour but keep getting an error. My username is user1@example.com",
        "priority": "high",
        "category": None  # To be determined by AI
    },
    {
        "id": "TICKET-002",
        "customer_email": "user2@example.com",
        "subject": "Feature request: Dark mode",
        "message": "It would be great if the application had a dark mode option. Many users prefer it for late night work.",
        "priority": "low",
        "category": None
    },
    {
        "id": "TICKET-003",
        "customer_email": "user3@example.com",
        "subject": "Billing issue - double charged",
        "message": "I was charged twice for my subscription this month. Please refund the extra charge immediately.",
        "priority": "urgent",
        "category": None
    },
    {
        "id": "TICKET-004",
        "customer_email": "spam@spammer.com",
        "subject": "YOU WON $1,000,000!!!",
        "message": "Click here to claim your prize! Limited time offer!",
        "priority": "low",
        "category": None
    }
]


class CustomerSupportAutomation:
    """Automated customer support system using AI workflows."""
    
    def __init__(self):
        self.processed_tickets = []
        self.ws_connection = None
    
    async def connect_websocket(self):
        """Connect to WebSocket for real-time updates."""
        try:
            self.ws_connection = await websockets.connect(
                WS_URL,
                extra_headers={"Authorization": f"Bearer {AUTH_TOKEN}"}
            )
            print("Connected to WebSocket for real-time updates")
        except Exception as e:
            print(f"WebSocket connection failed: {e}")
    
    async def listen_for_updates(self, instance_id: str):
        """Listen for workflow updates via WebSocket."""
        if not self.ws_connection:
            return
        
        # Subscribe to workflow updates
        await self.ws_connection.send(json.dumps({
            "type": "subscribe",
            "channel": "workflow",
            "instance_id": instance_id
        }))
        
        try:
            while True:
                message = await asyncio.wait_for(
                    self.ws_connection.recv(),
                    timeout=1.0
                )
                update = json.loads(message)
                
                if update.get("instance_id") == instance_id:
                    print(f"Real-time update: {update.get('status')} - {update.get('step')}")
                    
                    if update.get("status") in ["completed", "failed"]:
                        break
        except asyncio.TimeoutError:
            pass
    
    def process_ticket(self, ticket: Dict[str, Any]) -> Dict[str, Any]:
        """Process a single support ticket through the workflow."""
        print(f"\nProcessing ticket: {ticket['id']} - {ticket['subject']}")
        
        # Trigger customer support workflow
        response = requests.post(
            f"{API_BASE_URL}/workflows/trigger",
            headers=headers,
            json={
                "workflow_name": "customer_support_workflow",
                "inputs": {
                    "ticket_id": ticket["id"],
                    "customer_email": ticket["customer_email"],
                    "subject": ticket["subject"],
                    "message": ticket["message"],
                    "priority": ticket["priority"],
                    "timestamp": datetime.utcnow().isoformat()
                }
            }
        )
        response.raise_for_status()
        result = response.json()
        
        instance_id = result["instance_id"]
        print(f"Workflow triggered: {instance_id}")
        
        # Wait for completion
        start_time = time.time()
        timeout = 120  # 2 minutes
        
        while time.time() - start_time < timeout:
            response = requests.get(
                f"{API_BASE_URL}/workflows/status/{instance_id}",
                headers=headers
            )
            response.raise_for_status()
            status = response.json()
            
            if status["status"] == "completed":
                outputs = status["outputs"]
                
                # Extract results
                ticket_result = {
                    "ticket_id": ticket["id"],
                    "status": "processed",
                    "category": outputs.get("category", "uncategorized"),
                    "intent": outputs.get("intent", "unknown"),
                    "spam_score": outputs.get("spam_score", 0),
                    "response_generated": outputs.get("response_generated", False),
                    "escalated": outputs.get("escalated", False),
                    "automated_response": outputs.get("automated_response"),
                    "processing_time": time.time() - start_time
                }
                
                # Display results
                print(f"\nTicket {ticket['id']} processed:")
                print(f"  Category: {ticket_result['category']}")
                print(f"  Intent: {ticket_result['intent']}")
                print(f"  Spam Score: {ticket_result['spam_score']:.2f}")
                print(f"  Escalated: {ticket_result['escalated']}")
                
                if ticket_result["automated_response"]:
                    print(f"  Response: {ticket_result['automated_response'][:200]}...")
                
                return ticket_result
            
            elif status["status"] == "failed":
                print(f"Workflow failed: {status.get('error')}")
                return {
                    "ticket_id": ticket["id"],
                    "status": "failed",
                    "error": status.get("error")
                }
            
            time.sleep(2)
        
        print(f"Timeout processing ticket {ticket['id']}")
        return {
            "ticket_id": ticket["id"],
            "status": "timeout"
        }
    
    def generate_report(self, results: List[Dict[str, Any]]):
        """Generate a summary report of processed tickets."""
        print("\n=== Customer Support Automation Report ===\n")
        
        total_tickets = len(results)
        processed = sum(1 for r in results if r["status"] == "processed")
        failed = sum(1 for r in results if r["status"] == "failed")
        timed_out = sum(1 for r in results if r["status"] == "timeout")
        
        print(f"Total tickets: {total_tickets}")
        print(f"Successfully processed: {processed}")
        print(f"Failed: {failed}")
        print(f"Timed out: {timed_out}")
        
        if processed > 0:
            # Category breakdown
            categories = {}
            escalated_count = 0
            spam_count = 0
            avg_processing_time = 0
            
            for result in results:
                if result["status"] == "processed":
                    category = result.get("category", "uncategorized")
                    categories[category] = categories.get(category, 0) + 1
                    
                    if result.get("escalated"):
                        escalated_count += 1
                    
                    if result.get("spam_score", 0) > 0.7:
                        spam_count += 1
                    
                    avg_processing_time += result.get("processing_time", 0)
            
            avg_processing_time /= processed
            
            print("\nCategory Distribution:")
            for category, count in categories.items():
                print(f"  {category}: {count} ({count/processed*100:.1f}%)")
            
            print(f"\nEscalated tickets: {escalated_count}")
            print(f"Spam tickets: {spam_count}")
            print(f"Average processing time: {avg_processing_time:.2f} seconds")
        
        print("\n=== End of Report ===")
    
    async def run_automation(self):
        """Run the customer support automation demo."""
        print("=== Customer Support Automation Example ===\n")
        
        # Connect to WebSocket for real-time updates
        await self.connect_websocket()
        
        # Process all sample tickets
        results = []
        for ticket in SAMPLE_TICKETS:
            result = self.process_ticket(ticket)
            results.append(result)
            self.processed_tickets.append(result)
            
            # Small delay between tickets
            time.sleep(1)
        
        # Generate report
        self.generate_report(results)
        
        # Close WebSocket connection
        if self.ws_connection:
            await self.ws_connection.close()
    
    def demonstrate_routing_rules(self):
        """Demonstrate custom routing rules for tickets."""
        print("\n=== Custom Routing Rules Demo ===\n")
        
        # Define routing rules
        routing_rules = {
            "billing": {
                "team": "finance",
                "priority_boost": 1,
                "auto_escalate": True
            },
            "technical": {
                "team": "engineering",
                "priority_boost": 0,
                "auto_escalate": False
            },
            "feature_request": {
                "team": "product",
                "priority_boost": -1,
                "auto_escalate": False
            },
            "spam": {
                "team": "none",
                "priority_boost": -2,
                "auto_escalate": False
            }
        }
        
        print("Routing rules configured:")
        for category, rules in routing_rules.items():
            print(f"  {category}: -> Team: {rules['team']}, Priority: {rules['priority_boost']:+d}")
        
        # Apply routing to processed tickets
        print("\nApplying routing rules to processed tickets:")
        for ticket in self.processed_tickets:
            if ticket["status"] == "processed":
                category = ticket.get("category", "uncategorized")
                if category in routing_rules:
                    rule = routing_rules[category]
                    print(f"\nTicket {ticket['ticket_id']}:")
                    print(f"  Category: {category}")
                    print(f"  Routed to: {rule['team']} team")
                    print(f"  Priority adjustment: {rule['priority_boost']:+d}")
                    
                    if rule["auto_escalate"]:
                        print("  ⚠️  Auto-escalated due to category")


def demonstrate_batch_processing():
    """Demonstrate batch processing of tickets."""
    print("\n=== Batch Processing Demo ===\n")
    
    # Create a batch of tickets
    batch_size = 10
    batch_tickets = []
    
    for i in range(batch_size):
        batch_tickets.append({
            "id": f"BATCH-{i+1:03d}",
            "customer_email": f"customer{i+1}@example.com",
            "subject": f"Test ticket {i+1}",
            "message": "This is a test ticket for batch processing demonstration.",
            "priority": "medium"
        })
    
    print(f"Created batch of {batch_size} tickets")
    
    # Process in parallel (simulated)
    print("Processing tickets in parallel...")
    
    # In a real implementation, you would use asyncio or threading
    # For this demo, we'll just show the concept
    start_time = time.time()
    
    for i, ticket in enumerate(batch_tickets):
        print(f"  Processing ticket {i+1}/{batch_size}: {ticket['id']}")
        # Simulate processing
        time.sleep(0.1)
    
    elapsed = time.time() - start_time
    print(f"\nBatch processing completed in {elapsed:.2f} seconds")
    print(f"Average time per ticket: {elapsed/batch_size:.2f} seconds")


if __name__ == "__main__":
    # Run the main automation demo
    automation = CustomerSupportAutomation()
    asyncio.run(automation.run_automation())
    
    # Demonstrate routing rules
    automation.demonstrate_routing_rules()
    
    # Demonstrate batch processing
    demonstrate_batch_processing()