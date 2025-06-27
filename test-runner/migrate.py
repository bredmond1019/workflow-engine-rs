#!/usr/bin/env python3
"""
Migration script to move from the old test-runner.py to the new UV-based test runner
"""

import shutil
import sys
from pathlib import Path


def main():
    """Run migration"""
    print("🔄 Migrating to UV-based test runner...")
    print()
    
    # Check if old test runner exists
    old_runner = Path("../test-runner.py")
    if old_runner.exists():
        print("✓ Found old test-runner.py")
        
        # Create backup
        backup_path = Path("../test-runner.py.backup")
        if not backup_path.exists():
            shutil.copy2(old_runner, backup_path)
            print(f"✓ Created backup at {backup_path}")
        else:
            print("⚠️  Backup already exists, skipping")
    else:
        print("ℹ️  Old test-runner.py not found (already migrated?)")
    
    # Check for test reports
    old_reports = Path("../test-reports")
    new_reports = Path("test-reports")
    
    if old_reports.exists() and not new_reports.exists():
        print(f"\n📁 Found test reports at {old_reports}")
        response = input("Move test reports to new location? [y/N]: ")
        if response.lower() == 'y':
            shutil.move(str(old_reports), str(new_reports))
            print("✓ Moved test reports")
    
    print("\n✅ Migration complete!")
    print("\nNext steps:")
    print("1. Run ./install.sh to set up UV and install dependencies")
    print("2. Activate the virtual environment: source .venv/bin/activate")
    print("3. Run the new test runner: workflow-test")
    print("\nThe old test-runner.py has been backed up but can be removed once you verify the new runner works.")


if __name__ == "__main__":
    main()