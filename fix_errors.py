#!/usr/bin/env python3
"""Fix all Rust compilation errors by analyzing errors and generating fixes."""
import subprocess
import re
from collections import defaultdict

# Run cargo build and capture errors
result = subprocess.run(
    ["source ~/.cargo/env && cd ~/wikilabs-ai-copilot && cargo build 2>&1"],
    shell=True, capture_output=True, text=True,
    env={"PATH": "/home/khopu/.cargo/bin:/usr/bin:/bin",
         "HOME": "/home/khopu", "USER": "khopu"}
)
output = result.stdout

# Parse errors: extract file, line, column, error code, and message
error_pattern = re.compile(
    r'error\[E(\d+)\]: (.+?)\n(?: --> )?src/(\S+?):(\d+):(\d+)'
)
errors = []
for match in error_pattern.finditer(output):
    errors.append({
        'code': match.group(1),
        'message': match.group(2),
        'file': match.group(3),
        'line': int(match.group(4)),
        'col': int(match.group(5)),
    })

# Also capture lines without explicit src/ prefix (some errors)
error_start = re.compile(r'^error\[E\d+\]:')
current_err = {}
for line in output.split('\n'):
    m = error_start.match(line)
    if m:
        code_msg = line[len(m.group()):].strip()
        current_err['code'] = re.search(r'E(\d+)', line).group(1)
        current_err['message'] = code_msg
    elif line.startswith(' --> '):
        parts = line[5:].split(':')
        current_err['file'] = parts[0]
        if len(parts) >= 2:
            current_err['line'] = int(parts[1])
        if len(parts) >= 3:
            current_err['col'] = int(parts[2])
        errors.append(current_err.copy())
        current_err = {}

# Group errors by file
by_file = defaultdict(list)
for e in errors:
    by_file[e['file']].append(e)

print(f"Total errors: {len(errors)}")
print(f"Files with errors: {len(by_file)}")
print()

for f, errs in sorted(by_file.items(), key=lambda x: -len(x[1])):
    codes = defaultdict(int)
    for e in errs:
        codes[e['code']] += 1
    print(f"  src/{f}: {len(errs)} errors")
    for code, count in sorted(codes.items(), key=lambda x: -x[1]):
        print(f"    E{code}: {count}")
    # Sample messages
    print(f"    Messages: {', '.join(set(e['message'][:80] for e in errs[:5]))}")
    print()

# Save error data for next script
import json
with open('/tmp/cargo_errors.json', 'w') as f:
    json.dump({
        'by_file': {
            k: [{**v, 'file': k} for v in errs]
            for k, errs in by_file.items()
        },
        'total': len(errors)
    }, f)