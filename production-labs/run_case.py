#!/usr/bin/env python3
"""Run one case with a process-group watchdog, so a broken concurrency fix cannot hold class."""
import argparse,os,pathlib,signal,subprocess,sys
p=argparse.ArgumentParser();p.add_argument('case');p.add_argument('--baseline',action='store_true');p.add_argument('--filter',default='');p.add_argument('--timeout',type=int,default=90);a=p.parse_args()
root=pathlib.Path(__file__).resolve().parent
if a.case not in {d.name for d in root.glob('case*') if d.is_dir()}:p.error('unknown case')
cmd=['cargo','+nightly-2026-07-22','test','--manifest-path',str(root/'Cargo.toml'),'-p',a.case,'--test','baseline' if a.baseline else 'acceptance','--offline','--locked']
if a.filter:
 listing=subprocess.run(cmd+['--','--list'],capture_output=True,text=True,timeout=a.timeout)
 if listing.returncode or f'{a.filter}: test' not in listing.stdout.splitlines():
  print(listing.stderr,file=sys.stderr);p.error('filter does not identify a compiled test')
 cmd += [a.filter,'--','--exact']
proc=subprocess.Popen(cmd,start_new_session=True)
try:code=proc.wait(timeout=a.timeout)
except subprocess.TimeoutExpired:
 os.killpg(proc.pid,signal.SIGKILL);proc.wait();print('WATCHDOG: case exceeded time limit; inspect task ownership / waits.',file=sys.stderr);code=124
sys.exit(code)
