"""
Command-line interface for the test runner
"""

import asyncio
import sys
from pathlib import Path
import click
from dotenv import load_dotenv

from .runner import TestRunner
from .config import TestConfig


def get_version():
    """Get package version"""
    try:
        from . import __version__
        return __version__
    except ImportError:
        return "unknown"


@click.command()
@click.option(
    "--config", "-c",
    type=click.Path(exists=True, path_type=Path),
    help="Path to configuration file (YAML)"
)
@click.option(
    "--output-dir", "-o",
    type=click.Path(path_type=Path),
    help="Directory to save test results"
)
@click.option(
    "--no-save",
    is_flag=True,
    help="Don't save test results to file"
)
@click.option(
    "--no-rich",
    is_flag=True,
    help="Disable rich terminal output"
)
@click.option(
    "--sequential",
    is_flag=True,
    help="Run tests sequentially instead of in parallel"
)
@click.option(
    "--category",
    multiple=True,
    help="Run only specific test categories (can be used multiple times)"
)
@click.option(
    "--timeout",
    type=float,
    help="Default timeout for all tests in seconds"
)
@click.option(
    "--no-retry",
    is_flag=True,
    help="Disable retry on failed tests"
)
@click.option(
    "--env-file",
    type=click.Path(exists=True, path_type=Path),
    help="Path to .env file with environment variables"
)
@click.version_option(version=get_version())
def main(
    config: Path,
    output_dir: Path,
    no_save: bool,
    no_rich: bool,
    sequential: bool,
    category: tuple,
    timeout: float,
    no_retry: bool,
    env_file: Path
):
    """
    AI Workflow System Test Runner
    
    Run comprehensive tests on all system components and generate reports.
    
    Examples:
    
        # Run with default configuration
        workflow-test
        
        # Run with custom config file
        workflow-test --config test-config.yaml
        
        # Run only specific categories
        workflow-test --category backend --category mcp
        
        # Run with custom output directory
        workflow-test --output-dir ./my-reports
        
        # Run tests sequentially with no retry
        workflow-test --sequential --no-retry
    """
    # Load environment variables
    if env_file:
        load_dotenv(env_file)
    
    # Load or create configuration
    if config:
        test_config = TestConfig.from_yaml(config)
    else:
        test_config = TestConfig.default()
    
    # Apply CLI overrides
    if output_dir:
        test_config.output_dir = output_dir
    
    if no_save:
        test_config.save_results = False
    
    if no_rich:
        test_config.rich_output = False
    
    if sequential:
        test_config.parallel_execution = False
    
    if timeout:
        test_config.default_timeout = timeout
    
    if no_retry:
        test_config.retry_failed = False
    
    # Filter categories if specified
    if category:
        for cat_id in list(test_config.categories.keys()):
            if cat_id not in category:
                test_config.categories[cat_id].enabled = False
    
    # Run the tests
    exit_code = asyncio.run(run_tests(test_config))
    sys.exit(exit_code)


async def run_tests(config: TestConfig) -> int:
    """Run tests with the given configuration"""
    async with TestRunner(config) as runner:
        return await runner.run()


if __name__ == "__main__":
    # Check Python version
    if sys.version_info < (3, 8):
        print("Error: Python 3.8+ required")
        sys.exit(1)
    
    main()