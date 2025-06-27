"""Tests for the test runner module"""

import pytest
import asyncio
from unittest.mock import Mock, AsyncMock, patch
import httpx

from test_runner.runner import TestRunner, TestResult
from test_runner.config import TestConfig, TestDefinition, TestCategory


@pytest.fixture
def test_config():
    """Create a test configuration"""
    config = TestConfig()
    config.categories["test"] = TestCategory(
        name="Test Category",
        icon="🧪",
        tests=[
            TestDefinition(name="Port Test", check="port", host="localhost", port=12345),
            TestDefinition(name="HTTP Test", check="http", url="http://localhost:12346/health"),
            TestDefinition(name="Command Test", check="command", command="echo 'test'"),
        ]
    )
    config.save_results = False  # Don't save during tests
    return config


@pytest.mark.asyncio
async def test_check_port_success():
    """Test successful port check"""
    runner = TestRunner()
    
    with patch('asyncio.open_connection', new_callable=AsyncMock) as mock_open:
        mock_writer = AsyncMock()
        mock_open.return_value = (None, mock_writer)
        
        result = await runner.check_port("localhost", 8080)
        assert result is True


@pytest.mark.asyncio
async def test_check_port_failure():
    """Test failed port check"""
    runner = TestRunner()
    
    with patch('asyncio.open_connection', side_effect=ConnectionRefusedError()):
        result = await runner.check_port("localhost", 8080)
        assert result is False


@pytest.mark.asyncio
async def test_check_http_success():
    """Test successful HTTP check"""
    async with TestRunner() as runner:
        mock_response = AsyncMock()
        mock_response.status_code = 200
        
        with patch.object(runner.http_client, 'get', return_value=mock_response):
            success, status_code, error = await runner.check_http("http://localhost:8080/health")
            
            assert success is True
            assert status_code == 200
            assert error is None


@pytest.mark.asyncio
async def test_check_http_failure():
    """Test failed HTTP check"""
    async with TestRunner() as runner:
        with patch.object(runner.http_client, 'get', side_effect=httpx.ConnectError("Connection refused")):
            success, status_code, error = await runner.check_http("http://localhost:8080/health")
            
            assert success is False
            assert status_code is None
            assert error == "Connection refused"


@pytest.mark.asyncio
async def test_check_command_success():
    """Test successful command check"""
    runner = TestRunner()
    
    mock_proc = AsyncMock()
    mock_proc.returncode = 0
    mock_proc.communicate.return_value = (b"output", b"")
    
    with patch('asyncio.create_subprocess_shell', return_value=mock_proc):
        success, exit_code, error = await runner.check_command("echo test")
        
        assert success is True
        assert exit_code == 0
        assert error == ""


@pytest.mark.asyncio
async def test_check_command_failure():
    """Test failed command check"""
    runner = TestRunner()
    
    mock_proc = AsyncMock()
    mock_proc.returncode = 1
    mock_proc.communicate.return_value = (b"", b"error message")
    
    with patch('asyncio.create_subprocess_shell', return_value=mock_proc):
        success, exit_code, error = await runner.check_command("false")
        
        assert success is False
        assert exit_code == 1
        assert error == "error message"


@pytest.mark.asyncio
async def test_run_test_port(test_config):
    """Test running a port test"""
    runner = TestRunner(test_config)
    test = test_config.categories["test"].tests[0]  # Port test
    
    with patch.object(runner, 'check_port', return_value=True) as mock_check:
        result = await runner.run_test(test)
        
        assert result.name == "Port Test"
        assert result.status == "passed"
        assert result.details == "localhost:12345"
        assert result.duration > 0
        mock_check.assert_called_once_with("localhost", 12345, 5.0)


@pytest.mark.asyncio
async def test_run_test_http(test_config):
    """Test running an HTTP test"""
    async with TestRunner(test_config) as runner:
        test = test_config.categories["test"].tests[1]  # HTTP test
        
        with patch.object(runner, 'check_http', return_value=(True, 200, None)) as mock_check:
            result = await runner.run_test(test)
            
            assert result.name == "HTTP Test"
            assert result.status == "passed"
            assert result.details == "HTTP 200"
            assert result.duration > 0
            mock_check.assert_called_once_with("http://localhost:12346/health", 5.0)


@pytest.mark.asyncio
async def test_run_test_command(test_config):
    """Test running a command test"""
    runner = TestRunner(test_config)
    test = test_config.categories["test"].tests[2]  # Command test
    
    with patch.object(runner, 'check_command', return_value=(True, 0, "")) as mock_check:
        result = await runner.run_test(test)
        
        assert result.name == "Command Test"
        assert result.status == "passed"
        assert result.details == "Exit code: 0"
        assert result.duration > 0
        mock_check.assert_called_once_with("echo 'test'", 5.0)


@pytest.mark.asyncio
async def test_run_test_with_retry(test_config):
    """Test running a test with retry logic"""
    test_config.retry_failed = True
    runner = TestRunner(test_config)
    
    test = TestDefinition(
        name="Retry Test",
        check="port",
        host="localhost",
        port=12345,
        retry_count=3,
        retry_delay=0.1
    )
    
    # First two attempts fail, third succeeds
    with patch.object(runner, 'check_port', side_effect=[False, False, True]) as mock_check:
        result = await runner.run_test(test)
        
        assert result.status == "passed"
        assert result.attempts == 3
        assert mock_check.call_count == 3


@pytest.mark.asyncio
async def test_run_category(test_config):
    """Test running a category of tests"""
    async with TestRunner(test_config) as runner:
        with patch.object(runner, 'check_port', return_value=True), \
             patch.object(runner, 'check_http', return_value=(True, 200, None)), \
             patch.object(runner, 'check_command', return_value=(True, 0, "")):
            
            results = await runner.run_category("test", test_config.categories["test"])
            
            assert len(results) == 3
            assert all(r.status == "passed" for r in results)
            assert runner.results["total"] == 3
            assert runner.results["passed"] == 3
            assert runner.results["failed"] == 0


@pytest.mark.asyncio
async def test_runner_context_manager():
    """Test runner as async context manager"""
    async with TestRunner() as runner:
        assert runner.http_client is not None
    
    # After exiting, http_client should be closed
    assert runner.http_client is not None  # Still exists but is closed