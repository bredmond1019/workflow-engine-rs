"""Tests for configuration module"""

import pytest
from pathlib import Path
import tempfile
import yaml

from test_runner.config import TestConfig, TestDefinition, TestCategory


def test_test_definition():
    """Test TestDefinition creation"""
    test = TestDefinition(
        name="Test Service",
        check="http",
        url="http://localhost:8080/health"
    )
    
    assert test.name == "Test Service"
    assert test.check == "http"
    assert test.url == "http://localhost:8080/health"
    assert test.timeout == 5.0  # default
    assert test.retry_count == 1  # default


def test_test_category():
    """Test TestCategory creation"""
    tests = [
        TestDefinition(name="Service 1", check="port", host="localhost", port=8080),
        TestDefinition(name="Service 2", check="http", url="http://localhost:8081/health"),
    ]
    
    category = TestCategory(
        name="Test Category",
        icon="🧪",
        tests=tests
    )
    
    assert category.name == "Test Category"
    assert category.icon == "🧪"
    assert len(category.tests) == 2
    assert category.enabled is True


def test_test_config_default():
    """Test default configuration"""
    config = TestConfig.default()
    
    assert len(config.categories) > 0
    assert "infrastructure" in config.categories
    assert "backend" in config.categories
    assert config.save_results is True
    assert config.rich_output is True


def test_test_config_from_yaml():
    """Test loading configuration from YAML"""
    yaml_content = """
test_categories:
  custom:
    name: Custom Tests
    icon: 🔧
    enabled: true
    tests:
      - name: Custom Service
        check: http
        url: http://localhost:9999/health
        timeout: 10.0
        retry_count: 3

settings:
  output_dir: /tmp/test-output
  save_results: false
  rich_output: false
  parallel_execution: false
  max_workers: 5
  default_timeout: 3.0
  retry_failed: false
"""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.yaml', delete=False) as f:
        f.write(yaml_content)
        f.flush()
        
        config = TestConfig.from_yaml(Path(f.name))
    
    assert "custom" in config.categories
    assert config.categories["custom"].name == "Custom Tests"
    assert config.categories["custom"].icon == "🔧"
    assert len(config.categories["custom"].tests) == 1
    
    test = config.categories["custom"].tests[0]
    assert test.name == "Custom Service"
    assert test.timeout == 10.0
    assert test.retry_count == 3
    
    assert config.output_dir == Path("/tmp/test-output")
    assert config.save_results is False
    assert config.rich_output is False
    assert config.parallel_execution is False
    assert config.max_workers == 5
    assert config.default_timeout == 3.0
    assert config.retry_failed is False
    
    # Clean up
    Path(f.name).unlink()