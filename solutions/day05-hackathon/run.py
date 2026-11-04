#!/usr/bin/env python3
"""Native-only, bounded hackathon runner."""
import argparse,subprocess,sys
from pathlib import Path
root=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('mode',choices=['baseline','acceptance','probe']);p.add_argument('scenario',nargs='?',default='load',choices=['load','slow','replay','burst','query','rollout','fragment','layout']);p.add_argument('--clients',type=int,default=24);a=p.parse_args()
if not 1<=a.clients<=48:p.error('clients must be 1..48')
c=['cargo','+nightly-2026-07-22']
try:
 if a.mode!='probe':sys.exit(subprocess.run(c+['test','-j','1','--manifest-path',str(root/'Cargo.toml'),'--offline','--locked','--test',a.mode],timeout=180).returncode)
 subprocess.run(c+['build','-j','1','--release','--manifest-path',str(root/'Cargo.toml'),'--offline','--locked'],check=True,timeout=180)
 sys.exit(subprocess.run([str(root/'target/release/day05-hackathon'),a.scenario,str(a.clients)],timeout=20).returncode)
except (subprocess.TimeoutExpired,subprocess.CalledProcessError) as e:
 print('Command failed:',e,file=sys.stderr);sys.exit(1)
