#!/usr/bin/env python3
"""One short lab: a live Tokio Console view and a real Linux core snapshot."""
import argparse,datetime,hashlib,json,os,pathlib,shutil,socket,struct,subprocess,sys,threading,time
ROOT=pathlib.Path(__file__).resolve().parent
LABS=ROOT.parent/'labs'
OUT=ROOT/'evidence/live-lab'
TARGET=pathlib.Path('/tmp/day5-tokio-console-target')
IMAGE='rust-day5-core-lab'
NAME='rust-day5-live-core'

def command(args,**kw):
    print('+ '+' '.join(str(x) for x in args),flush=True)
    return subprocess.run([str(x) for x in args],check=True,**kw)

def build_console():
    env=os.environ.copy();env['RUSTFLAGS']='--cfg tokio_unstable';env['CARGO_TARGET_DIR']=str(TARGET)
    command(['cargo','+nightly-2026-07-22','build','--manifest-path',LABS/'Cargo.toml','-p','lab9-core-dump-starter','--bin','live-console','--features','tokio/tracing','--offline','--locked'],env=env,timeout=240)
    if not shutil.which('tokio-console'):
        raise RuntimeError('Install the viewer once: cargo +stable install tokio-console --locked --version 0.1.13')
    version=subprocess.check_output(['tokio-console','--version'],text=True).strip()
    if version!='tokio-console 0.1.13':
        raise RuntimeError(f'{version} is not the pinned viewer for this lab. Install: cargo +stable install tokio-console --locked --version 0.1.13 --force -j 1')

def free_port():
    with socket.socket() as s:s.bind(('127.0.0.1',0));return s.getsockname()[1]

def exact(s,n):
    data=b''
    while len(data)<n:
        chunk=s.recv(n-len(data))
        if not chunk:raise RuntimeError('Unexpected EOF from the metrics server')
        data+=chunk
    return data

def load(port,count=12,pause=.7):
    """Real framed requests; one deliberately leaked task is created per frame."""
    with socket.create_connection(('127.0.0.1',port),timeout=3) as s:
        for n in range(count):
            body=f'METRIC classroom.{n} {n}.0'.encode();s.sendall(struct.pack('>I',len(body))+body)
            response=exact(s,struct.unpack('>I',exact(s,4))[0]).decode()
            if response!='OK':raise RuntimeError(response)
            time.sleep(pause)

