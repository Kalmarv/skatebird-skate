"""Write SHA256 input identities without depending on PowerShell modules."""
import hashlib
import json
from pathlib import Path
import sys

output = Path(sys.argv[1])
records = []
for arg in sys.argv[2:]:
    path = Path(arg).resolve(strict=True)
    records.append({'path': str(path), 'bytes': path.stat().st_size,
                    'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
output.write_text(json.dumps(records, indent=2) + '\n', encoding='utf-8')
print(f'Recorded {len(records)} input hashes: {output}')
