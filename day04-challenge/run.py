#!/usr/bin/env python3
"""Bounded local classroom runner. No Docker, server installation or shell interpolation."""
from pathlib import Path
import argparse,subprocess,sys,time
root=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('mode',choices=['baseline','acceptance','healthy','slow','retry','burst','overload','profile','sample']);p.add_argument('--clients',type=int,default=24);args=p.parse_args()
if not 1<=args.clients<=64:p.error('--clients must be 1..64')
cargo=['cargo','+nightly-2026-07-22']
try:
 if args.mode in ('baseline','acceptance'):
  result=subprocess.run(cargo+['test','--manifest-path',str(root/'Cargo.toml'),'--offline','--locked','--test',args.mode],timeout=180);sys.exit(result.returncode)
 subprocess.run(cargo+['build','--release','--manifest-path',str(root/'Cargo.toml'),'--offline','--locked'],check=True,timeout=180)
 binary=root/'target/release/day04-challenge'
 if args.mode=='sample':
  if sys.platform!='darwin':p.error('sample uses macOS; use profile with your native Linux profiler')
  results=root/'results';results.mkdir(exist_ok=True)
  with (results/'profile.log').open('w') as out:
   child=subprocess.Popen([str(binary),'profile',str(args.clients),'20'],stdout=out,stderr=subprocess.STDOUT)
   try:
    time.sleep(.4)
    if child.poll() is not None:raise RuntimeError('Workload exited before sampling; inspect results/profile.log')
    subprocess.run(['/usr/bin/sample',str(child.pid),'3','1','-file',str(results/'cpu-sample.txt')],check=True,timeout=15)
    code=child.wait(timeout=35)
   finally:
    if child.poll() is None:child.terminate();child.wait(timeout=5)
  print('CPU sample:',results/'cpu-sample.txt');sys.exit(code)
 sys.exit(subprocess.run([str(binary),args.mode,str(args.clients),'20'],timeout=40).returncode)
except (subprocess.TimeoutExpired,subprocess.CalledProcessError,RuntimeError) as e:
 print('Challenge command failed:',e,file=sys.stderr);sys.exit(1)