def start_console_server():
    build_console();OUT.mkdir(parents=True,exist_ok=True)
    path=OUT/'console-server.json'
    if path.exists():
        old=json.loads(path.read_text())
        try:
            with socket.create_connection(('127.0.0.1',old['console_port']),timeout=.3):
                print('Reusing the classroom console endpoint recorded in',path);return old
        except OSError:pass
    metrics_port,console_port=free_port(),free_port()
    while console_port==metrics_port:console_port=free_port()
    env=os.environ.copy();env.update(TOKIO_CONSOLE='1',TOKIO_CONSOLE_BIND=f'127.0.0.1:{console_port}',METRICS_PORT=str(metrics_port),FAULT='task_leak',RUST_LOG='info')
    log=(OUT/'console-server.log').open('w')
    p=subprocess.Popen([str(TARGET/'debug/live-console')],env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
    log.close()
    try:
        deadline=time.monotonic()+10
        while time.monotonic()<deadline:
            if p.poll() is not None:raise RuntimeError('Console server exited; read console-server.log')
            try:
                with socket.create_connection(('127.0.0.1',console_port),timeout=.2):break
            except OSError:time.sleep(.1)
        else:raise RuntimeError('Console endpoint did not become ready')
    except BaseException:
        p.terminate();p.wait(timeout=3);raise
    state=dict(pid=p.pid,metrics_port=metrics_port,console_port=console_port,binary=str(TARGET/'debug/live-console'),created=datetime.datetime.now(datetime.timezone.utc).isoformat())
    path.write_text(json.dumps(state,indent=2)+'\n')
    print('Console server ready:',json.dumps(state));return state

def stop():
    path=OUT/'console-server.json'
    if not path.exists():return
    state=json.loads(path.read_text())
    # Verify the PID still names our dedicated executable before terminating it.
    result=subprocess.run(['ps','-p',str(state['pid']),'-o','command='],text=True,capture_output=True)
    if state['binary'] in result.stdout:
        os.kill(state['pid'],15);print('Stopped classroom console server',state['pid'])
    else:print('Recorded classroom server is no longer running; no process stopped.')
    path.unlink()

def console():
    state=start_console_server()
    # Bound the injected fault: 12 additional tasks, not an endless load generator.
    errors=[]
    def traffic():
        try:time.sleep(2);load(state['metrics_port'])
        except Exception as e:errors.append(str(e))
    worker=threading.Thread(target=traffic,daemon=True);worker.start()
    print('LIVE VIEW: watch tasks accumulate; Enter inspects a selected task; q exits. Filter/inspect source server.rs:129. The server stays running until Stop live lab.',flush=True)
    try:command(['tokio-console',f"http://127.0.0.1:{state['console_port']}"],env={**os.environ,'RUST_LOG':'off'})
    finally:
        worker.join(timeout=12)
        if errors:print('TRAFFIC ERROR:',errors)

def build_core():
    command(['docker','info','--format','{{.ServerVersion}}'],timeout=15)
    command(['docker','build','--progress=plain','-t',IMAGE,'-f',LABS/'lab9-core-dump/docker/Dockerfile',LABS],timeout=600)

def core():
    OUT.mkdir(parents=True,exist_ok=True)
    found=subprocess.run(['docker','image','inspect',IMAGE],capture_output=True)
    if found.returncode:raise RuntimeError('Core image is not built yet. Prepare outside the live class with: python3 day05-incident-demo/live_lab.py build-core')
    existing=subprocess.run(['docker','container','inspect',NAME],capture_output=True)
    if existing.returncode==0:
        print('Using preserved lab container. Run Inspect saved core; existing evidence is not overwritten.');inspect_core();return
    command(['docker','run','--name',NAME,'--cap-add=SYS_PTRACE','--security-opt','seccomp=unconfined','-e','LAB9_INTERACTIVE=0','-e','TOKIO_WORKER_THREADS=2',IMAGE],timeout=90)
    command(['docker','cp',f'{NAME}:/tmp/lab9-crash',OUT/'core-capture'],timeout=30)
    command(['docker','cp',f'{NAME}:/workspace/target/release/lab9-core-dump-starter',OUT/'matching-linux-binary'],timeout=30)
    # Reopen the dumped process from files in a separate Linux process.
    inspect_core()

def inspect_core():
    out=OUT/'core-capture'
    if not (out/'cores/core.deadlock').is_file():
        raise RuntimeError('Capture the core first with: python3 day05-incident-demo/live_lab.py core')
    args=['docker','run','--rm','--network','none','--entrypoint','gdb','-v',f'{OUT}:/evidence:ro',IMAGE,'-q','-nx','-batch','/evidence/matching-linux-binary','/evidence/core-capture/cores/core.deadlock','-ex','info threads','-ex','thread apply all bt']
    result=subprocess.run(args,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=30)
    (OUT/'core-backtraces.txt').write_text(result.stdout)
    print(result.stdout)
    if result.returncode or 'Core was generated' not in result.stdout or '#0' not in result.stdout:
        raise RuntimeError('Core reopen did not produce thread stacks; inspect core-backtraces.txt')
    core_file=out/'cores/core.deadlock';binary=OUT/'matching-linux-binary'
    meta=dict(core=str(core_file),core_bytes=core_file.stat().st_size,binary=str(binary),binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),inspection='separate Linux GDB process, original target stopped')
    (OUT/'core-metadata.json').write_text(json.dumps(meta,indent=2)+'\n')
    print('REAL CORE REOPENED:',json.dumps(meta))

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('action',choices=['prepare-console','start','console','load','stop','build-core','core','inspect']);a=p.parse_args().action
    if a=='prepare-console':build_console()
    elif a=='start':start_console_server()
    elif a=='console':console()
    elif a=='load':s=start_console_server();load(s['metrics_port'],count=6,pause=.1);print('Six requests accepted; six more leaked tasks were created.')
    elif a=='stop':stop()
    elif a=='build-core':build_core()
    elif a=='core':core()
    elif a=='inspect':inspect_core()
if __name__=='__main__':
    try:main()
    except (RuntimeError,subprocess.CalledProcessError,subprocess.TimeoutExpired) as e:print('LIVE LAB FAILED:',e,file=sys.stderr);sys.exit(1)
