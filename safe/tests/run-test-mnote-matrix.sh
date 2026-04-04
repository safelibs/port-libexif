#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

images=(
    canon_makernote_variant_1.jpg
    fuji_makernote_variant_1.jpg
    olympus_makernote_variant_2.jpg
    pentax_makernote_variant_2.jpg
)

for image in "${images[@]}"; do
    LC_ALL=C LANG= LANGUAGE= \
    "$script_dir/run-c-test.sh" test-mnote "$script_dir/testdata/$image"
done

LC_ALL=C LANG= LANGUAGE= \
"$script_dir/run-c-test.sh" test-apple-mnote
