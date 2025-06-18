#!/usr/bin/env python3
"""
Multi-Service MCP Server using the MCP Python SDK

This server provides MCP endpoints for Notion, HelpScout, and Slack services
with mock implementations for testing external MCP client nodes.

Usage:
    # Run in development mode
    mcp dev multi_service_mcp_server.py --service notion
    mcp dev multi_service_mcp_server.py --service helpscout
    mcp dev multi_service_mcp_server.py --service slack
    
    # Run in production mode
    mcp run multi_service_mcp_server.py --service notion
"""

import argparse
import sys
from datetime import datetime
from typing import Optional, List, Dict, Any
import uuid
from mcp.server.fastmcp import FastMCP

# Parse command line arguments before creating the server
parser = argparse.ArgumentParser(description="Multi-Service MCP Server")
parser.add_argument("--service", required=True, choices=["notion", "helpscout", "slack"],
                   help="Service type to emulate")
args = parser.parse_args()

# Initialize the MCP server with the service name
mcp = FastMCP(f"{args.service}-mcp-server")

# Service-specific tool implementations
if args.service == "notion":
    @mcp.tool()
    def search_pages(
        query: str,
        page_size: int = 10
    ) -> Dict[str, Any]:
        """
        Search for pages in Notion workspace
        
        Args:
            query: Search query
            page_size: Number of results to return
        
        Returns:
            Search results with matching pages
        """
        results = []
        for i in range(min(page_size, 3)):  # Return up to 3 mock results
            results.append({
                "id": f"page-{uuid.uuid4()}",
                "title": f"Page matching '{query}' - Result {i+1}",
                "url": f"https://notion.so/page-{i+1}",
                "created_time": datetime.now().isoformat(),
                "last_edited_time": datetime.now().isoformat()
            })
        
        return {
            "search_results": {
                "query": query,
                "results": results,
                "total_count": len(results),
                "has_more": False
            }
        }

    @mcp.tool()
    def create_page(
        parent_id: str,
        title: str,
        content: str = "",
        properties: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        """
        Create a new page in Notion
        
        Args:
            parent_id: Parent page or database ID
            title: Page title
            content: Page content
            properties: Page properties
        
        Returns:
            Created page information
        """
        page_id = f"page-{uuid.uuid4()}"
        return {
            "created_page": {
                "id": page_id,
                "title": title,
                "parent_id": parent_id,
                "url": f"https://notion.so/{page_id}",
                "created_time": datetime.now().isoformat()
            }
        }

    @mcp.tool()
    def update_page(
        page_id: str,
        title: Optional[str] = None,
        content: Optional[str] = None,
        properties: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        """
        Update an existing page in Notion
        
        Args:
            page_id: Page ID to update
            title: New page title
            content: New page content
            properties: Updated properties
        
        Returns:
            Updated page information
        """
        return {
            "updated_page": {
                "id": page_id,
                "title": title or "Updated Page",
                "updated_time": datetime.now().isoformat(),
                "url": f"https://notion.so/{page_id}"
            }
        }

    @mcp.tool()
    def get_page(page_id: str) -> Dict[str, Any]:
        """
        Get a specific page by ID
        
        Args:
            page_id: Page ID to retrieve
        
        Returns:
            Page details
        """
        return {
            "page": {
                "id": page_id,
                "title": "Mock Page Title",
                "content": "This is mock page content",
                "created_time": datetime.now().isoformat(),
                "last_edited_time": datetime.now().isoformat(),
                "url": f"https://notion.so/{page_id}"
            }
        }

    @mcp.tool()
    def list_databases() -> Dict[str, Any]:
        """
        List all databases in the workspace
        
        Returns:
            List of available databases
        """
        return {
            "databases": [
                {
                    "id": f"db-{uuid.uuid4()}",
                    "title": "Projects Database",
                    "created_time": datetime.now().isoformat()
                },
                {
                    "id": f"db-{uuid.uuid4()}",
                    "title": "Tasks Database", 
                    "created_time": datetime.now().isoformat()
                }
            ]
        }

    @mcp.tool()
    def query_database(
        database_id: str,
        filter: Optional[Dict[str, Any]] = None,
        sorts: Optional[List[Dict[str, Any]]] = None,
        page_size: int = 100
    ) -> Dict[str, Any]:
        """
        Query a database with filters and sorts
        
        Args:
            database_id: Database ID to query
            filter: Query filter
            sorts: Sort configuration
            page_size: Number of results
        
        Returns:
            Query results from the database
        """
        results = []
        for i in range(min(page_size, 5)):
            results.append({
                "id": f"entry-{uuid.uuid4()}",
                "properties": {
                    "Name": f"Database Entry {i+1}",
                    "Status": "Active",
                    "Created": datetime.now().isoformat()
                }
            })
        
        return {
            "query_results": {
                "database_id": database_id,
                "results": results,
                "has_more": False
            }
        }

elif args.service == "helpscout":
    @mcp.tool()
    def search_articles(
        keywords: str,
        page: int = 1,
        per_page: int = 10,
        collection_id: Optional[str] = None
    ) -> Dict[str, Any]:
        """
        Search knowledge base articles
        
        Args:
            keywords: Search keywords
            page: Page number
            per_page: Results per page
            collection_id: Collection ID to search in
        
        Returns:
            Search results with matching articles
        """
        articles = []
        for i in range(min(per_page, 3)):
            articles.append({
                "id": f"article-{uuid.uuid4()}",
                "name": f"Article about {keywords} - {i+1}",
                "text": f"This article covers information about {keywords}...",
                "status": "published",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat()
            })
        
        return {
            "search_results": {
                "articles": articles,
                "page": page,
                "total": len(articles)
            }
        }

    @mcp.tool()
    def get_article(article_id: str) -> Dict[str, Any]:
        """
        Get a specific article by ID
        
        Args:
            article_id: Article ID
        
        Returns:
            Article details
        """
        return {
            "article": {
                "id": article_id,
                "name": "Mock Article",
                "text": "This is the full content of the mock article...",
                "status": "published",
                "collection_id": f"collection-{uuid.uuid4()}",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat()
            }
        }

    @mcp.tool()
    def list_collections(
        page: int = 1,
        per_page: int = 10
    ) -> Dict[str, Any]:
        """
        List all knowledge base collections
        
        Args:
            page: Page number
            per_page: Results per page
        
        Returns:
            List of collections
        """
        return {
            "collections": [
                {
                    "id": f"collection-{uuid.uuid4()}",
                    "name": "Getting Started",
                    "article_count": 12
                },
                {
                    "id": f"collection-{uuid.uuid4()}",
                    "name": "Troubleshooting",
                    "article_count": 8
                }
            ]
        }

    @mcp.tool()
    def create_article(
        collection_id: str,
        name: str,
        text: str,
        status: str = "published",
        tags: Optional[List[str]] = None
    ) -> Dict[str, Any]:
        """
        Create a new knowledge base article
        
        Args:
            collection_id: Collection ID
            name: Article title
            text: Article content
            status: Article status
            tags: Article tags
        
        Returns:
            Created article information
        """
        article_id = f"article-{uuid.uuid4()}"
        return {
            "created_article": {
                "id": article_id,
                "collection_id": collection_id,
                "name": name,
                "status": status,
                "tags": tags or [],
                "created_at": datetime.now().isoformat()
            }
        }

    @mcp.tool()
    def search_conversations(
        query: str,
        mailbox_id: Optional[str] = None,
        status: Optional[str] = None,
        page: int = 1
    ) -> Dict[str, Any]:
        """
        Search customer conversations
        
        Args:
            query: Search query
            mailbox_id: Mailbox ID
            status: Conversation status
            page: Page number
        
        Returns:
            Search results with matching conversations
        """
        conversations = []
        for i in range(3):
            conversations.append({
                "id": f"conv-{uuid.uuid4()}",
                "subject": f"Conversation matching '{query}' - {i+1}",
                "status": status or "active",
                "mailbox_id": mailbox_id or f"mailbox-{uuid.uuid4()}",
                "created_at": datetime.now().isoformat()
            })
        
        return {
            "search_results": {
                "conversations": conversations,
                "page": page,
                "total": len(conversations)
            }
        }

    @mcp.tool()
    def get_conversation(conversation_id: str) -> Dict[str, Any]:
        """
        Get a specific conversation by ID
        
        Args:
            conversation_id: Conversation ID
        
        Returns:
            Conversation details
        """
        return {
            "conversation": {
                "id": conversation_id,
                "subject": "Customer inquiry",
                "status": "active",
                "customer": {
                    "id": f"customer-{uuid.uuid4()}",
                    "email": "customer@example.com"
                },
                "created_at": datetime.now().isoformat()
            }
        }

elif args.service == "slack":
    @mcp.tool()
    def send_message(
        channel: str,
        text: str,
        thread_ts: Optional[str] = None,
        blocks: Optional[List[Dict[str, Any]]] = None,
        attachments: Optional[List[Dict[str, Any]]] = None
    ) -> Dict[str, Any]:
        """
        Send a message to a Slack channel or user
        
        Args:
            channel: Channel ID or name
            text: Message text
            thread_ts: Thread timestamp for replies
            blocks: Rich message blocks
            attachments: Message attachments
        
        Returns:
            Sent message information
        """
        return {
            "message": {
                "ts": str(int(datetime.now().timestamp())),
                "channel": channel,
                "text": text,
                "user": "bot_user",
                "thread_ts": thread_ts,
                "timestamp": datetime.now().isoformat()
            }
        }

    @mcp.tool()
    def list_channels(
        exclude_archived: bool = True,
        limit: int = 100,
        cursor: Optional[str] = None
    ) -> Dict[str, Any]:
        """
        List all channels in the workspace
        
        Args:
            exclude_archived: Exclude archived channels
            limit: Number of channels to return
            cursor: Pagination cursor
        
        Returns:
            List of channels
        """
        return {
            "channels": [
                {
                    "id": "C1234567890",
                    "name": "general",
                    "is_channel": True,
                    "is_private": False,
                    "is_archived": False
                },
                {
                    "id": "C1234567891", 
                    "name": "random",
                    "is_channel": True,
                    "is_private": False,
                    "is_archived": False
                }
            ]
        }

    @mcp.tool()
    def get_channel_info(channel: str) -> Dict[str, Any]:
        """
        Get information about a specific channel
        
        Args:
            channel: Channel ID or name
        
        Returns:
            Channel information
        """
        return {
            "channel": {
                "id": channel if channel.startswith("C") else "C1234567890",
                "name": channel if not channel.startswith("C") else "general",
                "created": int(datetime.now().timestamp()),
                "creator": "U1234567890",
                "is_channel": True,
                "is_private": False,
                "is_archived": False,
                "topic": {
                    "value": "Company-wide announcements"
                },
                "purpose": {
                    "value": "This channel is for general discussion"
                }
            }
        }

    @mcp.tool()
    def list_users(
        limit: int = 100,
        cursor: Optional[str] = None,
        include_locale: bool = False
    ) -> Dict[str, Any]:
        """
        List users in the workspace
        
        Args:
            limit: Number of users to return
            cursor: Pagination cursor
            include_locale: Include locale info
        
        Returns:
            List of users
        """
        users = [
            {
                "id": "U1234567890",
                "name": "john.doe",
                "real_name": "John Doe",
                "profile": {
                    "email": "john@example.com"
                }
            },
            {
                "id": "U1234567891",
                "name": "jane.smith", 
                "real_name": "Jane Smith",
                "profile": {
                    "email": "jane@example.com"
                }
            }
        ]
        
        if include_locale:
            for user in users:
                user["locale"] = "en_US"
        
        return {"users": users}

    @mcp.tool()
    def get_user_info(user: str) -> Dict[str, Any]:
        """
        Get information about a specific user
        
        Args:
            user: User ID
        
        Returns:
            User information
        """
        return {
            "user": {
                "id": user,
                "name": "john.doe",
                "real_name": "John Doe",
                "tz": "America/New_York",
                "profile": {
                    "email": "john@example.com",
                    "phone": "+1234567890",
                    "title": "Software Engineer"
                },
                "is_admin": False,
                "is_bot": False
            }
        }

    @mcp.tool()
    def get_channel_history(
        channel: str,
        limit: int = 100,
        oldest: Optional[str] = None,
        latest: Optional[str] = None
    ) -> Dict[str, Any]:
        """
        Get message history for a channel
        
        Args:
            channel: Channel ID
            limit: Number of messages
            oldest: Oldest timestamp
            latest: Latest timestamp
        
        Returns:
            Channel message history
        """
        messages = []
        for i in range(min(limit, 5)):
            messages.append({
                "type": "message",
                "ts": str(int(datetime.now().timestamp()) - i * 3600),
                "user": f"U123456789{i}",
                "text": f"Historical message {i+1}",
                "channel": channel
            })
        
        return {
            "messages": messages,
            "has_more": limit > 5
        }

if __name__ == "__main__":
    # Run the server
    # This will start the server and handle all MCP protocol communication
    # Use: mcp run multi_service_mcp_server.py --service [notion|helpscout|slack]
    # Or for development: mcp dev multi_service_mcp_server.py --service [notion|helpscout|slack]
    mcp.run()