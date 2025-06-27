"""
Core test runner implementation
"""

import asyncio
import json
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import httpx

try:
    from rich.console import Console
    from rich.table import Table
    from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn, TaskProgressColumn
    from rich.panel import Panel
    from rich import box
    RICH_AVAILABLE = True
except ImportError:
    RICH_AVAILABLE = False

from .config import TestConfig, TestDefinition, TestCategory


class TestResult:
    """Test result data"""
    def __init__(self, name: str):
        self.name = name
        self.status = "unknown"  # passed, failed, skipped
        self.duration = 0.0
        self.details = ""
        self.error = None
        self.attempts = 0


class TestRunner:
    """Main test runner class"""
    
    def __init__(self, config: Optional[TestConfig] = None):
        self.config = config or TestConfig.default()
        self.console = Console() if RICH_AVAILABLE and self.config.rich_output else None
        self.http_client = None
        
        self.results = {
            "total": 0,
            "passed": 0,
            "failed": 0,
            "skipped": 0,
            "services": {},
            "start_time": datetime.now().isoformat(),
        }
    
    async def __aenter__(self):
        """Async context manager entry"""
        self.http_client = httpx.AsyncClient(timeout=self.config.default_timeout)
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit"""
        if self.http_client:
            await self.http_client.aclose()
    
    def print_header(self):
        """Print the application header"""
        if self.console:
            header = Panel.fit(
                "[bold purple]AI Workflow Orchestration[/bold purple]\n"
                "[dim]System Test Runner v1.0[/dim]",
                border_style="purple",
                box=box.DOUBLE
            )
            self.console.print(header)
            self.console.print()
        else:
            print("=" * 60)
            print("AI Workflow Orchestration - System Test Runner")
            print("=" * 60)
            print()
    
    async def check_port(self, host: str, port: int, timeout: float = None) -> bool:
        """Check if a port is open"""
        timeout = timeout or self.config.default_timeout
        try:
            _, writer = await asyncio.wait_for(
                asyncio.open_connection(host, port),
                timeout=timeout
            )
            writer.close()
            await writer.wait_closed()
            return True
        except (asyncio.TimeoutError, ConnectionRefusedError, OSError):
            return False
    
    async def check_http(self, url: str, timeout: float = None) -> Tuple[bool, Optional[int], Optional[str]]:
        """Check HTTP endpoint"""
        timeout = timeout or self.config.default_timeout
        try:
            response = await self.http_client.get(url, timeout=timeout)
            return response.status_code == 200, response.status_code, None
        except httpx.TimeoutException:
            return False, None, "Timeout"
        except httpx.ConnectError:
            return False, None, "Connection refused"
        except Exception as e:
            return False, None, str(e)
    
    async def check_command(self, command: str, timeout: float = None) -> Tuple[bool, int, str]:
        """Execute a command and check result"""
        timeout = timeout or self.config.default_timeout
        try:
            proc = await asyncio.create_subprocess_shell(
                command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            stdout, stderr = await asyncio.wait_for(proc.communicate(), timeout=timeout)
            success = proc.returncode == 0
            return success, proc.returncode, stderr.decode() if stderr else ""
        except asyncio.TimeoutError:
            return False, -1, "Command timeout"
        except Exception as e:
            return False, -1, str(e)
    
    async def run_test(self, test: TestDefinition) -> TestResult:
        """Run a single test with retry logic"""
        result = TestResult(test.name)
        
        for attempt in range(test.retry_count):
            result.attempts = attempt + 1
            start_time = time.time()
            
            try:
                if test.check == "port":
                    success = await self.check_port(test.host, test.port, test.timeout)
                    result.status = "passed" if success else "failed"
                    result.details = f"{test.host}:{test.port}"
                
                elif test.check == "http":
                    success, status_code, error = await self.check_http(test.url, test.timeout)
                    result.status = "passed" if success else "failed"
                    if status_code:
                        result.details = f"HTTP {status_code}"
                    else:
                        result.details = error or "No response"
                
                elif test.check == "command":
                    success, exit_code, error = await self.check_command(test.command, test.timeout)
                    result.status = "passed" if success else "failed"
                    result.details = f"Exit code: {exit_code}"
                    if error:
                        result.error = error
                
                else:
                    result.status = "skipped"
                    result.details = f"Unknown check type: {test.check}"
            
            except Exception as e:
                result.status = "failed"
                result.details = "Exception occurred"
                result.error = str(e)
            
            result.duration = time.time() - start_time
            
            # Break if successful or no retry needed
            if result.status == "passed" or not self.config.retry_failed:
                break
            
            # Wait before retry
            if attempt < test.retry_count - 1:
                await asyncio.sleep(test.retry_delay)
        
        return result
    
    async def run_category(self, category_id: str, category: TestCategory, 
                          progress=None, task_id=None) -> List[TestResult]:
        """Run all tests in a category"""
        if not category.enabled:
            return []
        
        results = []
        
        if self.config.parallel_execution:
            # Run tests in parallel
            tasks = []
            for test in category.tests:
                tasks.append(self.run_test(test))
            
            test_results = await asyncio.gather(*tasks, return_exceptions=True)
            
            for i, result in enumerate(test_results):
                if isinstance(result, Exception):
                    # Handle exception case
                    result = TestResult(category.tests[i].name)
                    result.status = "failed"
                    result.details = "Exception during test"
                    result.error = str(result)
                
                results.append(result)
                
                if progress and task_id:
                    progress.update(task_id, advance=1)
        else:
            # Run tests sequentially
            for test in category.tests:
                if progress and task_id:
                    progress.update(task_id, advance=1, description=f"Testing {test.name}...")
                
                result = await self.run_test(test)
                results.append(result)
        
        # Update counters
        for result in results:
            self.results["total"] += 1
            if result.status == "passed":
                self.results["passed"] += 1
            elif result.status == "failed":
                self.results["failed"] += 1
            else:
                self.results["skipped"] += 1
        
        return results
    
    def create_results_table(self, category_id: str, results: List[TestResult]) -> Table:
        """Create a rich table for test results"""
        category = self.config.categories[category_id]
        table = Table(
            title=f"{category.icon} {category.name}",
            box=box.ROUNDED,
            show_header=True,
            header_style="bold"
        )
        
        table.add_column("Service", style="cyan", no_wrap=True)
        table.add_column("Status", justify="center")
        table.add_column("Details", style="dim")
        table.add_column("Time", justify="right", style="dim")
        
        for result in results:
            status_style = {
                "passed": "[green]✓ PASS[/green]",
                "failed": "[red]✗ FAIL[/red]",
                "skipped": "[yellow]⚠ SKIP[/yellow]"
            }.get(result.status, "?")
            
            time_str = f"{result.duration:.2f}s"
            if result.attempts > 1:
                time_str += f" ({result.attempts} attempts)"
            
            table.add_row(
                result.name,
                status_style,
                result.details,
                time_str
            )
        
        return table
    
    async def run_all_tests(self):
        """Run all test categories"""
        if self.console and RICH_AVAILABLE:
            with Progress(
                SpinnerColumn(),
                TextColumn("[progress.description]{task.description}"),
                BarColumn(),
                TaskProgressColumn(),
                console=self.console
            ) as progress:
                # Calculate total tests
                total_tests = sum(
                    len(cat.tests) for cat in self.config.categories.values() 
                    if cat.enabled
                )
                
                # Create main task
                main_task = progress.add_task(
                    "[cyan]Running tests...", 
                    total=total_tests
                )
                
                # Run each category
                for category_id, category in self.config.categories.items():
                    if not category.enabled:
                        continue
                    
                    results = await self.run_category(
                        category_id, category, progress, main_task
                    )
                    self.results["services"][category_id] = [
                        {
                            "name": r.name,
                            "status": r.status,
                            "duration": r.duration,
                            "details": r.details,
                            "error": r.error,
                            "attempts": r.attempts
                        } for r in results
                    ]
                    
                    # Show results table
                    table = self.create_results_table(category_id, results)
                    self.console.print(table)
                    self.console.print()
        
        else:
            # Basic output without rich
            for category_id, category in self.config.categories.items():
                if not category.enabled:
                    continue
                
                print(f"\n{category.icon} {category.name}")
                print("-" * 40)
                
                results = await self.run_category(category_id, category)
                self.results["services"][category_id] = [
                    {
                        "name": r.name,
                        "status": r.status,
                        "duration": r.duration,
                        "details": r.details,
                        "error": r.error,
                        "attempts": r.attempts
                    } for r in results
                ]
                
                for result in results:
                    status = "✓" if result.status == "passed" else "✗"
                    print(f"  {status} {result.name}: {result.details}")
    
    def show_summary(self):
        """Display test summary"""
        duration = (datetime.now() - datetime.fromisoformat(self.results["start_time"])).total_seconds()
        
        if self.console and RICH_AVAILABLE:
            # Create summary panel
            summary_text = f"""
