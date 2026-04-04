#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
IMAGE_TAG="${LIBEXIF_ORIGINAL_TEST_IMAGE:-libexif-original-test:ubuntu24.04}"
ONLY=""

usage() {
  cat <<'EOF'
usage: test-original.sh [--only <dependent-name>]

Builds the local Debian packages from ./safe inside an Ubuntu 24.04 Docker
container, installs them, and smoke-tests the dependent software recorded in
dependents.json. The safe package build still stages the vendored ./original
tree alongside ./safe because the Rust build reuses those helper sources.

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

[[ -d "$ROOT/safe" ]] || {
  echo "missing safe source tree" >&2
  exit 1
}

[[ -d "$ROOT/original" ]] || {
  echo "missing original helper source tree" >&2
  exit 1
}

[[ -f "$ROOT/dependents.json" ]] || {
  echo "missing dependents.json" >&2
  exit 1
}

docker build -t "$IMAGE_TAG" - <<'DOCKERFILE'
FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/root/.rustup
ENV CARGO_HOME=/root/.cargo
ENV PATH=/root/.cargo/bin:${PATH}

RUN sed -i 's/^Types: deb$/Types: deb deb-src/' /etc/apt/sources.list.d/ubuntu.sources \
 && apt-get update \
 && apt-get install -y --no-install-recommends \
      build-essential \
      ca-certificates \
      curl \
      dbus-x11 \
      debhelper \
      dpkg-dev \
      fakeroot \
      exif \
      exiftran \
      eog \
      eog-dev \
      eog-plugins \
      file \
      foxtrotgps \
      gphoto2 \
      gtkam \
      imagemagick \
      jq \
      cargo \
      libcamlimages-ocaml \
      libcamlimages-ocaml-dev \
      libexif-gtk-dev \
      libexif-gtk3-5 \
      libgphoto2-dev \
      libgtk-3-dev \
      minidlna \
      ocaml-findlib \
      ocaml-nox \
      pkg-config \
      python3-dogtail \
      rustc \
      ruby \
      ruby-exif \
      shotwell \
      sqlite3 \
      tracker-extract \
      xdotool \
      xauth \
      xvfb \
      gerbera \
 && apt-get build-dep -y --no-install-recommends libexif \
 && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --profile minimal --default-toolchain stable \
 && cargo --version \
 && rustc --version \
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
SOURCE_ROOT=/tmp/libexif-safe-src
SOURCE_COPY="$SOURCE_ROOT/safe"
ORIGINAL_HELPER_COPY="$SOURCE_ROOT/original"
FIXTURE_ROOT=/tmp/libexif-fixtures
TEST_ROOT=/tmp/libexif-dependent-tests
FUJI_FIXTURE="$ROOT/safe/tests/testdata/fuji_makernote_variant_1.jpg"
GENERATED_FIXTURE="$FIXTURE_ROOT/generated-exif.jpg"
GPS_FIXTURE="$FIXTURE_ROOT/generated-gps.jpg"
GPHOTO_FIXTURE_DIR="$FIXTURE_ROOT/gphoto-camera"
EOG_PLUGIN_DIR="/usr/lib/$MULTIARCH/eog/plugins"
RUBY_VENDORARCHDIR="$(ruby -rrbconfig -e 'print RbConfig::CONFIG["vendorarchdir"]')"
ACTIVE_LIBEXIF=""
PACKAGE_RUNTIME_DEB=""
PACKAGE_DEV_DEB=""
PACKAGE_RUNTIME_VERSION=""
PACKAGE_DEV_VERSION=""

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

insert_block_before_marker() {
  local file="$1"
  local marker="$2"
  local block_file="$3"

  awk -v block="$block_file" -v marker="$marker" '
    index($0, marker) {
      while ((getline line < block) > 0) {
        print line
      }
      close(block)
    }
    { print }
  ' "$file" >"$file.new"
  mv "$file.new" "$file"
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

build_safe_packages() {
  local runtime_matches dev_matches

  log_step "Building safe libexif Debian packages"

  rm -rf "$SOURCE_ROOT"
  mkdir -p "$SOURCE_ROOT"
  cp -a "$ROOT/safe" "$SOURCE_COPY"
  cp -a "$ROOT/original" "$ORIGINAL_HELPER_COPY"

  if ! (
    cd "$SOURCE_COPY"
    dpkg-buildpackage -us -uc -b >/tmp/libexif-build.log 2>&1
  ); then
    cat /tmp/libexif-build.log >&2
    exit 1
  fi

  runtime_matches="$(find "$SOURCE_ROOT" -maxdepth 1 -type f -name 'libexif12_*.deb' | LC_ALL=C sort)"
  dev_matches="$(find "$SOURCE_ROOT" -maxdepth 1 -type f -name 'libexif-dev_*.deb' | LC_ALL=C sort)"

  [[ "$(printf '%s\n' "$runtime_matches" | sed '/^$/d' | wc -l)" -eq 1 ]] || die "expected exactly one libexif12 Debian package"
  [[ "$(printf '%s\n' "$dev_matches" | sed '/^$/d' | wc -l)" -eq 1 ]] || die "expected exactly one libexif-dev Debian package"

  PACKAGE_RUNTIME_DEB="$(printf '%s\n' "$runtime_matches" | head -n1)"
  PACKAGE_DEV_DEB="$(printf '%s\n' "$dev_matches" | head -n1)"
  PACKAGE_RUNTIME_VERSION="$(dpkg-deb -f "$PACKAGE_RUNTIME_DEB" Version)"
  PACKAGE_DEV_VERSION="$(dpkg-deb -f "$PACKAGE_DEV_DEB" Version)"

  printf 'PACKAGE_RUNTIME_DEB=%s\n' "$PACKAGE_RUNTIME_DEB"
  printf 'PACKAGE_DEV_DEB=%s\n' "$PACKAGE_DEV_DEB"
  printf 'PACKAGE_RUNTIME_VERSION=%s\n' "$PACKAGE_RUNTIME_VERSION"
  printf 'PACKAGE_DEV_VERSION=%s\n' "$PACKAGE_DEV_VERSION"
}

install_safe_packages() {
  local extract_dir deb_lib

  log_step "Installing safe libexif Debian packages"

  dpkg -i "$PACKAGE_RUNTIME_DEB" "$PACKAGE_DEV_DEB" >/tmp/libexif-install.log 2>&1 || {
    cat /tmp/libexif-install.log >&2
    exit 1
  }
  ldconfig

  [[ "$(dpkg-query -W -f='${Version}\n' libexif12)" == "$PACKAGE_RUNTIME_VERSION" ]] || die "active libexif12 version mismatch"
  [[ "$(dpkg-query -W -f='${Version}\n' libexif-dev)" == "$PACKAGE_DEV_VERSION" ]] || die "active libexif-dev version mismatch"

  ACTIVE_LIBEXIF="$(ldconfig -p | awk '/libexif\.so\.12 \(/{ print $NF; exit }')"
  [[ -n "$ACTIVE_LIBEXIF" ]] || die "unable to locate active libexif.so.12 via ldconfig"
  ACTIVE_LIBEXIF="$(readlink -f "$ACTIVE_LIBEXIF")"

  extract_dir="$(mktemp -d)"
  dpkg-deb -x "$PACKAGE_RUNTIME_DEB" "$extract_dir"
  deb_lib="$(find "$extract_dir" -type f -path '*/libexif.so.12*' | LC_ALL=C sort | head -n1)"
  [[ -n "$deb_lib" ]] || die "unable to locate libexif.so.12 inside built runtime package"
  cmp -s "$ACTIVE_LIBEXIF" "$deb_lib" || die "installed libexif.so.12 does not match the locally built package payload"

  printf 'ACTIVE_LIBEXIF=%s\n' "$ACTIVE_LIBEXIF"
}

build_fixture_writer() {
  local output_path="$1"

  cat >"$output_path" <<'EOF'
#include <libexif/exif-data.h>
#include <libexif/exif-utils.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static ExifEntry *init_tag(ExifData *exif, ExifIfd ifd, ExifTag tag)
{
	ExifEntry *entry = exif_content_get_entry(exif->ifd[ifd], tag);

	if (!entry) {
		entry = exif_entry_new();
		if (!entry) return NULL;
		entry->tag = tag;
		exif_content_add_entry(exif->ifd[ifd], entry);
		exif_entry_initialize(entry, tag);
		exif_entry_unref(entry);
	}
	return entry;
}

static ExifEntry *create_tag(ExifData *exif, ExifIfd ifd, ExifTag tag,
			     ExifFormat format, unsigned long components)
{
	ExifMem *mem = exif_mem_new_default();
	ExifEntry *entry;
	unsigned int size = components * exif_format_get_size(format);

	if (!mem) return NULL;
	entry = exif_entry_new_mem(mem);
	if (!entry) {
		exif_mem_unref(mem);
		return NULL;
	}
	entry->data = exif_mem_alloc(mem, size);
	if (!entry->data) {
		exif_entry_unref(entry);
		exif_mem_unref(mem);
		return NULL;
	}

	memset(entry->data, 0, size);
	entry->size = size;
	entry->tag = tag;
	entry->components = components;
	entry->format = format;
	exif_content_add_entry(exif->ifd[ifd], entry);
	exif_mem_unref(mem);
	exif_entry_unref(entry);
	return entry;
}

static ExifEntry *create_ascii_tag(ExifData *exif, ExifIfd ifd, ExifTag tag,
				   const char *value)
{
	ExifEntry *entry = create_tag(exif, ifd, tag, EXIF_FORMAT_ASCII,
				      strlen(value) + 1);
	if (!entry) return NULL;
	memcpy(entry->data, value, strlen(value) + 1);
	return entry;
}

static ExifEntry *create_rational_tag(ExifData *exif, ExifIfd ifd, ExifTag tag,
				      unsigned long components,
				      const ExifRational *values,
				      ExifByteOrder order)
{
	ExifEntry *entry = create_tag(exif, ifd, tag, EXIF_FORMAT_RATIONAL,
				      components);
	unsigned long i;

	if (!entry) return NULL;
	for (i = 0; i < components; ++i) {
		exif_set_rational(
			entry->data + i * exif_format_get_size(EXIF_FORMAT_RATIONAL),
			order, values[i]);
	}
	return entry;
}

static int copy_with_exif(const char *input_path, const char *output_path,
			  ExifData *exif)
{
	FILE *in = NULL;
	FILE *out = NULL;
	unsigned char *image = NULL;
	unsigned char *exif_data = NULL;
	unsigned int exif_len = 0;
	long image_size;
	int rc = 1;

	in = fopen(input_path, "rb");
	if (!in) {
		perror(input_path);
		goto cleanup;
	}
	if (fseek(in, 0, SEEK_END) != 0) goto cleanup;
	image_size = ftell(in);
	if (image_size < 4) goto cleanup;
	if (fseek(in, 0, SEEK_SET) != 0) goto cleanup;

	image = malloc((size_t) image_size);
	if (!image) goto cleanup;
	if (fread(image, 1, (size_t) image_size, in) != (size_t) image_size)
		goto cleanup;
	if (image[0] != 0xff || image[1] != 0xd8) {
		fprintf(stderr, "input is not a JPEG: %s\n", input_path);
		goto cleanup;
	}

	exif_data_save_data(exif, &exif_data, &exif_len);
	if (!exif_data || exif_len == 0 || exif_len + 2 > 0xffff)
		goto cleanup;

	out = fopen(output_path, "wb");
	if (!out) {
		perror(output_path);
		goto cleanup;
	}
	fputc(0xff, out);
	fputc(0xd8, out);
	fputc(0xff, out);
	fputc(0xe1, out);
	fputc(((exif_len + 2) >> 8) & 0xff, out);
	fputc((exif_len + 2) & 0xff, out);
	if (fwrite(exif_data, 1, exif_len, out) != exif_len) goto cleanup;
	if (fwrite(image + 2, 1, (size_t) image_size - 2, out) !=
	    (size_t) image_size - 2)
		goto cleanup;
	rc = 0;

cleanup:
	if (in) fclose(in);
	if (out) fclose(out);
	free(image);
	free(exif_data);
	return rc;
}

int main(int argc, char **argv)
{
	ExifData *exif;
	ExifEntry *entry;
	ExifByteOrder order = EXIF_BYTE_ORDER_INTEL;
	const char *mode;
	int with_gps;
	unsigned short orientation;
	ExifRational latitude[3] = {{33, 1}, {26, 1}, {2736, 100}};
	ExifRational longitude[3] = {{112, 1}, {4, 1}, {2640, 100}};

	if (argc != 4) {
		fprintf(stderr, "usage: %s MODE INPUT.jpg OUTPUT.jpg\n", argv[0]);
		return 2;
	}
	mode = argv[1];
	if (strcmp(mode, "basic") == 0) {
		with_gps = 0;
		orientation = 6;
	} else if (strcmp(mode, "gps") == 0) {
		with_gps = 1;
		orientation = 1;
	} else {
		fprintf(stderr, "unknown mode: %s\n", mode);
		return 2;
	}

	exif = exif_data_new();
	if (!exif) return 1;
	exif_data_set_option(exif, EXIF_DATA_OPTION_FOLLOW_SPECIFICATION);
	exif_data_set_data_type(exif, EXIF_DATA_TYPE_COMPRESSED);
	exif_data_set_byte_order(exif, order);
	exif_data_fix(exif);

	entry = init_tag(exif, EXIF_IFD_0, EXIF_TAG_ORIENTATION);
	exif_set_short(entry->data, order, orientation);
	entry = init_tag(exif, EXIF_IFD_0, EXIF_TAG_DATE_TIME);
	memcpy(entry->data, "2024:01:02 03:04:05", 20);
	if (with_gps) {
		create_ascii_tag(exif, EXIF_IFD_0, EXIF_TAG_MODEL, "libexif-gps");
		entry = init_tag(exif, EXIF_IFD_EXIF, EXIF_TAG_DATE_TIME_ORIGINAL);
		memcpy(entry->data, "2024:01:02 03:04:05", 20);
		entry = init_tag(exif, EXIF_IFD_EXIF, EXIF_TAG_PIXEL_X_DIMENSION);
		exif_set_long(entry->data, order, 8);
		entry = init_tag(exif, EXIF_IFD_EXIF, EXIF_TAG_PIXEL_Y_DIMENSION);
		exif_set_long(entry->data, order, 6);

		create_ascii_tag(exif, EXIF_IFD_GPS, EXIF_TAG_GPS_LATITUDE_REF,
				 "N");
		create_rational_tag(exif, EXIF_IFD_GPS, EXIF_TAG_GPS_LATITUDE, 3,
				    latitude, order);
		create_ascii_tag(exif, EXIF_IFD_GPS, EXIF_TAG_GPS_LONGITUDE_REF,
				 "W");
		create_rational_tag(exif, EXIF_IFD_GPS, EXIF_TAG_GPS_LONGITUDE, 3,
				    longitude, order);
		create_ascii_tag(exif, EXIF_IFD_GPS, EXIF_TAG_GPS_MAP_DATUM,
				 "WGS-84");
		create_ascii_tag(exif, EXIF_IFD_GPS, EXIF_TAG_GPS_DATE_STAMP,
				 "2024:01:02");
	}

	if (copy_with_exif(argv[2], argv[3], exif) != 0) {
		exif_data_unref(exif);
		return 1;
	}

	exif_data_unref(exif);
	return 0;
}
EOF
}

create_test_fixtures() {
  local fixture_writer_src="$FIXTURE_ROOT/libexif-fixture-writer.c"
  local fixture_writer_bin="$FIXTURE_ROOT/libexif-fixture-writer"

  log_step "Creating test fixtures"

  rm -rf "$FIXTURE_ROOT" "$TEST_ROOT"
  mkdir -p "$FIXTURE_ROOT" "$GPHOTO_FIXTURE_DIR" "$TEST_ROOT"

  convert -size 8x6 xc:red "$FIXTURE_ROOT/plain.jpg"
  build_fixture_writer "$fixture_writer_src"
  cc -Wall -Wextra -o "$fixture_writer_bin" "$fixture_writer_src" $(pkg-config --cflags --libs libexif)
  "$fixture_writer_bin" basic "$FIXTURE_ROOT/plain.jpg" "$GENERATED_FIXTURE"
  "$fixture_writer_bin" gps "$FIXTURE_ROOT/plain.jpg" "$GPS_FIXTURE"
  cp "$FUJI_FIXTURE" "$GPHOTO_FIXTURE_DIR/fuji.jpg"

  require_nonempty_file "$GENERATED_FIXTURE"
  require_nonempty_file "$GPS_FIXTURE"
  require_nonempty_file "$GPHOTO_FIXTURE_DIR/fuji.jpg"
}

build_foxtrotgps_runtime_probe() {
  local output_path="$1"

  cat >"$output_path" <<'EOF'
#define _GNU_SOURCE

#include <dlfcn.h>
#include <gtk/gtk.h>

#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>
#include <unistd.h>

typedef struct _GladeXML GladeXML;

extern GladeXML *gladexml;
extern GtkWidget *dialog_image_data;
extern GtkWidget *dialog_photo_correlate;
extern GtkWidget *lookup_widget (GtkWidget *widget, const gchar *widget_name);
extern void geo_photos_open_dialog_photo_correlate (void);
extern void geo_photos_open_dialog_image_data (void);
extern void geocode_set_photodir (char *photodir, GtkWidget *widget);

static const char *expected_datetime = "2024:01:02 03:04:05";
static const char *photo_dir;
static int stage;
static int ticks;

static gboolean
probe_cb (gpointer user_data)
{
	GtkWidget *button;
	GtkWidget *camera_label;
	GtkWidget *gps_label;
	const gchar *camera_text;
	const gchar *gps_text;

	(void) user_data;

	switch (stage) {
	case 0:
		if (gladexml == NULL)
			break;
		geo_photos_open_dialog_photo_correlate ();
		stage = 1;
		ticks = 0;
		break;
	case 1:
		if (dialog_photo_correlate == NULL)
			break;
		button = lookup_widget (dialog_photo_correlate, "button40");
		if (button == NULL)
			break;
		geocode_set_photodir ((char *) photo_dir, button);
		stage = 2;
		ticks = 0;
		break;
	case 2:
		if (dialog_photo_correlate == NULL)
			break;
		geo_photos_open_dialog_image_data ();
		stage = 3;
		ticks = 0;
		break;
	case 3:
		if (dialog_image_data == NULL)
			break;
		camera_label = lookup_widget (dialog_image_data, "label163");
		gps_label = lookup_widget (dialog_image_data, "label171");
		if (camera_label == NULL || gps_label == NULL)
			break;

		camera_text = gtk_label_get_text (GTK_LABEL (camera_label));
		gps_text = gtk_label_get_text (GTK_LABEL (gps_label));
		g_print ("CAMERA=%s\n", camera_text ? camera_text : "");
		g_print ("GPS=%s\n", gps_text ? gps_text : "");
		fflush (stdout);

		if (camera_text && strcmp (camera_text, expected_datetime) == 0)
			_exit (0);

		break;
	}

	if (++ticks > 60) {
		g_printerr ("foxtrotgps probe timed out in stage %d\n", stage);
		_exit (1);
	}

	return TRUE;
}

void
gtk_main (void)
{
	static void (*real_gtk_main) (void);

	if (real_gtk_main == NULL)
		real_gtk_main = dlsym (RTLD_NEXT, "gtk_main");

	photo_dir = getenv ("LIBEXIF_FOXTROTGPS_PHOTO_DIR");
	if (photo_dir == NULL || *photo_dir == '\0') {
		g_printerr ("missing LIBEXIF_FOXTROTGPS_PHOTO_DIR\n");
		_exit (2);
	}

	unsetenv ("LD_PRELOAD");
	g_timeout_add (250, probe_cb, NULL);
	real_gtk_main ();
}
EOF
}

build_gtkam_runtime_probe() {
  local output_path="$1"

  cat >"$output_path" <<'EOF'
#define _GNU_SOURCE

#include <dlfcn.h>
#include <gtk/gtk.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

typedef struct _GtkamDialogPrivate GtkamDialogPrivate;
typedef struct _GtkamListPrivate GtkamListPrivate;

typedef struct _GtkamDialog {
	GtkDialog parent;
	GtkWidget *image;
	GtkWidget *vbox;
	GtkamDialogPrivate *priv;
} GtkamDialog;

typedef struct _GtkamChooserPrivate {
	void *al;
	void *il;

	GtkWidget *label_speed;
	GtkWidget *check_multi;
	GtkWidget *button_add;
	GtkWidget *table;
	GtkEntry *entry_model;
	GtkEntry *entry_port;
	GtkEntry *entry_speed;
	GtkCombo *combo_model;
	GtkCombo *combo_port;
	GtkCombo *combo_speed;

	GtkWidget *ok;

	GtkTooltips *tooltips;

	gboolean needs_update;
} GtkamChooserPrivate;

typedef struct _GtkamChooser {
	GtkamDialog parent;
	GtkWidget *apply_button;
	GtkamChooserPrivate *priv;
} GtkamChooser;

struct _GtkamListPrivate {
	GtkListStore *store;
	gboolean thumbnails;
	GtkTreeViewColumn *col_previews;
	GtkItemFactory *factory;
	GtkTreeIter iter;
	void *head;
	void *tail;
};

typedef struct _GtkamList {
	GtkTreeView parent;
	GtkamListPrivate *priv;
} GtkamList;

static const char *camera_model;
static const char *camera_port;
static GtkWidget *saved_menu_item;
static int stage;
static int ticks;
static gboolean debug_enabled;

static GtkWidget *find_tree_view_with_columns (GtkWidget *widget,
						 gint n_columns);

static void
debug (const char *format, ...)
{
	va_list args;

	if (!debug_enabled)
		return;

	va_start (args, format);
	vfprintf (stderr, format, args);
	va_end (args);
	fflush (stderr);
}

static void
flush_events (int rounds)
{
	int i;

	for (i = 0; i < rounds; i++) {
		while (gtk_events_pending ())
			gtk_main_iteration ();
	}
}

static GtkWidget *
find_widget_by_name (GtkWidget *widget, const char *name)
{
	GList *children;
	GList *iter;
	GtkWidget *found;

	if (widget == NULL)
		return NULL;

	if (strcmp (gtk_widget_get_name (widget), name) == 0)
		return widget;

	if (!GTK_IS_CONTAINER (widget))
		return NULL;

	children = gtk_container_get_children (GTK_CONTAINER (widget));
	for (iter = children; iter; iter = iter->next) {
		found = find_widget_by_name (GTK_WIDGET (iter->data), name);
		if (found != NULL) {
			g_list_free (children);
			return found;
		}
	}
	g_list_free (children);

	return NULL;
}

static GtkWidget *
find_toplevel_by_type_name (const char *type_name)
{
	GList *toplevels;
	GList *iter;
	GtkWidget *window;

	toplevels = gtk_window_list_toplevels ();
	for (iter = toplevels; iter; iter = iter->next) {
		window = GTK_WIDGET (iter->data);
		if (strcmp (G_OBJECT_TYPE_NAME (window), type_name) == 0) {
			g_list_free (toplevels);
			return window;
		}
	}
	g_list_free (toplevels);

	return NULL;
}

static GtkWidget *
find_widget_in_toplevels_by_name (const char *name)
{
	GList *toplevels;
	GList *iter;
	GtkWidget *found;

	toplevels = gtk_window_list_toplevels ();
	for (iter = toplevels; iter; iter = iter->next) {
		found = find_widget_by_name (GTK_WIDGET (iter->data), name);
		if (found != NULL) {
			g_list_free (toplevels);
			return found;
		}
	}
	g_list_free (toplevels);

	return NULL;
}

static GtkWidget *
find_tree_view_in_toplevels_with_columns (gint n_columns)
{
	GList *toplevels;
	GList *iter;
	GtkWidget *found;

	toplevels = gtk_window_list_toplevels ();
	for (iter = toplevels; iter; iter = iter->next) {
		found = find_tree_view_with_columns (GTK_WIDGET (iter->data),
						     n_columns);
		if (found != NULL) {
			g_list_free (toplevels);
			return found;
		}
	}
	g_list_free (toplevels);

	return NULL;
}

static gint
tree_view_column_count (GtkTreeView *view)
{
	GList *columns;
	gint count;

	columns = gtk_tree_view_get_columns (view);
	count = g_list_length (columns);
	g_list_free (columns);

	return count;
}

static void
debug_dump_tree_views (GtkWidget *widget)
{
	GList *children;
	GList *iter;

	if (widget == NULL)
		return;

	if (GTK_IS_TREE_VIEW (widget))
		debug ("  treeview=%p columns=%d type=%s\n", (void *) widget,
		       tree_view_column_count (GTK_TREE_VIEW (widget)),
		       G_OBJECT_TYPE_NAME (widget));

	if (!GTK_IS_CONTAINER (widget))
		return;

	children = gtk_container_get_children (GTK_CONTAINER (widget));
	for (iter = children; iter; iter = iter->next)
		debug_dump_tree_views (GTK_WIDGET (iter->data));
	g_list_free (children);
}

static void
debug_dump_toplevels (void)
{
	GList *toplevels;
	GList *iter;
	GtkWidget *widget;
	const char *title;

	toplevels = gtk_window_list_toplevels ();
	for (iter = toplevels; iter; iter = iter->next) {
		widget = GTK_WIDGET (iter->data);
		title = GTK_IS_WINDOW (widget) ?
			gtk_window_get_title (GTK_WINDOW (widget)) : "";
		debug ("toplevel=%p type=%s title=%s\n", (void *) widget,
		       G_OBJECT_TYPE_NAME (widget), title ? title : "");
		debug_dump_tree_views (widget);
	}
	g_list_free (toplevels);
}

static GtkWidget *
find_tree_view_with_columns (GtkWidget *widget, gint n_columns)
{
	GList *children;
	GList *iter;
	GtkWidget *found;

	if (widget == NULL)
		return NULL;

	if (GTK_IS_TREE_VIEW (widget) &&
	    tree_view_column_count (GTK_TREE_VIEW (widget)) == n_columns)
		return widget;

	if (!GTK_IS_CONTAINER (widget))
		return NULL;

	children = gtk_container_get_children (GTK_CONTAINER (widget));
	for (iter = children; iter; iter = iter->next) {
		found = find_tree_view_with_columns (GTK_WIDGET (iter->data),
						     n_columns);
		if (found != NULL) {
			g_list_free (children);
			return found;
		}
	}
	g_list_free (children);

	return NULL;
}

static gboolean
iter_has_text (GtkTreeModel *model, GtkTreeIter *iter, const char *needle)
{
	int i;
	int n_columns;

	n_columns = gtk_tree_model_get_n_columns (model);
	for (i = 0; i < n_columns; i++) {
		GType type;
		gchar *value = NULL;

		type = gtk_tree_model_get_column_type (model, i);
		if (!g_type_is_a (type, G_TYPE_STRING))
			continue;

		gtk_tree_model_get (model, iter, i, &value, -1);
		if (value && strcmp (value, needle) == 0) {
			g_free (value);
			return TRUE;
		}
		g_free (value);
	}

	return FALSE;
}

static gboolean
find_iter_by_text (GtkTreeModel *model,
		      GtkTreeIter *parent,
		      GtkTreeIter *result,
		      const char *needle)
{
	GtkTreeIter iter;
	gboolean valid;

	if (parent != NULL)
		valid = gtk_tree_model_iter_children (model, &iter, parent);
	else
		valid = gtk_tree_model_get_iter_first (model, &iter);

	while (valid) {
		if (iter_has_text (model, &iter, needle)) {
			*result = iter;
			return TRUE;
		}
		if (find_iter_by_text (model, &iter, result, needle))
			return TRUE;
		valid = gtk_tree_model_iter_next (model, &iter);
	}

	return FALSE;
}

static void
select_iter (GtkTreeView *view, GtkTreeIter *iter)
{
	GtkTreePath *path;
	GtkTreeSelection *selection;

	path = gtk_tree_model_get_path (gtk_tree_view_get_model (view), iter);
	gtk_tree_view_expand_to_path (view, path);
	gtk_tree_view_scroll_to_cell (view, path, NULL, FALSE, 0.0, 0.0);

	selection = gtk_tree_view_get_selection (view);
	gtk_tree_selection_unselect_all (selection);
	gtk_tree_selection_select_iter (selection, iter);

	gtk_tree_path_free (path);
}

static void
dump_tree_view_rows (GtkTreeView *view, int *have_model, int *have_date)
{
	GtkTreeModel *model;
	GtkTreeIter iter;
	gboolean valid;

	model = gtk_tree_view_get_model (view);
	if (model == NULL)
		return;

	valid = gtk_tree_model_get_iter_first (model, &iter);
	while (valid) {
		gchar *tag = NULL;
		gchar *value = NULL;

		gtk_tree_model_get (model, &iter, 0, &tag, 1, &value, -1);
		if (tag && value) {
			g_print ("ROW|%s|%s\n", tag, value);
			if (strcmp (tag, "Model") == 0 &&
			    strcmp (value, "FinePix Z33WP") == 0)
				*have_model = 1;
			if (strcmp (tag, "DateTimeOriginal") == 0 &&
			    strcmp (value, "2009:03:25 03:27:25") == 0)
				*have_date = 1;
		}
		g_free (tag);
		g_free (value);
		valid = gtk_tree_model_iter_next (model, &iter);
	}
}

static void
scan_widget_for_exif_rows (GtkWidget *widget, int *have_model, int *have_date)
{
	GList *children;
	GList *iter;

	if (widget == NULL)
		return;

	if (GTK_IS_TREE_VIEW (widget))
		dump_tree_view_rows (GTK_TREE_VIEW (widget), have_model, have_date);

	if (!GTK_IS_CONTAINER (widget))
		return;

	children = gtk_container_get_children (GTK_CONTAINER (widget));
	for (iter = children; iter; iter = iter->next)
		scan_widget_for_exif_rows (GTK_WIDGET (iter->data),
					   have_model, have_date);
	g_list_free (children);
}

static gboolean
probe_cb (gpointer user_data)
{
	GtkWidget *chooser_widget;
	GtkWidget *add_camera_item;
	GtkWidget *tree_widget;
	GtkWidget *list_widget;
	GList *toplevels;
	GList *iter;
	GtkTreeModel *model;
	GtkTreeIter selected_iter;
	GtkamChooser *chooser;
	GtkamList *list;
	int have_model = 0;
	int have_date = 0;

	(void) user_data;

	switch (stage) {
	case 0:
		add_camera_item = find_widget_in_toplevels_by_name ("AddCamera");
		debug ("stage0 add=%p\n", (void *) add_camera_item);
		if (add_camera_item == NULL)
			break;
		gtk_menu_item_activate (GTK_MENU_ITEM (add_camera_item));
		stage = 1;
		ticks = 0;
		break;
	case 1:
		chooser_widget = find_toplevel_by_type_name ("GtkamChooser");
		debug ("stage1 chooser=%p\n", (void *) chooser_widget);
		if (chooser_widget == NULL)
			break;
		chooser = (GtkamChooser *) chooser_widget;
		if (chooser->priv == NULL || chooser->priv->ok == NULL)
			break;

		gtk_entry_set_text (chooser->priv->entry_model, camera_model);
		gtk_entry_set_text (chooser->priv->entry_port, camera_port);
		gtk_entry_set_text (chooser->priv->entry_speed, "Best");
		flush_events (10);
		gtk_button_clicked (GTK_BUTTON (chooser->priv->ok));
		stage = 2;
		ticks = 0;
		break;
	case 2:
		debug ("stage2 chooser_still_open=%p\n",
		       (void *) find_toplevel_by_type_name ("GtkamChooser"));
		tree_widget = find_tree_view_in_toplevels_with_columns (1);
		list_widget = find_tree_view_in_toplevels_with_columns (3);
		debug ("stage2 tree=%p list=%p\n", (void *) tree_widget,
		       (void *) list_widget);
		if (tree_widget == NULL || list_widget == NULL)
			debug_dump_toplevels ();
		if (tree_widget == NULL || list_widget == NULL)
			break;

		model = gtk_tree_view_get_model (GTK_TREE_VIEW (tree_widget));
		if (model == NULL ||
		    !find_iter_by_text (model, NULL, &selected_iter,
					"Directory Browse"))
			break;

		select_iter (GTK_TREE_VIEW (tree_widget), &selected_iter);
		flush_events (10);

		model = gtk_tree_view_get_model (GTK_TREE_VIEW (list_widget));
		if (model == NULL ||
		    !find_iter_by_text (model, NULL, &selected_iter, "fuji.jpg"))
			break;

		select_iter (GTK_TREE_VIEW (list_widget), &selected_iter);
		list = (GtkamList *) list_widget;
		list->priv->iter = selected_iter;
		saved_menu_item = gtk_item_factory_get_widget (list->priv->factory,
							      "/View EXIF data");
		debug ("saved_menu_item=%p\n", (void *) saved_menu_item);
		if (saved_menu_item == NULL)
			break;

		stage = 3;
		ticks = 0;
		break;
	case 3:
		debug ("stage3 activating=%p\n", (void *) saved_menu_item);
		if (saved_menu_item == NULL)
			break;
		gtk_menu_item_activate (GTK_MENU_ITEM (saved_menu_item));
		saved_menu_item = NULL;
		stage = 4;
		ticks = 0;
		break;
	case 4:
		flush_events (10);
		toplevels = gtk_window_list_toplevels ();
		for (iter = toplevels; iter; iter = iter->next)
			scan_widget_for_exif_rows (GTK_WIDGET (iter->data),
						   &have_model, &have_date);
		g_list_free (toplevels);
		debug ("stage4 have_model=%d have_date=%d\n", have_model,
		       have_date);
		fflush (stdout);

		if (have_model && have_date)
			_exit (0);
		break;
	}

	if (++ticks > 60) {
		g_printerr ("gtkam probe timed out in stage %d\n", stage);
		_exit (1);
	}

	return TRUE;
}

void
gtk_main (void)
{
	static void (*real_gtk_main) (void);

	if (real_gtk_main == NULL)
		real_gtk_main = dlsym (RTLD_NEXT, "gtk_main");

	camera_model = getenv ("LIBEXIF_GTKAM_CAMERA_MODEL");
	camera_port = getenv ("LIBEXIF_GTKAM_CAMERA_PORT");
	debug_enabled = getenv ("LIBEXIF_GTKAM_DEBUG") != NULL;
	if (camera_model == NULL || *camera_model == '\0' ||
	    camera_port == NULL || *camera_port == '\0') {
		g_printerr ("missing GTKam probe environment\n");
		_exit (2);
	}

	unsetenv ("LD_PRELOAD");
	g_timeout_add (250, probe_cb, NULL);
	real_gtk_main ();
}
EOF
}

run_eog_map_probe() {
  local image_path="$1"
  local expected_sensitive="$2"
  local log_dir="$3"

  mkdir -p "$log_dir"

  cat >"$log_dir/check-eog-map.py" <<'EOF'
from dogtail.tree import root
from dogtail.utils import doDelay


def find_jump_node(node):
    desc = getattr(node, "description", "") or ""
    if "Jump to current image" in desc and "location" in desc:
        return node
    try:
        children = node.children
    except Exception:
        return None
    for child in children:
        found = find_jump_node(child)
        if found is not None:
            return found
    return None


for _ in range(60):
    try:
        app = root.application("eog")
        break
    except Exception:
        doDelay(1)
else:
    raise SystemExit("eog not found")

for _ in range(8):
    doDelay(1)

node = find_jump_node(app)
if node is None:
    raise SystemExit("map jump control not found")

print(f"sensitive={getattr(node, 'sensitive')}")
EOF

  cat >"$log_dir/run-eog-map.sh" <<EOF
set -euo pipefail
export NO_AT_BRIDGE=0
export GTK_MODULES=gail:atk-bridge
gsettings set org.gnome.eog.plugins active-plugins "['map']"
eog "$image_path" >"$log_dir/eog.stdout" 2>"$log_dir/eog.stderr" &
app_pid=\$!
window_name="$(basename "$image_path")"
for _ in \$(seq 1 30); do
  wid="\$(xdotool search --onlyvisible --name "\$window_name" 2>/dev/null | head -n1 || true)"
  if [[ -n "\$wid" ]]; then
    break
  fi
  sleep 1
done
if [[ -z "\${wid:-}" ]]; then
  echo "failed to locate eog window for \$window_name" >&2
  exit 1
fi
xdotool key --window "\$wid" F9
sleep 3
python3 "$log_dir/check-eog-map.py" >"$log_dir/map-state.out"
gsettings get org.gnome.eog.plugins active-plugins >"$log_dir/active.txt"
kill "\$app_pid" || true
wait "\$app_pid" || true
EOF

  if ! timeout 30 xvfb-run -a dbus-run-session -- bash "$log_dir/run-eog-map.sh" >"$log_dir/dbus.log" 2>&1; then
    cat "$log_dir/dbus.log" >&2
    exit 1
  fi

  require_contains "$log_dir/active.txt" 'map'
  require_contains "$log_dir/map-state.out" "sensitive=$expected_sensitive"
}

run_eog_exif_display_probe() {
  local image_path="$1"
  local log_dir="$2"

  mkdir -p "$log_dir"

  cat >"$log_dir/check-eog-exif-display.py" <<'EOF'
from dogtail.tree import root
from dogtail.utils import doDelay

EXPECTED = [
    "libexif-gps",
    "Tue, 02 January 2024  03:04:05",
]


def collect_text(node, out):
    for attr in ("name", "description"):
        value = getattr(node, attr, None)
        if value:
            out.append(str(value))
    try:
        children = node.children
    except Exception:
        return
    for child in children:
        collect_text(child, out)


for _ in range(60):
    try:
        app = root.application("eog")
        break
    except Exception:
        doDelay(1)
else:
    raise SystemExit("eog not found")

for _ in range(8):
    doDelay(1)

texts = []
collect_text(app, texts)
for expected in EXPECTED:
    if not any(expected in text for text in texts):
        raise SystemExit(f"missing {expected!r}")

for text in texts:
    print(text)
EOF

  cat >"$log_dir/run-eog-exif-display.sh" <<EOF
set -euo pipefail
export NO_AT_BRIDGE=0
export GTK_MODULES=gail:atk-bridge
gsettings set org.gnome.eog.plugins active-plugins "['exif-display']"
eog "$image_path" >"$log_dir/eog.stdout" 2>"$log_dir/eog.stderr" &
app_pid=\$!
window_name="$(basename "$image_path")"
for _ in \$(seq 1 30); do
  wid="\$(xdotool search --onlyvisible --name "\$window_name" 2>/dev/null | head -n1 || true)"
  if [[ -n "\$wid" ]]; then
    break
  fi
  sleep 1
done
if [[ -z "\${wid:-}" ]]; then
  echo "failed to locate eog window for \$window_name" >&2
  exit 1
fi
xdotool key --window "\$wid" F9
sleep 3
python3 "$log_dir/check-eog-exif-display.py" >"$log_dir/exif-display.out"
gsettings get org.gnome.eog.plugins active-plugins >"$log_dir/active.txt"
kill "\$app_pid" || true
wait "\$app_pid" || true
EOF

  if ! timeout 40 xvfb-run -a dbus-run-session -- bash "$log_dir/run-eog-exif-display.sh" >"$log_dir/dbus.log" 2>&1; then
    cat "$log_dir/dbus.log" >&2
    exit 1
  fi

  require_contains "$log_dir/active.txt" 'exif-display'
  require_contains "$log_dir/exif-display.out" 'libexif-gps'
  require_contains "$log_dir/exif-display.out" 'Tue, 02 January 2024  03:04:05'
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

  assert_binary_uses_active_libexif "$EOG_PLUGIN_DIR/libexif-display.so"
  run_eog_exif_display_probe "$GPS_FIXTURE" "$dir/eog-exif-display"
}

test_eog_plugin_map() {
  local dir="$1"

  assert_binary_uses_active_libexif "$EOG_PLUGIN_DIR/libmap.so"
  exif -m "$GPS_FIXTURE" >"$dir/gps-exif.out"
  require_contains "$dir/gps-exif.out" $'North or South Latitude\tN'
  require_contains "$dir/gps-exif.out" $'Latitude\t33, 26, 27.36'
  require_contains "$dir/gps-exif.out" $'East or West Longitude\tW'
  require_contains "$dir/gps-exif.out" $'GPS Date\t2024:01:02'
  run_eog_map_probe "$GPS_FIXTURE" "True" "$dir/gps"
  run_eog_map_probe "$GENERATED_FIXTURE" "False" "$dir/no-gps"
}

test_tracker_extract() {
  local dir="$1"

  tracker3 extract "$GPS_FIXTURE" >"$dir/tracker.out" 2>"$dir/tracker.err"
  require_contains "$dir/tracker.out" 'slo:latitude "33.440933'
  require_contains "$dir/tracker.out" 'slo:longitude "-112.07399'
  require_contains "$dir/tracker.out" 'nfo:model "libexif-gps"'
  require_contains "$dir/tracker.out" 'nie:contentCreated "2024-01-02T03:04:05+0000"'
  require_contains "$dir/tracker.out" 'nfo:orientation nfo:orientation-top'
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
  local probe_src="$dir/foxtrotgps-runtime-probe.c"
  local probe_so="$dir/foxtrotgps-runtime-probe.so"
  local photo_dir="$dir/photos"
  local status=0

  assert_binary_uses_active_libexif /usr/bin/foxtrotgps
  mkdir -p "$photo_dir"
  cp "$GPS_FIXTURE" "$photo_dir/photo.jpg"
  build_foxtrotgps_runtime_probe "$probe_src"
  cc -shared -fPIC -Wall -Wextra -Wno-deprecated-declarations \
    -o "$probe_so" "$probe_src" \
    $(pkg-config --cflags --libs gtk+-2.0) -ldl
  timeout 40 dbus-run-session -- xvfb-run -a sh -c \
    'LIBEXIF_FOXTROTGPS_PHOTO_DIR="$1" LD_PRELOAD="$2" exec /usr/bin/foxtrotgps' \
    sh "$photo_dir" "$probe_so" >"$dir/foxtrotgps.out" 2>"$dir/foxtrotgps.err" || status=$?
  if [[ "${status:-0}" != 0 ]]; then
    cat "$dir/foxtrotgps.out" >&2 || true
    cat "$dir/foxtrotgps.err" >&2 || true
  fi
  assert_status_equals 0 "${status:-0}" "foxtrotgps"
  require_contains "$dir/foxtrotgps.out" 'CAMERA=2024:01:02 03:04:05'
  require_not_contains "$dir/foxtrotgps.out" 'symbol lookup error'
  require_not_contains "$dir/foxtrotgps.err" 'symbol lookup error'
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
  local probe_src="$dir/gtkam-runtime-probe.c"
  local probe_so="$dir/gtkam-runtime-probe.so"
  local status=0

  assert_binary_uses_active_libexif /usr/bin/gtkam
  build_gtkam_runtime_probe "$probe_src"
  cc -shared -fPIC -Wall -Wextra -Wno-deprecated-declarations \
    -o "$probe_so" "$probe_src" \
    $(pkg-config --cflags --libs gtk+-2.0) -ldl
  timeout 40 xvfb-run -a sh -c \
    'LIBEXIF_GTKAM_CAMERA_MODEL="$1" LIBEXIF_GTKAM_CAMERA_PORT="$2" LIBEXIF_GTKAM_DEBUG=1 LD_PRELOAD="$3" exec /usr/bin/gtkam' \
    sh 'Directory Browse' "Disk (disk:$GPHOTO_FIXTURE_DIR)" "$probe_so" \
    >"$dir/gtkam.out" 2>"$dir/gtkam.err" || status=$?
  if [[ "${status:-0}" != 0 ]]; then
    cat "$dir/gtkam.out" >&2 || true
    cat "$dir/gtkam.err" >&2 || true
  fi
  assert_status_equals 0 "${status:-0}" "gtkam"
  require_contains "$dir/gtkam.out" 'ROW|Model|FinePix Z33WP'
  require_contains "$dir/gtkam.out" 'ROW|DateTimeOriginal|2009:03:25 03:27:25'
}

test_minidlna() {
  local dir="$1"
  local media_dir="$dir/media"
  local db_dir="$dir/db"
  local log_dir="$dir/log"
  local status=0

  assert_binary_uses_active_libexif /usr/sbin/minidlnad

  mkdir -p "$media_dir" "$db_dir" "$log_dir"
  cp "$GPS_FIXTURE" "$media_dir/photo.jpg"
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
  sqlite3 "$db_dir/files.db" "select PATH || '|' || DATE || '|' || MIME from DETAILS where PATH like '%/photo.jpg';" >"$dir/minidlna-sqlite.out"
  require_contains "$dir/minidlna-sqlite.out" 'photo.jpg|2024-01-02T03:04:05|image/jpeg'
}

test_gerbera() {
  local dir="$1"
  local media_dir="$dir/media"
  local home_dir="$dir/home"
  local config_block="$dir/gerbera-import.xml"
  local status=0

  assert_binary_uses_active_libexif /usr/bin/gerbera

  mkdir -p "$home_dir" "$media_dir"
  cp "$GPS_FIXTURE" "$media_dir/photo.jpg"
  gerbera --create-config >"$dir/config.xml"
  cat >"$config_block" <<EOF
    <autoscan use-inotify="no">
      <directory location="$media_dir" mode="timed" interval="300" recursive="yes" hidden-files="no" media-type="Any" />
    </autoscan>
    <library-options>
      <libexif charset="UTF-8">
        <metadata>
          <add-data tag="EXIF_TAG_MODEL" key="exif:model" />
          <add-data tag="EXIF_TAG_DATE_TIME_ORIGINAL" key="exif:datetime-original" />
        </metadata>
        <auxdata>
          <add-data tag="EXIF_TAG_ORIENTATION" />
        </auxdata>
      </libexif>
    </library-options>
EOF
  insert_block_before_marker "$dir/config.xml" '</import>' "$config_block"

  timeout 20 gerbera --config "$dir/config.xml" --home "$home_dir" --offline >"$dir/gerbera.stdout" 2>"$dir/gerbera.stderr" || status=$?
  assert_status_equals 124 "${status:-0}" "gerbera"
  require_contains "$dir/gerbera.stdout" 'Configuration check succeeded.'
  require_contains "$dir/gerbera.stdout" 'database created successfully.'
  require_nonempty_file "$home_dir/gerbera.db"

  sqlite3 "$home_dir/gerbera.db" "select location || '|' || mime_type || '|' || auxdata from mt_cds_object where location like '%/photo.jpg';" >"$dir/gerbera-object.out"
  sqlite3 "$home_dir/gerbera.db" "select property_name || '=' || property_value from mt_metadata where item_id in (select id from mt_cds_object where location like '%/photo.jpg') order by property_name;" >"$dir/gerbera-metadata.out"

  require_contains "$dir/gerbera-object.out" 'photo.jpg|image/jpeg|EXIF_TAG_ORIENTATION=Top-left'
  require_contains "$dir/gerbera-metadata.out" 'dc:date=2024-01-02T03:04:05'
  require_contains "$dir/gerbera-metadata.out" 'exif:datetime-original=2024:01:02 03:04:05'
  require_contains "$dir/gerbera-metadata.out" 'exif:model=libexif-gps'
}

test_ruby_exif() {
  local dir="$1"

  assert_binary_uses_active_libexif "$RUBY_VENDORARCHDIR/exif.so"
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
  local srcdir
  local identify_bin

  apt-get update >"$dir/imagemagick-apt-update.log" 2>&1
  apt-get build-dep -y --no-install-recommends imagemagick >"$dir/imagemagick-builddep.log" 2>&1 || {
    cat "$dir/imagemagick-builddep.log" >&2
    exit 1
  }
  (
    cd "$dir"
    apt-get source imagemagick >"$dir/imagemagick-source.log" 2>&1
  ) || {
    cat "$dir/imagemagick-source.log" >&2
    exit 1
  }

  srcdir="$(find "$dir" -maxdepth 1 -mindepth 1 -type d -name 'imagemagick-*' | LC_ALL=C sort | head -n1)"
  [[ -n "$srcdir" ]] || die "failed to unpack imagemagick source package"

  if ! (
    cd "$srcdir"
    # Noble's debugedit can choke on newer DWARF emitted in transitive inputs.
    DEB_BUILD_OPTIONS='nocheck noautodbgsym nostrip' dpkg-buildpackage -us -uc -b >"$dir/imagemagick-build.log" 2>&1
  ); then
    cat "$dir/imagemagick-build.log" >&2
    exit 1
  fi

  find "$dir" -maxdepth 1 -type f -name '*.deb' | LC_ALL=C sort >"$dir/imagemagick-debs.out"
  require_contains "$dir/imagemagick-debs.out" 'imagemagick_'
  require_contains "$dir/imagemagick-debs.out" 'imagemagick-6.q16_'

  identify_bin="$(find "$srcdir" -type f -path '*/utilities/identify' | LC_ALL=C sort | head -n1)"
  [[ -x "$identify_bin" ]] || die "failed to locate freshly built ImageMagick identify binary"

  "$identify_bin" -verbose "$GENERATED_FIXTURE" >"$dir/imagemagick.out"
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
build_safe_packages
install_safe_packages
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
