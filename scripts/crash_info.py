#!/usr/bin/env python3
import subprocess
import json

def find_latest_crash(appname):
    crash_dir = '~/Library/Logs/DiagnosticReports'
    cmd = f'find {crash_dir}/{appname}-*.ips | sort -r | head -1'
    crash_file = subprocess.check_output(cmd, shell=True).decode('ascii').rstrip()
    return crash_file

def read_data(filename):
    with open(filename, 'r') as f:
        dat = ''.join(f.readlines()[1:])
        return json.loads(dat)

dat = read_data(find_latest_crash('dis86'))
frames = dat['threads'][0]['frames']
symbols = [frame['symbol'] for frame in frames]

# d = frames
# print(json.dumps(d, indent='  '))

print('Backtrace:')
for sym in symbols:
    print(f'  {sym}')
