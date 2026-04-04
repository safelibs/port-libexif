#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
IMAGE_TAG="${LIBEXIF_ORIGINAL_TEST_IMAGE:-libexif-original-test:ubuntu24.04}"
ONLY=""

usage() {
  cat <<'EOF'
usage: test-original.sh [--only <dependent-name>]

Builds the local Debian packages from ./original inside an Ubuntu 24.04 Docker
container, installs them, and smoke-tests the dependent software recorded in
dependents.json.

--only runs just one dependent by exact .dependents[].name.
EOF
}

while (($#)); do
  case "$1" in
    --only)
      ONLY="${2:?missing value for --only}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      printf 'unknown option: %s\n' "$1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

command -v docker >/dev/null 2>&1 || {
  echo "docker is required to run $0" >&2
  exit 1
}

[[ -d "$ROOT/original" ]] || {
  echo "missing original source tree" >&2
  exit 1
}

[[ -f "$ROOT/dependents.json" ]] || {
  echo "missing dependents.json" >&2
  exit 1
}

docker build -t "$IMAGE_TAG" - <<'DOCKERFILE'
FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive

RUN sed -i 's/^Types: deb$/Types: deb deb-src/' /etc/apt/sources.list.d/ubuntu.sources \
 && apt-get update \
 && apt-get install -y --no-install-recommends \
      build-essential \
      ca-certificates \
      dbus-x11 \
      debhelper \
      dpkg-dev \
      fakeroot \
      exif \
      exiftran \
      eog \
      eog-plugins \
      file \
      foxtrotgps \
      gphoto2 \
      gtkam \
      imagemagick \
      jq \
      libcamlimages-ocaml \
      libcamlimages-ocaml-dev \
      libexif-gtk-dev \
      libexif-gtk3-5 \
      libgtk-3-dev \
      minidlna \
      ocaml-findlib \
      ocaml-nox \
      pkg-config \
      ruby \
      ruby-exif \
      shotwell \
      sqlite3 \
      tracker-extract \
      xauth \
      xvfb \
      gerbera \
 && apt-get build-dep -y --no-install-recommends libexif \
 && rm -rf /var/lib/apt/lists/*
DOCKERFILE

docker run --rm -i \
  -e "LIBEXIF_TEST_ONLY=$ONLY" \
  -v "$ROOT":/work:ro \
  "$IMAGE_TAG" \
  bash -s <<'CONTAINER_SCRIPT'
set -euo pipefail

export LANG=C.UTF-8
export LC_ALL=C.UTF-8
export DEBIAN_FRONTEND=noninteractive

ROOT=/work
ONLY_FILTER="${LIBEXIF_TEST_ONLY:-}"
MULTIARCH="$(dpkg-architecture -qDEB_HOST_MULTIARCH)"
SOURCE_COPY=/tmp/libexif-original-src
FIXTURE_ROOT=/tmp/libexif-fixtures
TEST_ROOT=/tmp/libexif-dependent-tests
FUJI_FIXTURE="$ROOT/original/test/testdata/fuji_makernote_variant_1.jpg"
GENERATED_FIXTURE="$FIXTURE_ROOT/generated-exif.jpg"
GPHOTO_FIXTURE_DIR="$FIXTURE_ROOT/gphoto-camera"
ACTIVE_LIBEXIF=""
ORIGINAL_RUNTIME_DEB=""
ORIGINAL_DEV_DEB=""
ORIGINAL_RUNTIME_VERSION=""
ORIGINAL_DEV_VERSION=""

declare -a REQUIRED_DEPENDENTS=(
  "exif"
  "exiftran"
  "eog-plugin-exif-display"
  "eog-plugin-map"
  "tracker-extract"
  "Shotwell"
  "FoxtrotGPS"
  "gphoto2"
  "GTKam"
  "MiniDLNA"
  "Gerbera"
  "ruby-exif"
  "libexif-gtk3"
  "CamlImages"
  "ImageMagick"
)

log_step() {
  printf '\n==> %s\n' "$1"
}

die() {
  echo "error: $*" >&2
  exit 1
}

require_nonempty_file() {
  local path="$1"

  [[ -s "$path" ]] || die "expected non-empty file: $path"
}

require_contains() {
  local path="$1"
  local needle="$2"

  if ! grep -F -- "$needle" "$path" >/dev/null 2>&1; then
    printf 'missing expected text in %s: %s\n' "$path" "$needle" >&2
    printf -- '--- %s ---\n' "$path" >&2
    cat "$path" >&2
    exit 1
  fi
}

require_not_contains() {
  local path="$1"
  local needle="$2"

  if grep -F -- "$needle" "$path" >/dev/null 2>&1; then
    printf 'found unexpected text in %s: %s\n' "$path" "$needle" >&2
    printf -- '--- %s ---\n' "$path" >&2
    cat "$path" >&2
    exit 1
  fi
}

should_run() {
  local name="$1"

  [[ -z "$ONLY_FILTER" || "$name" == "$ONLY_FILTER" ]]
}

reset_test_dir() {
  local name="$1"
  local dir="$TEST_ROOT/$name"

  rm -rf "$dir"
  mkdir -p "$dir"
  printf '%s\n' "$dir"
}

assert_status_equals() {
  local expected="$1"
  local actual="$2"
  local label="$3"

  [[ "$actual" == "$expected" ]] || die "$label returned status $actual, expected $expected"
}

assert_binary_uses_active_libexif() {
  local target="$1"
  local resolved

  resolved="$(ldd "$target" | awk '$1 == "libexif.so.12" { print $3; exit }')"
  [[ -n "$resolved" ]] || die "ldd did not report libexif.so.12 for $target"
  [[ "$(readlink -f "$resolved")" == "$ACTIVE_LIBEXIF" ]] || {
    printf 'expected %s to resolve libexif.so.12 from %s, got %s\n' "$target" "$ACTIVE_LIBEXIF" "$resolved" >&2
    ldd "$target" >&2
    exit 1
  }
}

validate_dependents() {
  local expected_count actual_count matched=0
  local name

  expected_count="${#REQUIRED_DEPENDENTS[@]}"
  actual_count="$(jq -r '.dependents | length' "$ROOT/dependents.json")"
  [[ "$actual_count" == "$expected_count" ]] || {
    printf 'dependents.json count mismatch: expected %s, found %s\n' "$expected_count" "$actual_count" >&2
    exit 1
  }

  for name in "${REQUIRED_DEPENDENTS[@]}"; do
    jq -e --arg name "$name" '.dependents[] | select(.name == $name)' "$ROOT/dependents.json" >/dev/null
  done

  if [[ -n "$ONLY_FILTER" ]]; then
    matched=0
    for name in "${REQUIRED_DEPENDENTS[@]}"; do
      if [[ "$name" == "$ONLY_FILTER" ]]; then
        matched=1
        break
      fi
    done
    [[ "$matched" -eq 1 ]] || die "unknown dependent for --only: $ONLY_FILTER"
  fi
}

build_original_packages() {
  local runtime_matches dev_matches

  log_step "Building original libexif Debian packages"

  rm -rf "$SOURCE_COPY"
  rm -f /tmp/libexif12_*.deb /tmp/libexif-dev_*.deb /tmp/libexif-doc_*.deb /tmp/libexif12-dbgsym_*.deb
  cp -a "$ROOT/original" "$SOURCE_COPY"

  if ! (
    cd "$SOURCE_COPY"
    dpkg-buildpackage -us -uc -b >/tmp/libexif-build.log 2>&1
  ); then
    cat /tmp/libexif-build.log >&2
    exit 1
  fi

  runtime_matches="$(find /tmp -maxdepth 1 -type f -name 'libexif12_*.deb' | LC_ALL=C sort)"
  dev_matches="$(find /tmp -maxdepth 1 -type f -name 'libexif-dev_*.deb' | LC_ALL=C sort)"

  [[ "$(printf '%s\n' "$runtime_matches" | sed '/^$/d' | wc -l)" -eq 1 ]] || die "expected exactly one libexif12 Debian package"
  [[ "$(printf '%s\n' "$dev_matches" | sed '/^$/d' | wc -l)" -eq 1 ]] || die "expected exactly one libexif-dev Debian package"

  ORIGINAL_RUNTIME_DEB="$(printf '%s\n' "$runtime_matches" | head -n1)"
  ORIGINAL_DEV_DEB="$(printf '%s\n' "$dev_matches" | head -n1)"
  ORIGINAL_RUNTIME_VERSION="$(dpkg-deb -f "$ORIGINAL_RUNTIME_DEB" Version)"
  ORIGINAL_DEV_VERSION="$(dpkg-deb -f "$ORIGINAL_DEV_DEB" Version)"

  printf 'ORIGINAL_RUNTIME_DEB=%s\n' "$ORIGINAL_RUNTIME_DEB"
  printf 'ORIGINAL_DEV_DEB=%s\n' "$ORIGINAL_DEV_DEB"
  printf 'ORIGINAL_RUNTIME_VERSION=%s\n' "$ORIGINAL_RUNTIME_VERSION"
  printf 'ORIGINAL_DEV_VERSION=%s\n' "$ORIGINAL_DEV_VERSION"
}

install_original_packages() {
  local extract_dir deb_lib

  log_step "Installing original libexif Debian packages"

  dpkg -i "$ORIGINAL_RUNTIME_DEB" "$ORIGINAL_DEV_DEB" >/tmp/libexif-install.log 2>&1 || {
    cat /tmp/libexif-install.log >&2
    exit 1
  }
  ldconfig

  [[ "$(dpkg-query -W -f='${Version}\n' libexif12)" == "$ORIGINAL_RUNTIME_VERSION" ]] || die "active libexif12 version mismatch"
  [[ "$(dpkg-query -W -f='${Version}\n' libexif-dev)" == "$ORIGINAL_DEV_VERSION" ]] || die "active libexif-dev version mismatch"

  ACTIVE_LIBEXIF="$(ldconfig -p | awk '/libexif\.so\.12 \(/{ print $NF; exit }')"
  [[ -n "$ACTIVE_LIBEXIF" ]] || die "unable to locate active libexif.so.12 via ldconfig"
  ACTIVE_LIBEXIF="$(readlink -f "$ACTIVE_LIBEXIF")"

  extract_dir="$(mktemp -d)"
  dpkg-deb -x "$ORIGINAL_RUNTIME_DEB" "$extract_dir"
  deb_lib="$(find "$extract_dir" -type f -path '*/libexif.so.12*' | LC_ALL=C sort | head -n1)"
  [[ -n "$deb_lib" ]] || die "unable to locate libexif.so.12 inside built runtime package"
  cmp -s "$ACTIVE_LIBEXIF" "$deb_lib" || die "installed libexif.so.12 does not match the locally built package payload"

  printf 'ACTIVE_LIBEXIF=%s\n' "$ACTIVE_LIBEXIF"
}

