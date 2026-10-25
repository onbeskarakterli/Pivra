"""Real window close, certified by shared pidfd/partial-frame observer. No decoder mock."""
import importlib.util,os,json,shutil,tempfile
from pathlib import Path
spec=importlib.util.spec_from_file_location('fd05a_benchmark',Path(__file__).resolve().parents[1]/'fd05a/benchmark.py')
b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
with tempfile.TemporaryDirectory(prefix='pivra-fd06-close-') as folder:
    root=Path(folder);main=root/'01-main.png';shutil.copyfile('tests/fixtures/images/wide.png',main);b.fixture(root/'02-large.png',10000,5000)
    result=b.job('target/debug/examples/thumbnail_overlay_probe',main,10000,5000,True,extra=('--native-thumbnails','--fd06-active-close'))
    fd,report=tempfile.mkstemp(prefix='pivra-fd06-close-report-',suffix='.json')
    with os.fdopen(fd,'w') as out:json.dump(result,out,indent=2)
    print(result['stdout'],result['stderr']);print('Report:',report)
    raise SystemExit(result['exit'])