[bold]Test Results Summary[/bold]

[green]✓ Passed:[/green]  {self.results['passed']}
[red]✗ Failed:[/red]  {self.results['failed']}
[yellow]⚠ Skipped:[/yellow] {self.results['skipped']}
──────────────
[bold]Total:[/bold]     {self.results['total']}

[dim]Duration: {duration:.1f}s[/dim]
"""
            
            if self.results['failed'] == 0 and self.results['total'] > 0:
                summary_text += "\n[bold green]✨ All systems operational![/bold green]"
                border_style = "green"
            elif self.results['failed'] <= 3:
                summary_text += "\n[bold yellow]⚠️  Minor issues detected[/bold yellow]"
                border_style = "yellow"
            else:
                summary_text += "\n[bold red]❌ Multiple failures detected[/bold red]"
                border_style = "red"
            
            summary = Panel(
                summary_text.strip(),
                title="Summary",
                border_style=border_style,
                box=box.DOUBLE
            )
            self.console.print(summary)
        
        else:
            # Basic summary
            print("\n" + "=" * 40)
            print("SUMMARY")
            print("=" * 40)
            print(f"Passed:  {self.results['passed']}")
            print(f"Failed:  {self.results['failed']}")
            print(f"Skipped: {self.results['skipped']}")
            print(f"Total:   {self.results['total']}")
            print(f"\nDuration: {duration:.1f}s")
    
    def save_results(self):
        """Save test results to file"""
        if not self.config.save_results:
            return
        
        self.config.output_dir.mkdir(exist_ok=True)
        
        timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        json_file = self.config.output_dir / f"test-results-{timestamp}.json"
        
        # Add end time and duration
        self.results["end_time"] = datetime.now().isoformat()
        self.results["duration"] = (
            datetime.fromisoformat(self.results["end_time"]) - 
            datetime.fromisoformat(self.results["start_time"])
        ).total_seconds()
        
        with open(json_file, 'w') as f:
            json.dump(self.results, f, indent=2)
        
        if self.console:
            self.console.print(f"\n[dim]Results saved to: {json_file}[/dim]")
        else:
            print(f"\nResults saved to: {json_file}")
    
    async def run(self) -> int:
        """Main execution function"""
        self.print_header()
        await self.run_all_tests()
        self.show_summary()
        self.save_results()
        
        # Return exit code based on failures
        return 0 if self.results['failed'] == 0 else 1