create_test_fixtures() {
  local tmp_step1="$FIXTURE_ROOT/generated-step1.jpg"

  log_step "Creating test fixtures"

  rm -rf "$FIXTURE_ROOT" "$TEST_ROOT"
  mkdir -p "$FIXTURE_ROOT" "$GPHOTO_FIXTURE_DIR" "$TEST_ROOT"

  convert -size 8x6 xc:red "$FIXTURE_ROOT/plain.jpg"
  exif --create-exif --ifd=0 --tag Orientation --set-value 6 -o "$tmp_step1" "$FIXTURE_ROOT/plain.jpg" >/tmp/libexif-fixture-create.log 2>&1
  exif --ifd=0 --tag DateTime --set-value '2024:01:02 03:04:05' -o "$GENERATED_FIXTURE" "$tmp_step1" >/tmp/libexif-fixture-update.log 2>&1
  cp "$FUJI_FIXTURE" "$GPHOTO_FIXTURE_DIR/fuji.jpg"

  require_nonempty_file "$GENERATED_FIXTURE"
  require_nonempty_file "$GPHOTO_FIXTURE_DIR/fuji.jpg"
}

run_eog_plugin_smoke() {
  local plugin_name="$1"
  local image_path="$2"
  local log_dir="$3"

  cat >"$log_dir/run-eog-plugin.sh" <<EOF
set -euo pipefail
gsettings set org.gnome.eog.plugins active-plugins "['$plugin_name']"
timeout 8 xvfb-run -a eog "$image_path" >"$log_dir/eog.stdout" 2>"$log_dir/eog.stderr" || status=\$?
status=\${status:-0}
printf '%s\n' "\$status" >"$log_dir/status"
gsettings get org.gnome.eog.plugins active-plugins >"$log_dir/active.txt"
EOF

  if ! dbus-run-session -- bash "$log_dir/run-eog-plugin.sh" >"$log_dir/dbus.log" 2>&1; then
    cat "$log_dir/dbus.log" >&2
    exit 1
  fi

  assert_status_equals 124 "$(cat "$log_dir/status")" "eog with plugin $plugin_name"
  require_contains "$log_dir/active.txt" "$plugin_name"
}

