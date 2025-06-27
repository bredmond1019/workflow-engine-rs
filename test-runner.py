#!/usr/bin/env python3
"""
AI Workflow System - Advanced Test Runner
Provides rich terminal output and comprehensive test orchestration
"""

import asyncio
import json
import os
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# Check if rich is installed
try:
    from rich.console import Console
    from rich.table import Table
    from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn, TaskProgressColumn
    from rich.panel import Panel
    from rich.layout import Layout
    from rich.live import Live
    from rich.tree import Tree
    from rich.syntax import Syntax
    from rich import box
    RICH_AVAILABLE = True
except ImportError:
    RICH_AVAILABLE = False
    print("⚠️  Rich library not installed. Install with: pip install rich")
    print("   Falling back to basic output mode.\n")

# Test categories
TEST_CATEGORIES = {
    "infrastructure": {
        "name": "Infrastructure",
        "icon": "🏗️",
        "tests": [
            {"name": "PostgreSQL", "check": "port", "host": "localhost", "port": 5432},
            {"name": "Redis", "check": "port", "host": "localhost", "port": 6379},
        ]
    },
    "backend": {
        "name": "Backend Services",
        "icon": "🚀",
        "tests": [
            {"name": "Main API", "check": "http", "url": "http://localhost:8080/health"},
            {"name": "GraphQL Gateway", "check": "http", "url": "http://localhost:4000/.well-known/apollo/server-health"},
            {"name": "Swagger UI", "check": "http", "url": "http://localhost:8080/swagger-ui/"},
        ]
    },
    "mcp": {
        "name": "MCP Servers",
        "icon": "🔌",
        "tests": [
            {"name": "HelpScout MCP", "check": "http", "url": "http://localhost:8001/health"},
            {"name": "Notion MCP", "check": "http", "url": "http://localhost:8002/health"},
            {"name": "Slack MCP", "check": "http", "url": "http://localhost:8003/health"},
        ]
    },
    "microservices": {
        "name": "Microservices",
        "icon": "🎯",
        "tests": [
            {"name": "Content Processing", "check": "http", "url": "http://localhost:8082/health"},
            {"name": "Knowledge Graph", "check": "http", "url": "http://localhost:8083/health"},
            {"name": "Realtime Communication", "check": "http", "url": "http://localhost:8084/health"},
        ]
    },
    "frontend": {
        "name": "Frontend",
        "icon": "🎨",
        "tests": [
            {"name": "Vite Dev Server", "check": "http", "url": "http://localhost:5173"},
        ]
    },
    "monitoring": {
        "name": "Monitoring",
        "icon": "📊",
        "tests": [
            {"name": "Prometheus", "check": "http", "url": "http://localhost:9090/-/ready"},
            {"name": "Grafana", "check": "http", "url": "http://localhost:3000/api/health"},
            {"name": "Jaeger", "check": "http", "url": "http://localhost:16686/"},
        ]
    }
}

