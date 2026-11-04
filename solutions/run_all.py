#!/usr/bin/env python3
"""Run the published reference suites without modifying exercise files."""
from pathlib import Path
import subprocess,sys
root=Path(__file__).resolve().parent
for name in ['day05-hackathon', 'day04-classroom/8b', 'day04-classroom/8c', 'day04-classroom/final', 'day04-challenge', 'production-labs', 'labs']:
    print('Checking '+name,flush=True)
    result=subprocess.run(['cargo','+nightly-2026-07-22','test','--manifest-path',str(root/name/'Cargo.toml'),'--workspace','--offline','--locked','-j','1'])
    if result.returncode:sys.exit(result.returncode)
print('All reference suites passed.')