test_exif() {
  local dir="$1"

  assert_binary_uses_active_libexif /usr/bin/exif
  exif -m "$GENERATED_FIXTURE" >"$dir/exif.out"
  require_contains "$dir/exif.out" $'Orientation\tRight-top'
  require_contains "$dir/exif.out" $'Date and Time\t2024:01:02 03:04:05'
}

test_exiftran() {
  local dir="$1"

  assert_binary_uses_active_libexif /usr/bin/exiftran
  cp "$GENERATED_FIXTURE" "$dir/oriented.jpg"
  exif -m "$dir/oriented.jpg" >"$dir/before.out"
  require_contains "$dir/before.out" $'Orientation\tRight-top'

  exiftran -ai "$dir/oriented.jpg" >"$dir/exiftran.log" 2>&1
  exif -m "$dir/oriented.jpg" >"$dir/after.out"
  identify "$dir/oriented.jpg" >"$dir/identify.out"

  require_contains "$dir/after.out" $'Orientation\tTop-left'
  require_contains "$dir/identify.out" 'JPEG 6x8 6x8+0+0'
}

test_eog_plugin_exif_display() {
  local dir="$1"

  assert_binary_uses_active_libexif /usr/lib/x86_64-linux-gnu/eog/plugins/libexif-display.so
  run_eog_plugin_smoke "exif-display" "$FUJI_FIXTURE" "$dir"
}