class TestRunner:
    def __init__(self):
        self.console = Console() if RICH_AVAILABLE else None
        self.results = {
            "total": 0,
            "passed": 0,
            "failed": 0,
            "skipped": 0,
            "services": {},
            "start_time": datetime.now().isoformat(),
        }
        
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
    
    async def check_port(self, host: str, port: int, timeout: float = 2.0) -> bool:
        """Check if a port is open"""
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
    
    async def check_http(self, url: str, timeout: float = 5.0) -> Tuple[bool, Optional[int]]:
        """Check HTTP endpoint"""
        try:
            proc = await asyncio.create_subprocess_exec(
                'curl', '-s', '-o', '/dev/null', '-w', '%{http_code}',
                '--connect-timeout', str(int(timeout)),
                url,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.DEVNULL
            )
            stdout, _ = await proc.communicate()
            status_code = int(stdout.decode().strip())
            return status_code == 200, status_code
        except:
            return False, None
    
    async def run_test(self, test: Dict) -> Dict:
        """Run a single test"""
        result = {
            "name": test["name"],
            "status": "unknown",
            "duration": 0,
            "details": ""
        }
        
        start_time = time.time()
        
        try:
            if test["check"] == "port":
                success = await self.check_port(test["host"], test["port"])
                result["status"] = "passed" if success else "failed"
                result["details"] = f"{test['host']}:{test['port']}"
            
            elif test["check"] == "http":
                success, status_code = await self.check_http(test["url"])
                result["status"] = "passed" if success else "failed"
                result["details"] = f"HTTP {status_code}" if status_code else "No response"
            
            elif test["check"] == "command":
                proc = await asyncio.create_subprocess_shell(
                    test["command"],
                    stdout=asyncio.subprocess.PIPE,
                    stderr=asyncio.subprocess.PIPE
                )
                await proc.communicate()
                result["status"] = "passed" if proc.returncode == 0 else "failed"
                result["details"] = f"Exit code: {proc.returncode}"
        
        except Exception as e:
            result["status"] = "failed"
            result["details"] = str(e)
        
        result["duration"] = time.time() - start_time
        return result
    
    async def run_category(self, category_id: str, category: Dict, progress=None, task_id=None) -> List[Dict]:
        """Run all tests in a category"""
        results = []
        
        for i, test in enumerate(category["tests"]):
            if progress and task_id:
                progress.update(task_id, advance=1, description=f"Testing {test['name']}...")
            
            result = await self.run_test(test)
            results.append(result)
            
            # Update counters
            self.results["total"] += 1
            if result["status"] == "passed":
                self.results["passed"] += 1
            elif result["status"] == "failed":
                self.results["failed"] += 1
            else:
                self.results["skipped"] += 1
        
        return results
    
    def create_results_table(self, category: str, results: List[Dict]) -> Table:
        """Create a rich table for test results"""
        table = Table(
            title=f"{TEST_CATEGORIES[category]['icon']} {TEST_CATEGORIES[category]['name']}",
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
            }.get(result["status"], "?")
            
            table.add_row(
                result["name"],
                status_style,
                result["details"],
                f"{result['duration']:.2f}s"
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
                total_tests = sum(len(cat["tests"]) for cat in TEST_CATEGORIES.values())
                
                # Create main task
                main_task = progress.add_task(
                    "[cyan]Running tests...", 
                    total=total_tests
                )
                
                # Run each category
                for category_id, category in TEST_CATEGORIES.items():
                    results = await self.run_category(
                        category_id, category, progress, main_task
                    )
                    self.results["services"][category_id] = results
                    
                    # Show results table
                    table = self.create_results_table(category_id, results)
                    self.console.print(table)
                    self.console.print()
        
        else:
            # Basic output without rich
            for category_id, category in TEST_CATEGORIES.items():
                print(f"\n{category['icon']} {category['name']}")
                print("-" * 40)
                
                results = await self.run_category(category_id, category)
                self.results["services"][category_id] = results
                
                for result in results:
                    status = "✓" if result["status"] == "passed" else "✗"
                    print(f"  {status} {result['name']}: {result['details']}")
    
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
        report_dir = Path("test-reports")
        report_dir.mkdir(exist_ok=True)
        
        timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        json_file = report_dir / f"test-results-{timestamp}.json"
        
        with open(json_file, 'w') as f:
            json.dump(self.results, f, indent=2)
        
        if self.console:
            self.console.print(f"\n[dim]Results saved to: {json_file}[/dim]")
        else:
            print(f"\nResults saved to: {json_file}")
    
    async def run(self):
        """Main execution function"""
        self.print_header()
        await self.run_all_tests()
        self.show_summary()
        self.save_results()
        
        # Return exit code based on failures
        return 0 if self.results['failed'] == 0 else 1

async def main():
    """Main entry point"""
    runner = TestRunner()
    exit_code = await runner.run()
    sys.exit(exit_code)

if __name__ == "__main__":
    # Check Python version
    if sys.version_info < (3, 7):
        print("Error: Python 3.7+ required")
        sys.exit(1)
    
    # Run the test runner
    asyncio.run(main())