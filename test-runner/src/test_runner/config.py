"""
Test runner configuration and test definitions
"""

from dataclasses import dataclass, field
from typing import Dict, List, Optional
import os
from pathlib import Path
import yaml


@dataclass
class TestDefinition:
    """Single test definition"""
    name: str
    check: str  # "port", "http", "command"
    host: Optional[str] = None
    port: Optional[int] = None
    url: Optional[str] = None
    command: Optional[str] = None
    timeout: float = 5.0
    retry_count: int = 1
    retry_delay: float = 1.0


@dataclass
class TestCategory:
    """Test category definition"""
    name: str
    icon: str
    tests: List[TestDefinition]
    enabled: bool = True


@dataclass
class TestConfig:
    """Test runner configuration"""
    categories: Dict[str, TestCategory] = field(default_factory=dict)
    output_dir: Path = Path("test-reports")
    save_results: bool = True
    rich_output: bool = True
    parallel_execution: bool = True
    max_workers: int = 10
    default_timeout: float = 5.0
    retry_failed: bool = True
    
    @classmethod
    def from_yaml(cls, path: Path) -> "TestConfig":
        """Load configuration from YAML file"""
        with open(path) as f:
            data = yaml.safe_load(f)
        
        config = cls()
        
        # Load test categories
        for cat_id, cat_data in data.get("test_categories", {}).items():
            tests = []
            for test_data in cat_data.get("tests", []):
                test = TestDefinition(**test_data)
                tests.append(test)
            
            category = TestCategory(
                name=cat_data["name"],
                icon=cat_data.get("icon", "📋"),
                tests=tests,
                enabled=cat_data.get("enabled", True)
            )
            config.categories[cat_id] = category
        
        # Load global settings
        settings = data.get("settings", {})
        config.output_dir = Path(settings.get("output_dir", config.output_dir))
        config.save_results = settings.get("save_results", config.save_results)
        config.rich_output = settings.get("rich_output", config.rich_output)
        config.parallel_execution = settings.get("parallel_execution", config.parallel_execution)
        config.max_workers = settings.get("max_workers", config.max_workers)
        config.default_timeout = settings.get("default_timeout", config.default_timeout)
        config.retry_failed = settings.get("retry_failed", config.retry_failed)
        
        return config
    
    @classmethod
    def default(cls) -> "TestConfig":
        """Create default configuration"""
        return cls(categories=DEFAULT_TEST_CATEGORIES)


# Default test categories (migrated from the original script)
DEFAULT_TEST_CATEGORIES = {
    "infrastructure": TestCategory(
        name="Infrastructure",
        icon="🏗️",
        tests=[
            TestDefinition(name="PostgreSQL", check="port", host="localhost", port=5432),
            TestDefinition(name="Redis", check="port", host="localhost", port=6379),
        ]
    ),
    "backend": TestCategory(
        name="Backend Services",
        icon="🚀",
        tests=[
            TestDefinition(name="Main API", check="http", url="http://localhost:8080/health"),
            TestDefinition(name="GraphQL Gateway", check="http", 
                          url="http://localhost:4000/.well-known/apollo/server-health"),
            TestDefinition(name="Swagger UI", check="http", url="http://localhost:8080/swagger-ui/"),
        ]
    ),
    "mcp": TestCategory(
        name="MCP Servers",
        icon="🔌",
        tests=[
            TestDefinition(name="HelpScout MCP", check="http", url="http://localhost:8001/health"),
            TestDefinition(name="Notion MCP", check="http", url="http://localhost:8002/health"),
            TestDefinition(name="Slack MCP", check="http", url="http://localhost:8003/health"),
        ]
    ),
    "microservices": TestCategory(
        name="Microservices",
        icon="🎯",
        tests=[
            TestDefinition(name="Content Processing", check="http", url="http://localhost:8082/health"),
            TestDefinition(name="Knowledge Graph", check="http", url="http://localhost:8083/health"),
            TestDefinition(name="Realtime Communication", check="http", url="http://localhost:8084/health"),
        ]
    ),
    "frontend": TestCategory(
        name="Frontend",
        icon="🎨",
        tests=[
            TestDefinition(name="Vite Dev Server", check="http", url="http://localhost:5173"),
        ]
    ),
    "monitoring": TestCategory(
        name="Monitoring",
        icon="📊",
        tests=[
            TestDefinition(name="Prometheus", check="http", url="http://localhost:9090/-/ready"),
            TestDefinition(name="Grafana", check="http", url="http://localhost:3000/api/health"),
            TestDefinition(name="Jaeger", check="http", url="http://localhost:16686/"),
        ]
    )
}

# Convenience export
TEST_CATEGORIES = DEFAULT_TEST_CATEGORIES