test_eog_plugin_map() {
  local dir="$1"

  assert_binary_uses_active_libexif /usr/lib/x86_64-linux-gnu/eog/plugins/libmap.so
  run_eog_plugin_smoke "map" "$GENERATED_FIXTURE" "$dir"
}

test_tracker_extract() {
  local dir="$1"

  tracker3 extract "$GENERATED_FIXTURE" >"$dir/tracker.out" 2>"$dir/tracker.err"
  require_contains "$dir/tracker.out" 'nfo:width 8'
  require_contains "$dir/tracker.out" 'nfo:height 6'
  require_not_contains "$dir/tracker.err" 'No metadata or extractor modules found'
}

test_shotwell() {
  local dir="$1"

  assert_binary_uses_active_libexif /usr/bin/shotwell
  xvfb-run -a shotwell --show-metadata "$FUJI_FIXTURE" >"$dir/shotwell.out" 2>"$dir/shotwell.err"
  require_contains "$dir/shotwell.out" 'Exif.Image.Model                                                FinePix Z33WP'
  require_contains "$dir/shotwell.out" 'Exif.Photo.PixelXDimension                                      640'
}

test_foxtrotgps() {
  local dir="$1"
  local status=0

  assert_binary_uses_active_libexif /usr/bin/foxtrotgps
  xvfb-run -a timeout 5 foxtrotgps --lat 33.4484 --lon -112.0740 >"$dir/foxtrotgps.out" 2>"$dir/foxtrotgps.err" || status=$?
  assert_status_equals 124 "${status:-0}" "foxtrotgps"
}

