#!/usr/bin/env python3
"""Bounded classroom runs. Saves raw evidence; stops only its own child processes."""
import argparse, datetime, hashlib, json, os, pathlib, platform, re, shutil, subprocess, sys, time
ROOT = pathlib.Path(__file__).resolve().parent
OUT = ROOT / 'evidence'
TOOLCHAIN = '+nightly-2026-07-22'

def execute(args, limit=90):
    result = subprocess.run(args, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=limit)
    print(result.stdout, end='', flush=True)
    if result.returncode:
        raise RuntimeError(f'Command failed ({result.returncode}): {args}')
    return result.stdout

def cargo(command, args=()):
    return ['cargo', TOOLCHAIN, command, '--manifest-path', str(ROOT / 'Cargo.toml'), '--offline', '--locked', *args]

def save(name, output):
    OUT.mkdir(exist_ok=True)
    raw=OUT / f'{name}.txt'; raw.write_text(output)
    meta={'scenario':name, 'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(), 'platform':platform.platform(), 'source_sha256':hashlib.sha256((ROOT/'src/main.rs').read_bytes()).hexdigest(), 'raw':raw.name}
    (OUT/f'{name}.json').write_text(json.dumps(meta,indent=2)+'\n')
    print(f'RAW EVIDENCE: {raw}')

def run(name):
    output=execute(cargo('run',['--',name]))
    save(name,output)
    return output

def linux_evidence(binary):
    """GDB owns its inferior; no attach to an unrelated process or ptrace changes."""
    gdb=shutil.which('gdb')
    if not gdb:
        raise RuntimeError('GDB is missing. Install your distribution GDB package, then retry. See LINUX-EXERCISE.md.')
    commands=['set pagination off','set confirm off','set disable-randomization off',
              'break classroom_capture_ready','run','info threads','thread apply all bt','kill']
    args=[gdb,'-nx','-q','--batch']
    for command in commands:args.extend(['-ex',command])
    args.extend(['--args',str(binary),'sample-child'])
    try:
        result=subprocess.run(args,cwd=ROOT,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=30)
        output=result.stdout
    except subprocess.TimeoutExpired as error:
        output=error.stdout or ''
        if isinstance(output,bytes):output=output.decode(errors='replace')
        (OUT/'gdb-capture.log').write_text(output)
        raise RuntimeError('GDB capture timed out. Inspect evidence/gdb-capture.log; no successful capture is recorded.') from error
    (OUT/'gdb-capture.log').write_text(output)
    ready=re.search(r'SAMPLE_READY pid=(\d+)',output)
    stopped=re.search(r'(?:hit Breakpoint \d+|Breakpoint \d+,).*classroom_capture_ready',output)
    if result.returncode or not ready or not stopped or not re.search(r'^#0\s',output,re.M):
        raise RuntimeError('GDB did not capture the ready process. Read evidence/gdb-capture.log. Use a permitted native Linux host; do not change ptrace_scope for this exercise.')
    pid=int(ready.group(1))
    events='\n'.join(line for line in output.splitlines() if line.startswith((' INFO',' WARN','SAMPLE_READY')))+'\n'
    (OUT/'sample-child.log').write_text(events)
    snapshot='Linux GDB native thread backtraces (one stopped instant, not statistical samples or a core dump).\nPID: '+str(pid)+'\n\n'+output
    (OUT/'native-sample.txt').write_text(snapshot)
    print(events)
    print('NATIVE LINUX EXCERPT:')
    print('\n'.join(line for line in output.splitlines() if any(word in line for word in ('Thread ', '#0 ', '#1 ', 'futex', 'epoll', 'classroom_capture_ready')))[:6500])
    save('evidence',snapshot)
    meta_path=OUT/'evidence.json';meta=json.loads(meta_path.read_text())
    meta.update(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),binary=str(binary),pid=pid,artifact='native-sample.txt',kind='Linux GDB all-thread backtrace snapshot, not a core dump',debugger='gdb',command=args)
    meta_path.write_text(json.dumps(meta,indent=2)+'\n')
    print('The breakpoint thread is intentionally stopped at classroom_capture_ready. Inspect the other threads and correlate with the lock events; a futex/epoll wait alone is not proof of deadlock.')
    print(f'FULL LINUX STACKS: {OUT / "native-sample.txt"}')
    return OUT/'native-sample.txt'


