#!/usr/bin/env python3
"""
Customer Support Core Logic (MCP-independent)

This module contains the core business logic for customer support functionality,
separated from MCP protocol dependencies for easier testing and development.

Following TDD principles - GREEN phase implementation to make tests pass.
"""

from datetime import datetime
from typing import Optional, List, Dict, Any, Tuple


def analyze_message_sentiment(message: str) -> Tuple[str, float]:
    """
    Analyze sentiment of a customer support message
    
    Args:
        message: Customer message text
        
    Returns:
        Tuple of (sentiment, confidence) where:
        - sentiment: "positive", "negative", or "neutral"
        - confidence: float between 0.0 and 1.0
        
    Raises:
        TypeError: If message is not a string
    """
    if not isinstance(message, str):
        raise TypeError(f"Message must be a string, got {type(message)}")
    
    positive_words = ["happy", "great", "excellent", "good", "satisfied", "pleased"]
    negative_words = ["angry", "frustrated", "terrible", "bad", "disappointed", "upset"]
    
    message_lower = message.lower()
    positive_count = sum(1 for word in positive_words if word in message_lower)
    negative_count = sum(1 for word in negative_words if word in message_lower)
    
    if positive_count > negative_count:
        sentiment = "positive"
        confidence = min(0.9, 0.6 + (positive_count * 0.1))
    elif negative_count > positive_count:
        sentiment = "negative"
        confidence = min(0.9, 0.6 + (negative_count * 0.1))
    else:
        sentiment = "neutral"
        confidence = 0.75
    
    return sentiment, confidence


def categorize_message(message: str) -> Tuple[str, float, Dict[str, int]]:
    """
    Categorize a customer support message
    
    Args:
        message: Customer message text
        
    Returns:
        Tuple of (category, confidence, scores) where:
        - category: Primary category string
        - confidence: float between 0.0 and 1.0
        - scores: Dict mapping categories to keyword match counts
        
    Raises:
        TypeError: If message is not a string
    """
    if not isinstance(message, str):
        raise TypeError(f"Message must be a string, got {type(message)}")
    
    categories = {
        "billing": ["bill", "charge", "payment", "invoice", "refund", "subscription"],
        "technical": ["error", "bug", "crash", "not working", "broken", "issue"],
        "account": ["login", "password", "access", "account", "profile", "settings"],
        "general": ["question", "help", "how to", "information", "support"]
    }
    
    message_lower = message.lower()
    category_scores = {}
    
    for category, keywords in categories.items():
        score = sum(1 for keyword in keywords if keyword in message_lower)
        if score > 0:
            category_scores[category] = score
    
    # If no categories match, default to general
    primary_category = max(category_scores, key=category_scores.get) if category_scores else "general"
    confidence = min(0.9, category_scores.get(primary_category, 0) * 0.2 + 0.3)
    
    return primary_category, confidence, category_scores


def validate_ticket_data(
    ticket_id: str,
    customer_id: str,
    message: str,
    priority: str = "medium"
) -> Dict[str, Any]:
    """
    Validate customer support ticket data for completeness and format
    
    Args:
        ticket_id: Unique ticket identifier
        customer_id: Customer identifier
        message: Customer message content
        priority: Ticket priority (low, medium, high, urgent)
    
    Returns:
        Validation result dictionary with status and any missing fields
    """
    # Normalize priority
    valid_priorities = ["low", "medium", "high", "urgent"]
    if priority not in valid_priorities:
        priority = "medium"
    
    # Simple validation logic
    is_valid = bool(ticket_id and customer_id and message)
    missing_fields = []
    
    if not ticket_id:
        missing_fields.append("ticket_id")
    if not customer_id:
        missing_fields.append("customer_id")
    if not message:
        missing_fields.append("message")
    
    return {
        "validation_result": {
            "is_valid": is_valid,
            "missing_fields": missing_fields,
            "ticket_id": ticket_id,
            "priority": priority,
            "timestamp": datetime.now().isoformat()
        }
    }


