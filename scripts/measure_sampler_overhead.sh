#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "sampler overhead measurement is macOS-only" >&2
  exit 1
fi

echo "CPU sampler: /usr/sbin/iostat -C -w 1 -c 2"
/usr/bin/time -lp /usr/sbin/iostat -C -w 1 -c 2 >/dev/null

echo "Memory sampler: /usr/bin/vm_stat + /usr/sbin/sysctl -n hw.memsize"
/usr/bin/time -lp /usr/bin/vm_stat >/dev/null
/usr/bin/time -lp /usr/sbin/sysctl -n hw.memsize >/dev/null