test_gphoto2() {
  local dir="$1"

  assert_binary_uses_active_libexif /usr/bin/gphoto2
  gphoto2 --camera "Directory Browse" --port "disk:$GPHOTO_FIXTURE_DIR" --show-exif 1 >"$dir/gphoto2-exif.out"
  gphoto2 --camera "Directory Browse" --port "disk:$GPHOTO_FIXTURE_DIR" --show-info 1 >"$dir/gphoto2-info.out"
  require_contains "$dir/gphoto2-exif.out" 'Model               |FinePix Z33WP'
  require_contains "$dir/gphoto2-exif.out" 'DateTimeOriginal    |2009:03:25 03:27:25'
  require_contains "$dir/gphoto2-info.out" "Mime type:   'image/jpeg'"
}

test_gtkam() {
  local dir="$1"
  local status=0

  assert_binary_uses_active_libexif /usr/bin/gtkam
  xvfb-run -a timeout 5 gtkam >"$dir/gtkam.out" 2>"$dir/gtkam.err" || status=$?
  assert_status_equals 124 "${status:-0}" "gtkam"
}

test_minidlna() {
  local dir="$1"
  local media_dir="$dir/media"
  local db_dir="$dir/db"
  local log_dir="$dir/log"
  local status=0

  assert_binary_uses_active_libexif /usr/sbin/minidlnad

  mkdir -p "$media_dir" "$db_dir" "$log_dir"
  cp "$GENERATED_FIXTURE" "$media_dir/photo.jpg"
  cat >"$dir/minidlna.conf" <<EOF
media_dir=P,$media_dir
friendly_name=libexif-test
port=49152
db_dir=$db_dir
log_dir=$log_dir
inotify=no
album_art_names=Cover.jpg/cover.jpg/AlbumArtSmall.jpg/albumartsmall.jpg
EOF

  timeout 15 minidlnad -d -R -f "$dir/minidlna.conf" >"$dir/minidlna.stdout" 2>"$dir/minidlna.stderr" || status=$?
  assert_status_equals 124 "${status:-0}" "minidlnad"
  require_nonempty_file "$db_dir/files.db"
  sqlite3 "$db_dir/files.db" "select PATH || '|' || MIME from DETAILS where PATH like '%/photo.jpg';" >"$dir/minidlna-sqlite.out"
  require_contains "$dir/minidlna-sqlite.out" 'photo.jpg|image/jpeg'
}

test_gerbera() {
  local dir="$1"
  local home_dir="$dir/home"
  local status=0

  assert_binary_uses_active_libexif /usr/bin/gerbera

  mkdir -p "$home_dir"
  gerbera --create-config >"$dir/config.xml"
  timeout 10 gerbera --config "$dir/config.xml" --home "$home_dir" --offline >"$dir/gerbera.stdout" 2>"$dir/gerbera.stderr" || status=$?
  assert_status_equals 124 "${status:-0}" "gerbera"
  require_contains "$dir/gerbera.stdout" 'Configuration check succeeded.'
  require_contains "$dir/gerbera.stdout" 'database created successfully.'
  require_nonempty_file "$home_dir/gerbera.db"
}

test_ruby_exif() {
  local dir="$1"

  assert_binary_uses_active_libexif /usr/lib/x86_64-linux-gnu/ruby/vendor_ruby/3.2.0/exif.so
  ruby -rexif -e 'x = Exif.new(ARGV[0]); puts x["Model"]; puts x["Date and Time"]' "$FUJI_FIXTURE" >"$dir/ruby-exif.out"
  require_contains "$dir/ruby-exif.out" 'FinePix Z33WP'
  require_contains "$dir/ruby-exif.out" '2009:03:25 03:27:25'
}

test_libexif_gtk3() {
  local dir="$1"

  cat >"$dir/test-libexif-gtk.c" <<'EOF'
#include <gtk/gtk.h>
#include <libexif/exif-data.h>
#include <libexif-gtk/gtk-exif-browser.h>

int main(int argc, char **argv) {
  ExifData *data;
  GtkWidget *window;
  GtkWidget *browser;

  gtk_init(&argc, &argv);
  data = exif_data_new_from_file(argv[1]);
  if (!data) {
    g_printerr("failed to load exif data\n");
    return 1;
  }

  window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
  browser = gtk_exif_browser_new();
  gtk_container_add(GTK_CONTAINER(window), browser);
  gtk_exif_browser_set_data(GTK_EXIF_BROWSER(browser), data);
  gtk_widget_show_all(window);
  while (gtk_events_pending()) {
    gtk_main_iteration();
  }

  g_print("browser-ready\n");
  exif_data_unref(data);
  return 0;
}
EOF

  cc -o "$dir/test-libexif-gtk" "$dir/test-libexif-gtk.c" $(pkg-config --cflags --libs libexif-gtk3)
  assert_binary_uses_active_libexif "$dir/test-libexif-gtk"
  xvfb-run -a "$dir/test-libexif-gtk" "$FUJI_FIXTURE" >"$dir/libexif-gtk.out" 2>"$dir/libexif-gtk.err"
  require_contains "$dir/libexif-gtk.out" 'browser-ready'
}

