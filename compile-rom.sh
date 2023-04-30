#!/bin/bash

set -e

src_path=$1
os_path="programs/os"
if [ ! -d "$src_path" ]; then
    echo "Usage: $0 <path to source directory>"
    exit 1
fi

rom_name=$(basename "$src_path")
output_dir="build/$rom_name"

[ -d "build" ] && rm -r build
mkdir -p "$output_dir"

echo "Using output dir $output_dir"

cp "$src_path/"* "$output_dir"
cp "$os_path/"* "$output_dir"

echo "Compiling Jack code..."
hack-stack/target/release/jack-compile "$output_dir"

echo "Translating VM code..."
hack-stack/target/release/hack-vm-translate "$output_dir"

echo "Assembling $output_dir/$rom_name.asm -> $output_dir/$rom_name.hack"
hack-stack/target/release/hack-assemble "$output_dir/$rom_name.asm"

num_inst=$(wc -l "$output_dir/$rom_name.hack" | awk '{print $1}')
echo "Program has $num_inst instructions"

echo "Copying to hack-web/www/roms"
cp "$output_dir/$rom_name.hack" "hack-web/www/roms/"