def check_customer_status(customer_id: str, include_billing: bool = False) -> Dict[str, Any]:
    """
    Check customer account status (mock implementation for testing)
    
    Args:
        customer_id: Customer identifier
        include_billing: Whether to include billing information
        
    Returns:
        Customer status information dictionary
    """
    # Mock implementation for testing
    status_data = {
        "customer_status": {
            "customer_id": customer_id,
            "account_status": "active",
            "tier": "premium" if customer_id.endswith("PREMIUM") else "standard",
            "last_login": "2024-01-15T10:30:00Z"
        }
    }
    
    if include_billing:
        status_data["customer_status"]["billing_info"] = {
            "subscription_status": "active",
            "next_billing_date": "2024-02-15",
            "outstanding_balance": 0.0
        }
    
    return status_data


def escalate_to_human(
    ticket_id: str,
    reason: str,
    urgency: str = "medium",
    department: str = "general_support"
) -> Dict[str, Any]:
    """
    Escalate ticket to human agent (mock implementation for testing)
    
    Args:
        ticket_id: Ticket identifier
        reason: Reason for escalation
        urgency: Escalation urgency (low, medium, high)
        department: Target department
        
    Returns:
        Escalation result dictionary
    """
    # Generate mock escalation ID
    escalation_id = f"ESC-{datetime.now().strftime('%Y%m%d')}-{hash(ticket_id) % 10000:04d}"
    
    return {
        "escalation_result": {
            "ticket_id": ticket_id,
            "escalation_id": escalation_id,
            "status": "escalated",
            "urgency": urgency,
            "department": department,
            "reason": reason,
            "escalated_at": datetime.now().isoformat()
        }
    }


# Mock MCP server class for testing compatibility
class MockMCPServer:
    """Mock MCP server for testing when MCP library is not available"""
    
    def __init__(self, name: str):
        self.name = name
        self.tools = {}
    
    def tool(self):
        """Mock tool decorator"""
        def decorator(func):
            self.tools[func.__name__] = func
            return func
        return decorator


# Create a mock MCP server instance for testing
mcp = MockMCPServer("customer-support-mcp-server")

# Register tools with the MCP server for testing
@mcp.tool()
def validate_ticket(ticket_id: str, customer_id: str, message: str, priority: str = "medium") -> Dict[str, Any]:
    """MCP tool wrapper for validate_ticket_data"""
    return validate_ticket_data(ticket_id, customer_id, message, priority)

@mcp.tool()
def analyze_sentiment(message: str, language: str = "en") -> Dict[str, Any]:
    """MCP tool wrapper for analyze_message_sentiment"""
    sentiment, confidence = analyze_message_sentiment(message)
    return {
        "sentiment_analysis": {
            "sentiment": sentiment,
            "confidence": confidence,
            "language": language
        }
    }

@mcp.tool()
def categorize_issue(message: str) -> Dict[str, Any]:
    """MCP tool wrapper for categorize_message"""
    category, confidence, scores = categorize_message(message)
    return {
        "issue_categorization": {
            "primary_category": category,
            "confidence": confidence,
            "category_scores": scores
        }
    }

@mcp.tool()
def check_customer_status_tool(customer_id: str, include_billing: bool = False) -> Dict[str, Any]:
    """MCP tool wrapper for check_customer_status"""
    return check_customer_status(customer_id, include_billing)

@mcp.tool()
def escalate_to_human_tool(ticket_id: str, reason: str, urgency: str = "medium", department: str = "general_support") -> Dict[str, Any]:
    """MCP tool wrapper for escalate_to_human"""
    return escalate_to_human(ticket_id, reason, urgency, department)


# For backward compatibility, expose the original function names
def analyze_message_sentiment_compat(message: str) -> tuple:
    """Compatibility wrapper for original function signature"""
    return analyze_message_sentiment(message)


def categorize_message_compat(message: str) -> tuple:
    """Compatibility wrapper for original function signature"""
    return categorize_message(message)