test_camlimages() {
  local dir="$1"

  cat >"$dir/camlimages_exif_test.ml" <<'EOF'
let read_file path =
  let ic = open_in_bin path in
  let len = in_channel_length ic in
  let buf = really_input_string ic len in
  close_in ic;
  buf

let be16 s off =
  ((Char.code s.[off]) lsl 8) lor (Char.code s.[off + 1])

(* CamlImages' Exif parser operates on the raw APP1 EXIF payload. *)
let exif_blob path =
  let data = read_file path in
  let len = String.length data in
  let rec loop off =
    if off + 4 >= len then failwith "no exif app1 segment"
    else if Char.code data.[off] <> 0xff then loop (off + 1)
    else
      let marker = Char.code data.[off + 1] in
      if marker = 0xe1 then
        let seg_len = be16 data (off + 2) in
        let body_off = off + 4 in
        let body_len = seg_len - 2 in
        let body = String.sub data body_off body_len in
        if String.length body >= 6 && String.sub body 0 6 = "Exif\000\000" then
          body
        else
          loop (body_off + body_len)
      else if marker = 0xd8 || marker = 0xd9 || (marker >= 0xd0 && marker <= 0xd7) then
        loop (off + 2)
      else
        let seg_len = be16 data (off + 2) in
        loop (off + 2 + seg_len)
  in
  loop 0

let () =
  let blob = exif_blob Sys.argv.(1) in
  let data = Exif.Data.from_string blob in
  match Exif.Analyze.datetime data with
  | Some (`Ok dt) -> print_endline (Exif.DateTime.to_string dt)
  | Some _ -> failwith "unexpected datetime parse result"
  | None -> failwith "missing datetime"
EOF

  ocamlfind ocamlopt -package camlimages.exif -linkpkg -o "$dir/camlimages-exif-test" "$dir/camlimages_exif_test.ml"
  "$dir/camlimages-exif-test" "$GENERATED_FIXTURE" >"$dir/camlimages.out"
  require_contains "$dir/camlimages.out" '2024:01:02 03:04:05'
}

test_imagemagick() {
  local dir="$1"

  identify -verbose "$GENERATED_FIXTURE" >"$dir/imagemagick.out"
  require_contains "$dir/imagemagick.out" 'Orientation: RightTop'
}

run_named_test() {
  local name="$1"
  local function_name="$2"
  local dir

  if ! should_run "$name"; then
    return 0
  fi

  dir="$(reset_test_dir "$name")"
  log_step "Testing $name"
  "$function_name" "$dir"
}

validate_dependents
build_original_packages
install_original_packages
create_test_fixtures

run_named_test "exif" test_exif
run_named_test "exiftran" test_exiftran
run_named_test "eog-plugin-exif-display" test_eog_plugin_exif_display
run_named_test "eog-plugin-map" test_eog_plugin_map
run_named_test "tracker-extract" test_tracker_extract
run_named_test "Shotwell" test_shotwell
run_named_test "FoxtrotGPS" test_foxtrotgps
run_named_test "gphoto2" test_gphoto2
run_named_test "GTKam" test_gtkam
run_named_test "MiniDLNA" test_minidlna
run_named_test "Gerbera" test_gerbera
run_named_test "ruby-exif" test_ruby_exif
run_named_test "libexif-gtk3" test_libexif_gtk3
run_named_test "CamlImages" test_camlimages
run_named_test "ImageMagick" test_imagemagick

log_step "All requested downstream tests passed"
CONTAINER_SCRIPT
