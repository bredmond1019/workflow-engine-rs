#!/usr/bin/env python3
"""
Multi-Agent Progress Monitor
Tracks task completion across all agents and updates coordination file
"""

import os
import re
import time
from datetime import datetime
from pathlib import Path

PROJECT_DIR = Path("/Users/brandon/Documents/Projects/AIArchitecture/ai-system-rust")
TASKS_DIR = PROJECT_DIR / "tasks"
COORDINATION_FILE = TASKS_DIR / "multi-agent-coordination.md"

class AgentMonitor:
    def __init__(self):
        self.agents = {
            1: {"name": "DevOps & Foundation", "file": "agent-1-tasks.md"},
            2: {"name": "AI & Core Engine", "file": "agent-2-tasks.md"},
            3: {"name": "Integration & Services", "file": "agent-3-tasks.md"},
            4: {"name": "Database & Events", "file": "agent-4-tasks.md"},
            5: {"name": "Production & QA", "file": "agent-5-tasks.md"}
        }
        
    def parse_task_file(self, filepath):
        """Parse a task file and return completion statistics"""
        if not filepath.exists():
            return {"total": 0, "completed": 0, "in_progress": 0, "percentage": 0}
        
        content = filepath.read_text()
        
        # Count tasks by looking for checkbox patterns
        total_tasks = len(re.findall(r'^\s*-\s*\[[ x]\]', content, re.MULTILINE))
        completed_tasks = len(re.findall(r'^\s*-\s*\[x\]', content, re.MULTILINE))
        
        # Look for in-progress indicators (tasks with partial completion notes)
        in_progress = 0
        if "in progress" in content.lower() or "partial" in content.lower():
            in_progress = 1
        
        percentage = (completed_tasks / total_tasks * 100) if total_tasks > 0 else 0
        
        return {
            "total": total_tasks,
            "completed": completed_tasks,
            "in_progress": in_progress,
            "percentage": percentage
        }
    
    def get_all_agent_status(self):
        """Get status for all agents"""
        status = {}
        for agent_id, agent_info in self.agents.items():
            task_file = TASKS_DIR / agent_info["file"]
            stats = self.parse_task_file(task_file)
            status[agent_id] = {
                "name": agent_info["name"],
                "stats": stats,
                "status": self.determine_status(stats)
            }
        return status
    
    def determine_status(self, stats):
        """Determine agent status based on statistics"""
        if stats["percentage"] == 100:
            return "✅ Complete"
        elif stats["percentage"] == 0:
            return "⏸️ Not Started"
        elif stats["in_progress"] > 0:
            return "🚧 Active"
        else:
            return "🟡 In Progress"
    
    def format_progress_bar(self, percentage, width=20):
        """Create a text-based progress bar"""
        filled = int(width * percentage / 100)
        bar = "█" * filled + "░" * (width - filled)
        return f"[{bar}] {percentage:.0f}%"
    
    def display_dashboard(self, status):
        """Display a formatted dashboard"""
        print("\n" + "="*60)
        print("Multi-Agent Execution Status Dashboard")
        print("="*60)
        print(f"Time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print("-"*60)
        
        total_tasks = sum(s["stats"]["total"] for s in status.values())
        completed_tasks = sum(s["stats"]["completed"] for s in status.values())
        overall_percentage = (completed_tasks / total_tasks * 100) if total_tasks > 0 else 0
        
        for agent_id, agent_status in sorted(status.items()):
            stats = agent_status["stats"]
            print(f"\nAgent {agent_id}: {agent_status['name']}")
            print(f"Status: {agent_status['status']}")
            print(f"Progress: {self.format_progress_bar(stats['percentage'])} ({stats['completed']}/{stats['total']})")
        
        print("\n" + "-"*60)
        print(f"Overall Progress: {self.format_progress_bar(overall_percentage)} ({completed_tasks}/{total_tasks})")
        print("="*60)
    
    def update_coordination_file(self, status):
        """Update the coordination file with current status"""
        # Read current coordination file
        content = COORDINATION_FILE.read_text()
        
        # Find the status section or create it
        status_marker = "## Agent Status"
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        
        # Build status update
        status_lines = [
            f"\n## Agent Status (Last Updated: {timestamp})\n",
            "\n### Progress\n"
        ]
        
        for agent_id, agent_status in sorted(status.items()):
            stats = agent_status["stats"]
            status_lines.append(
                f"- **Agent {agent_id}**: {stats['completed']}/{stats['total']} tasks "
                f"({stats['percentage']:.0f}%) {agent_status['status']}\n"
            )
        
        # Calculate overall progress
        total_tasks = sum(s["stats"]["total"] for s in status.values())
        completed_tasks = sum(s["stats"]["completed"] for s in status.values())
        overall_percentage = (completed_tasks / total_tasks * 100) if total_tasks > 0 else 0
        
        status_lines.extend([
            f"\n### Overall Progress: {completed_tasks}/{total_tasks} tasks ({overall_percentage:.0f}%)\n"
        ])
        
        # Update or append status section
        status_text = "".join(status_lines)
        
        if status_marker in content:
            # Replace existing status section
            # Find the section and replace it
            before_status = content.split(status_marker)[0]
            # Try to find the next section (starting with ##)
            after_parts = content.split(status_marker)[1].split("\n##", 1)
            if len(after_parts) > 1:
                after_status = "\n##" + after_parts[1]
            else:
                after_status = ""
            
            new_content = before_status + status_text + after_status
        else:
            # Append status section
            new_content = content + "\n" + status_text
        
        # Write back
        COORDINATION_FILE.write_text(new_content)
    
    def monitor_continuous(self, interval=30):
        """Continuously monitor agent progress"""
        print("Starting continuous monitoring...")
        print(f"Checking every {interval} seconds. Press Ctrl+C to stop.")
        
        last_status = {}
        
        try:
            while True:
                status = self.get_all_agent_status()
                
                # Check for changes
                changed = False
                for agent_id in status:
                    if agent_id not in last_status:
                        changed = True
                        break
                    if status[agent_id]["stats"] != last_status[agent_id]["stats"]:
                        changed = True
                        break
                
                # Display and update if changed
                if changed or not last_status:
                    os.system('clear' if os.name == 'posix' else 'cls')
                    self.display_dashboard(status)
                    self.update_coordination_file(status)
                    print(f"\nNext check in {interval} seconds...")
                
                # Check for completion
                all_complete = all(
                    s["stats"]["percentage"] == 100 
                    for s in status.values()
                )
                
                if all_complete:
                    print("\n🎉 All agents have completed their tasks!")
                    break
                
                last_status = status
                time.sleep(interval)
                
        except KeyboardInterrupt:
            print("\n\nMonitoring stopped by user.")

def main():
    monitor = AgentMonitor()
    
    # Display current status once
    status = monitor.get_all_agent_status()
    monitor.display_dashboard(status)
    
    # Ask if user wants continuous monitoring
    response = input("\nStart continuous monitoring? (y/n): ")
    if response.lower() == 'y':
        monitor.monitor_continuous()

if __name__ == "__main__":
    main()