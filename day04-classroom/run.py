#!/usr/bin/env python3
"""Run the saved cumulative demo, save raw measurements, compare matching cases."""
from pathlib import Path
import csv,io,re,statistics,subprocess,sys
root=Path(__file__).resolve().parent
kind=sys.argv[1] if len(sys.argv)>1 else ''
if kind not in ('memory','scan'):raise SystemExit('Usage: python3 day04-classroom/run.py memory|scan')
case='8b' if kind=='memory' else '8c'
project=root/'demos'/case
stage=int(re.search(r'pub const STAGE: usize = (\d+)',(project/'src/lib.rs').read_text())[1])
base=['cargo','+nightly-2026-07-22','run','--release','--manifest-path',str(project/'Cargo.toml'),'--offline','--locked']
subprocess.run(base+['--bin','check-stage'],check=True,timeout=180)
cmd=base+(['--bin','measure-memory','--','compare','1000'] if kind=='memory' else ['--bin','compare-stage'])
r=subprocess.run(cmd,check=True,text=True,stdout=subprocess.PIPE,timeout=180)
results=root/'results'/kind;results.mkdir(parents=True,exist_ok=True)
out=results/f'stage-{stage}.csv';out.write_text(r.stdout)
def summaries(text):
 groups={}
 for row in csv.DictReader(io.StringIO(text)):
  if kind=='memory':key=(row['workload'],row['implementation']);value=int(row['elapsed_ns'])
  else:key=(row['n'],row['sparse'],row['implementation']);value=int(row['ns_per_scan'])
  groups.setdefault(key,[]).append(value)
 return {k:statistics.median(v) for k,v in groups.items()}
current=summaries(r.stdout)
print('\nSeven-sample medians; memory = whole batch including reset, scan = one scan. Units: ns.')
previous=results/f'stage-{stage-1}.csv'
old=summaries(previous.read_text()) if previous.exists() else {}
for key,value in current.items():
 label='/'.join(key);prior=old.get(key)
 print(f'{label}: {value:.0f} ns'+(f' | previous stage {prior:.0f} ns | change {(value/prior-1)*100:+.1f}%' if prior else ''))
if kind=='memory':
 print('Capacity and allocation counts (last sample; requested bytes, not RSS):')
 for row in [r for r in csv.DictReader(io.StringIO(r.stdout)) if r['sample']=='6']:
  print(row['workload'],row['implementation'],'allocations='+row['allocation_calls'],'peak='+row['peak_requested_bytes_delta'],'retained='+row['retained_requested_bytes_delta'],'payload_capacity='+row['retained_payload_capacity'])
print('Raw output:',out)
print('Timing is a local observation. Correctness fixes need checks, not a promised speedup. Previous-stage timing comes from its earlier run, not a simultaneous experiment.')