def native_evidence():
    """Capture the dedicated demo child using the host-native debugger/sampler."""
    execute(cargo('build'))
    metadata=json.loads(subprocess.check_output(cargo('metadata',['--format-version','1','--no-deps']),text=True,cwd=ROOT))
    binary=pathlib.Path(metadata['target_directory'])/'debug/day05-incident-demo'
    OUT.mkdir(exist_ok=True)
    if platform.system()=='Linux':
        return linux_evidence(binary)
    with (OUT/'sample-child.log').open('w+') as log:
        child=subprocess.Popen([str(binary),'sample-child'],stdout=log,stderr=subprocess.STDOUT,cwd=ROOT)
        try:
            deadline=time.monotonic()+5
            while time.monotonic()<deadline:
                log.flush(); text=(OUT/'sample-child.log').read_text()
                if 'SAMPLE_READY' in text:break
                if child.poll() is not None:raise RuntimeError('Sample child exited before ready')
                time.sleep(.02)
            else:raise RuntimeError('Sample readiness deadline exceeded')
            print(text)
            sample=shutil.which('sample')
            if platform.system()=='Darwin' and sample:
                output=execute([sample,str(child.pid),'1','10','-file',str(OUT/'native-sample.txt')],limit=10)
                native=(OUT/'native-sample.txt').read_text()
                print('\nNATIVE SAMPLE EXCERPT (actual captured thread stacks):')
                excerpt=[line for line in native.splitlines() if any(word in line for word in ('Thread_', 'kevent', 'park', 'mach_msg', 'semaphore', 'pthread'))][:18]
                print('\n'.join(excerpt) if excerpt else '\n'.join(native.splitlines()[:30]))
                print('COMPARE: child logs identify the two held locks; native stacks show worker/thread activity. A parked thread alone does not prove deadlock.')
                save('evidence',text+'\n'+output+'\n'+'\n'.join(excerpt))
                meta_path=OUT/'evidence.json'
                meta=json.loads(meta_path.read_text())
                meta.update(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),binary=str(binary),pid=child.pid,artifact='native-sample.txt',kind='macOS live thread sample, not a core dump')
                meta_path.write_text(json.dumps(meta,indent=2)+'\n')
                print(f'FULL NATIVE SAMPLE: {OUT / "native-sample.txt"}')
            else:
                raise RuntimeError('Native capture requires Linux with GDB or macOS with sample. No core capture is claimed.')
        finally:
            if child.poll() is None:
                child.terminate()
                try:child.wait(timeout=3)
                except subprocess.TimeoutExpired:child.kill();child.wait()
    return OUT/'native-sample.txt'

def review():
    output=run('all')
    print('\nEVIDENCE REVIEW | original | repaired | conclusion')
    for name in ('deadlock','leak','panic'):
        parts=re.findall(r'=== '+name+r' (BEFORE|AFTER).*?observed (Observation[^\n]*)',output,re.S)
        assert len(parts)==2, (name,parts)
        print(name+' | '+parts[0][1]+' | '+parts[1][1]+' | '+('PASS' if f'CHECK PASS: {name}' in output else 'FAIL'))
    print('Review: increasing timeout does not break a lock cycle; retaining handles alone does not stop work; catching panics is not input validation.')

def report():
    if not (OUT/'all.txt').exists():run('all')
    entries=[]
    for meta in sorted(OUT.glob('*.json')):
        data=json.loads(meta.read_text());entries.append(f"- {data['scenario']}: UTC {data['utc']}; source {data['source_sha256']}; raw `{data['raw']}`" + (f"; binary SHA256 {data['binary_sha256']}; snapshot `{data['artifact']}`" if 'binary_sha256' in data else ''))
    text='''# Classroom incident evidence\n\nThese are actual local executions, not production measurements.\n\n## Mechanisms and repairs\n\n- Deadlock: opposing lock order; both first locks synchronized by a barrier. Fixed paths acquire metrics then registry.\n- Task leak: completed TCP replies leave sleeper tasks alive. Fixed requests join finite children. Demo supervisor cleans up injected leaked tasks after observation.\n- Panic: malformed input panics a connection task; supervised JoinError and a second successful connection show task/process separation. Fixed parser returns ERR.\n\n## Captured runs\n\n'''+ '\n'.join(entries)+'''\n\n## Limits\n\nNo Docker or Linux core was captured. native-sample.txt, when present, is a macOS live thread sample or Linux GDB backtrace snapshot; evidence.json identifies which. Neither is a core dump. Task counts come from explicit lifetime guards, not tokio-console. Runs are bounded local scenarios; they do not establish production SLOs, all schedules, crash durability or cross-process retry safety. Canary windows are labelled teaching fixtures.\n'''
    OUT.mkdir(exist_ok=True);(OUT/'INCIDENT.md').write_text(text)
    print(text);print(f'REPORT: {OUT / "INCIDENT.md"}')

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['all','deadlock','leak','panic','retry','rollout','evidence','test','review','report'])
    name=parser.parse_args().command
    if name=='evidence':native_evidence()
    elif name=='test':save('test',execute(cargo('test',['--','--nocapture','--test-threads=1'])))
    elif name=='review':review()
    elif name=='report':report()
    else:run(name)

if __name__=='__main__':
    try:main()
    except (RuntimeError,subprocess.TimeoutExpired) as error:
        print(f'DEMO FAILED: {error}',file=sys.stderr);sys.exit(1)
