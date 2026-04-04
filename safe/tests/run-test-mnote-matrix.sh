#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

export LC_ALL=C
export LANG=
export LANGUAGE=

images=(
    canon_makernote_variant_1.jpg
    fuji_makernote_variant_1.jpg
    olympus_makernote_variant_2.jpg
    pentax_makernote_variant_2.jpg
)

for image in "${images[@]}"; do
    "$script_dir/run-c-test.sh" test-mnote "$script_dir/testdata/$image"
done
