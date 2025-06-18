#!/usr/bin/env python3
"""
Customer Support MCP Server using the MCP Python SDK

This server provides customer support tools accessible through the MCP protocol,
allowing AI agents to validate tickets, analyze sentiment, categorize issues,
check customer status, and escalate tickets to human agents.
"""

from datetime import datetime
from typing import Optional, List, Dict, Any
from mcp.server.fastmcp import FastMCP

# Initialize the MCP server
mcp = FastMCP("customer-support-mcp-server")

# Simple sentiment analysis helper
def analyze_message_sentiment(message: str) -> tuple[str, float]:
    """Analyze sentiment of a message"""
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

# Categorization helper
def categorize_message(message: str) -> tuple[str, float, Dict[str, int]]:
    """Categorize a support message"""
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
    
    primary_category = max(category_scores, key=category_scores.get) if category_scores else "general"
    confidence = min(0.9, category_scores.get(primary_category, 0) * 0.2 + 0.3)
    
    return primary_category, confidence, category_scores

@mcp.tool()
def validate_ticket(
    ticket_id: str,
    customer_id: str,
    message: str,
    priority: str = "medium"
) -> Dict[str, Any]:
    """
    Validates customer support ticket data for completeness and format
    
    Args:
        ticket_id: Unique ticket identifier
        customer_id: Customer identifier
        message: Customer message content
        priority: Ticket priority (low, medium, high, urgent)
    
    Returns:
        Validation result with status and any missing fields
    """
    if priority not in ["low", "medium", "high", "urgent"]:
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

@mcp.tool()
def analyze_sentiment(
    message: str,
    language: str = "en"
) -> Dict[str, Any]:
    """
    Analyzes the sentiment of customer messages
    
    Args:
        message: Customer message to analyze
        language: Language code (optional, defaults to 'en')
    
    Returns:
        Sentiment analysis results including sentiment, confidence, and metadata
    """
    sentiment, confidence = analyze_message_sentiment(message)
    
    return {
        "sentiment_analysis": {
            "sentiment": sentiment,
            "confidence": confidence,
            "language": language,
            "message_length": len(message),
            "timestamp": datetime.now().isoformat()
        }
    }

@mcp.tool()
def categorize_issue(
    message: str,
    ticket_history: Optional[List[str]] = None
) -> Dict[str, Any]:
    """
    Categorizes customer issues into predefined categories
    
    Args:
        message: Customer message
        ticket_history: Previous interactions (optional)
    
    Returns:
        Issue categorization with primary category and confidence
    """
    primary_category, confidence, all_scores = categorize_message(message)
    
    return {
        "issue_categorization": {
            "primary_category": primary_category,
            "confidence": confidence,
            "all_scores": all_scores,
            "timestamp": datetime.now().isoformat()
        }
    }

@mcp.tool()
def check_customer_status(
    customer_id: str,
    include_billing: bool = True
) -> Dict[str, Any]:
    """
    Checks customer account status and subscription information
    
    Args:
        customer_id: Customer identifier
        include_billing: Include billing information (defaults to True)
    
    Returns:
        Customer status information including account details
    """
    # Mock customer data
    customer_status = {
        "customer_id": customer_id,
        "account_status": "active",
        "subscription_tier": "premium",
        "member_since": "2023-01-15",
        "last_activity": "2024-12-02T10:30:00Z",
        "support_priority": "standard"
    }
    
    if include_billing:
        customer_status["billing_info"] = {
            "payment_method": "credit_card_ending_1234",
            "next_billing_date": "2024-12-15",
            "outstanding_balance": 0.00,
            "last_payment": "2024-11-15"
        }
    
    customer_status["timestamp"] = datetime.now().isoformat()
    return {"customer_status": customer_status}

@mcp.tool()
def escalate_to_human(
    ticket_id: str,
    reason: str,
    urgency: str = "medium",
    department: Optional[str] = None
) -> Dict[str, Any]:
    """
    Escalates ticket to human agent when needed
    
    Args:
        ticket_id: Ticket to escalate
        reason: Escalation reason
        urgency: Escalation urgency (low, medium, high, critical)
        department: Target department (optional)
    
    Returns:
        Escalation details including ID and estimated response time
    """
    if urgency not in ["low", "medium", "high", "critical"]:
        urgency = "medium"
    
    if department is None:
        department = "general_support"
    
    escalation_id = f"ESC-{ticket_id}-{datetime.now().strftime('%Y%m%d%H%M%S')}"
    
    return {
        "escalation_result": {
            "escalation_id": escalation_id,
            "ticket_id": ticket_id,
            "reason": reason,
            "urgency": urgency,
            "assigned_department": department,
            "estimated_response_time": "2-4 hours" if urgency in ["high", "critical"] else "24-48 hours",
            "status": "escalated",
            "timestamp": datetime.now().isoformat()
        }
    }

if __name__ == "__main__":
    # Run the server
    # This will start the server and handle all MCP protocol communication
    # Use: mcp run customer_support_server.py
    # Or for development: mcp dev customer_support_server.py
    mcp.run()