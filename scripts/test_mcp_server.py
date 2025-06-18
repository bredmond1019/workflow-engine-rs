#!/usr/bin/env python3
"""
Test script for the Customer Support MCP Server

This script tests the MCP server by sending various requests and verifying responses.
"""

import asyncio
import json
import subprocess
import sys
import logging
from typing import Dict, Any, Optional

class MCPServerTester:
    def __init__(self, server_script: str):
        self.server_script = server_script
        self.process = None

    async def start_server(self):
        """Start the MCP server process"""
        self.process = subprocess.Popen(
            [sys.executable, self.server_script],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=0
        )
        logging.info("MCP Server started")

    async def stop_server(self):
        """Stop the MCP server process"""
        if self.process:
            self.process.terminate()
            await asyncio.sleep(0.1)
            if self.process.poll() is None:
                self.process.kill()
            self.process = None
            logging.info("MCP Server stopped")

    async def send_request(self, request: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Send a request to the MCP server and get response"""
        if not self.process:
            raise RuntimeError("Server not started")

        request_json = json.dumps(request) + "\n"
        self.process.stdin.write(request_json)
        self.process.stdin.flush()

        # Read response (if expected)
        if request.get("method") != "notifications/initialized":
            response_line = self.process.stdout.readline()
            if response_line:
                return json.loads(response_line.strip())
        
        return None

    async def test_initialize(self):
        """Test initialization"""
        logging.info("Testing initialization...")
        
        request = {
            "method": "initialize",
            "id": "init-1",
            "params": {
                "protocol_version": "2024-11-05",
                "capabilities": {
                    "roots": None,
                    "sampling": None
                },
                "client_info": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        }
        
        response = await self.send_request(request)
        assert response is not None, "No response to initialize"
        assert response["id"] == "init-1", f"Wrong ID in response: {response}"
        assert "result" in response, f"No result in initialize response: {response}"
        
        result = response["result"]
        assert "server_info" in result, f"No server_info in result: {result}"
        assert result["server_info"]["name"] == "customer-support-mcp-server"
        
        logging.info("✓ Initialization test passed")
        
        # Send initialized notification
        notification = {
            "method": "notifications/initialized"
        }
        await self.send_request(notification)

    async def test_list_tools(self):
        """Test listing tools"""
        logging.info("Testing list tools...")
        
        request = {
            "method": "tools/list",
            "id": "list-1"
        }
        
        response = await self.send_request(request)
        assert response is not None, "No response to list tools"
        assert response["id"] == "list-1"
        assert "result" in response
        
        result = response["result"]
        assert "tools" in result
        tools = result["tools"]
        assert len(tools) == 5, f"Expected 5 tools, got {len(tools)}"
        
        tool_names = [tool["name"] for tool in tools]
        expected_tools = [
            "validate_ticket", "analyze_sentiment", "categorize_issue", 
            "check_customer_status", "escalate_to_human"
        ]
        
        for expected_tool in expected_tools:
            assert expected_tool in tool_names, f"Missing tool: {expected_tool}"
        
        logging.info("✓ List tools test passed")
        return tools

    async def test_validate_ticket_tool(self):
        """Test validate_ticket tool"""
        logging.info("Testing validate_ticket tool...")
        
        request = {
            "method": "tools/call",
            "id": "call-validate-1",
            "params": {
                "name": "validate_ticket",
                "arguments": {
                    "ticket_id": "TKT-12345",
                    "customer_id": "CUST-67890",
                    "message": "I need help with my billing",
                    "priority": "medium"
                }
            }
        }
        
        response = await self.send_request(request)
        assert response is not None
        assert response["id"] == "call-validate-1"
        assert "result" in response
        
        result = response["result"]
        assert "content" in result
        assert not result.get("is_error", False)
        
        content = result["content"][0]
        assert content["type"] == "text"
        
        # Parse the tool result
        tool_result = json.loads(content["text"])
        assert "validation_result" in tool_result
        validation = tool_result["validation_result"]
        assert validation["is_valid"] == True
        assert validation["ticket_id"] == "TKT-12345"
        
        logging.info("✓ Validate ticket tool test passed")

    async def test_analyze_sentiment_tool(self):
        """Test analyze_sentiment tool"""
        logging.info("Testing analyze_sentiment tool...")
        
        # Test positive sentiment
        request = {
            "method": "tools/call",
            "id": "call-sentiment-1",
            "params": {
                "name": "analyze_sentiment",
                "arguments": {
                    "message": "I am very happy with your excellent service!",
                    "language": "en"
                }
            }
        }
        
        response = await self.send_request(request)
        assert response is not None
        result = response["result"]
        content = json.loads(result["content"][0]["text"])
        
        sentiment_analysis = content["sentiment_analysis"]
        assert sentiment_analysis["sentiment"] == "positive"
        assert sentiment_analysis["confidence"] > 0.5
        
        logging.info("✓ Analyze sentiment tool test passed")

    async def test_categorize_issue_tool(self):
        """Test categorize_issue tool"""
        logging.info("Testing categorize_issue tool...")
        
        request = {
            "method": "tools/call",
            "id": "call-categorize-1",
            "params": {
                "name": "categorize_issue",
                "arguments": {
                    "message": "I can't login to my account and forgot my password"
                }
            }
        }
        
        response = await self.send_request(request)
        assert response is not None
        result = response["result"]
        content = json.loads(result["content"][0]["text"])
        
        categorization = content["issue_categorization"]
        assert categorization["primary_category"] == "account"
        assert categorization["confidence"] > 0.0
        
        logging.info("✓ Categorize issue tool test passed")

    async def test_check_customer_status_tool(self):
        """Test check_customer_status tool"""
        logging.info("Testing check_customer_status tool...")
        
        request = {
            "method": "tools/call",
            "id": "call-status-1",
            "params": {
                "name": "check_customer_status",
                "arguments": {
                    "customer_id": "CUST-12345",
                    "include_billing": True
                }
            }
        }
        
        response = await self.send_request(request)
        assert response is not None
        result = response["result"]
        content = json.loads(result["content"][0]["text"])
        
        customer_status = content["customer_status"]
        assert customer_status["customer_id"] == "CUST-12345"
        assert customer_status["account_status"] == "active"
        assert "billing_info" in customer_status
        
        logging.info("✓ Check customer status tool test passed")

    async def test_escalate_to_human_tool(self):
        """Test escalate_to_human tool"""
        logging.info("Testing escalate_to_human tool...")
        
        request = {
            "method": "tools/call",
            "id": "call-escalate-1",
            "params": {
                "name": "escalate_to_human",
                "arguments": {
                    "ticket_id": "TKT-12345",
                    "reason": "Complex billing dispute requiring manual review",
                    "urgency": "high",
                    "department": "billing_support"
                }
            }
        }
        
        response = await self.send_request(request)
        assert response is not None
        result = response["result"]
        content = json.loads(result["content"][0]["text"])
        
        escalation = content["escalation_result"]
        assert escalation["ticket_id"] == "TKT-12345"
        assert escalation["urgency"] == "high"
        assert escalation["status"] == "escalated"
        assert "escalation_id" in escalation
        
        logging.info("✓ Escalate to human tool test passed")

    async def test_error_handling(self):
        """Test error handling"""
        logging.info("Testing error handling...")
        
        # Test non-existent tool
        request = {
            "method": "tools/call",
            "id": "call-error-1",
            "params": {
                "name": "non_existent_tool",
                "arguments": {}
            }
        }
        
        response = await self.send_request(request)
        assert response is not None
        assert "error" in response
        assert response["error"]["code"] == -32601
        
        logging.info("✓ Error handling test passed")

    async def run_all_tests(self):
        """Run all tests"""
        logging.info("Starting MCP Server tests...")
        
        try:
            await self.start_server()
            
            # Give server time to start
            await asyncio.sleep(0.5)
            
            # Run tests in order
            await self.test_initialize()
            tools = await self.test_list_tools()
            await self.test_validate_ticket_tool()
            await self.test_analyze_sentiment_tool()
            await self.test_categorize_issue_tool()
            await self.test_check_customer_status_tool()
            await self.test_escalate_to_human_tool()
            await self.test_error_handling()
            
            logging.info("🎉 All tests passed!")
            
        except Exception as e:
            logging.error(f"❌ Test failed: {e}")
            raise
        finally:
            await self.stop_server()

async def main():
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s'
    )
    
    server_script = "scripts/customer_support_server.py"
    tester = MCPServerTester(server_script)
    
    try:
        await tester.run_all_tests()
    except Exception as e:
        logging.error(f"Test suite failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main())