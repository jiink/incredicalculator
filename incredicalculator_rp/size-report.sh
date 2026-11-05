#!/usr/bin/env bash
# Builds the RP firmware (release) and reports flash and RAM usage against
# the limits in memory.x, plus the largest symbols in each.
#
# Usage, from anywhere:
#   incredicalculator_rp/size-report.sh
#   incredicalculator_rp/size-report.sh --no-build --top 20
#
# Uses llvm-size and llvm-nm from PATH, or else from the toolchain's
# llvm-tools component (listed in rust-toolchain.toml).
set -euo pipefail

no_build=0
top=10
while [ $# -gt 0 ]; do
    case "$1" in
        --no-build) no_build=1 ;;
        --top) top="${2:?--top needs a number}"; shift ;;
        -h|--help) sed -n '2,10p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done

crate_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(dirname "$crate_dir")"
target=thumbv8m.main-none-eabihf
elf="$repo_root/target/$target/release/incredicalculator_rp"

# Run rustc from the crate so rust-toolchain.toml picks the toolchain.
tools_dir="$(cd "$crate_dir" && rustc --print sysroot)/lib/rustlib/$(cd "$crate_dir" && rustc -vV | sed -n 's/^host: //p')/bin"
find_tool() {
    if command -v "$1" >/dev/null 2>&1; then
        command -v "$1"
    elif [ -x "$tools_dir/$1" ]; then
        echo "$tools_dir/$1"
    else
        echo "$1 not found on PATH or in $tools_dir;" \
            "run 'rustup component add llvm-tools' or install LLVM." >&2
        exit 1
    fi
}
llvm_size="$(find_tool llvm-size)"
llvm_nm="$(find_tool llvm-nm)"

if [ "$no_build" -eq 0 ]; then
    cargo build --release --manifest-path "$crate_dir/Cargo.toml" --target "$target"
fi

if [ ! -f "$elf" ]; then
    echo "Firmware not found at $elf; run without --no-build." >&2
    exit 1
fi

color=0
[ -t 1 ] && color=1

tmp_size="$(mktemp)"
tmp_nm="$(mktemp)"
trap 'rm -f "$tmp_size" "$tmp_nm"' EXIT
"$llvm_size" -A "$elf" > "$tmp_size"
"$llvm_nm" --print-size --size-sort --demangle "$elf" > "$tmp_nm"

python3 - "$crate_dir/memory.x" "$tmp_size" "$tmp_nm" "$top" "$color" <<'EOF'
import re, sys

memory_x, size_file, nm_file, top, color = sys.argv[1:6]
top, color = int(top), color == "1"
KB = 1024

def paint(text, c):
    codes = {"cyan": 36, "red": 31, "yellow": 33, "green": 32}
    return f"\033[{codes[c]}m{text}\033[0m" if color else text

# --- Limits from memory.x ----------------------------------------------------
mem = open(memory_x).read()
def region_length(name):
    m = re.search(rf"^\s*{name}\s*:\s*ORIGIN\s*=\s*\S+\s*,\s*LENGTH\s*=\s*(\d+)\s*([KM]?)", mem, re.M)
    if not m:
        sys.exit(f"Region {name} not found in memory.x")
    return int(m[1]) * {"": 1, "K": KB, "M": KB * KB}[m[2]]
flash_limit = region_length("FLASH")
ram_limit = region_length("RAM")

def in_flash(addr): return 0x10000000 <= addr < 0x20000000
def in_ram(addr): return 0x20000000 <= addr < 0x20080000

# --- Section sizes -----------------------------------------------------------
flash = ram = 0
ram_sections = {}
for line in open(size_file):
    m = re.match(r"^(\.\S+)\s+(\d+)\s+(\d+)$", line.strip())
    if not m:
        continue
    name, size, addr = m[1], int(m[2]), int(m[3])
    if size == 0:
        continue
    if in_flash(addr):
        flash += size
    elif in_ram(addr):
        ram += size
        ram_sections[name] = size
        # .data is stored in flash and copied to RAM at boot.
        if name == ".data":
            flash += size

def kib(n): return f"{n / KB:8,.1f} KiB"
def row(label, used, limit):
    return f"{label:<32}{kib(used)} of{kib(limit)}  ({100 * used / limit:5.1f}%)"

print()
print(paint("incredicalculator_rp memory usage", "cyan"))
print(row("Flash (code + data)", flash, flash_limit))
print(row("RAM (static: data/bss/uninit)", ram, ram_limit))
for name, size in sorted(ram_sections.items(), key=lambda s: -s[1]):
    print(f"    {name:<22}{kib(size)}")
stack = ram_limit - ram
stack_color = "red" if stack < 16 * KB else "yellow" if stack < 48 * KB else "green"
print(paint(f"{'Left for core 0 stack':<32}{kib(stack)}", stack_color))
print('  (heap and stack peaks at run time are logged as "memory: ..." over defmt)')

# --- Largest symbols ---------------------------------------------------------
symbols = []
for line in open(nm_file):
    m = re.match(r"^([0-9a-f]{8}) ([0-9a-f]{8}) (\S) (.+)$", line.rstrip("\n"))
    if m:
        name = re.sub(r"::h[0-9a-f]{16}$", "", m[4])
        symbols.append((int(m[1], 16), int(m[2], 16), name))

def show_largest(title, items):
    print()
    print(paint(title, "cyan"))
    for _, size, name in sorted(items, key=lambda s: -s[1])[:top]:
        if len(name) > 90:
            name = name[:87] + "..."
        print(f"{kib(size)}  {name}")

show_largest("Largest RAM symbols", [s for s in symbols if in_ram(s[0])])
show_largest("Largest flash symbols", [s for s in symbols if in_flash(s[0])])
EOF
