/** \file test-extract.c
 * \brief Extract EXIF data from a file and write it to another file.
 *
 * Copyright (C) 2019 Dan Fandrich <dan@coneharvesters.com>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the
 * Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA  02110-1301  USA.
 *
 */

#include "libexif/exif-data.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const unsigned char header[4] = {'\xff', '\xd8', '\xff', '\xe1'};

static int
file_exists(const char *path)
{
	FILE *f = fopen(path, "rb");

	if (!f)
		return 0;
	fclose(f);
	return 1;
}

static const char *
find_default_input(void)
{
	static char path[512];
	static const char *roots[] = {
		"../original/test/testdata",
		"../../original/test/testdata",
		"original/test/testdata",
		NULL
	};
	static const char *images[] = {
		"canon_makernote_variant_1.jpg",
		"fuji_makernote_variant_1.jpg",
		NULL
	};
	int i;
	int j;

	for (i = 0; roots[i]; ++i) {
		for (j = 0; images[j]; ++j) {
			snprintf(path, sizeof(path), "%s/%s", roots[i], images[j]);
			if (file_exists(path))
				return path;
		}
	}

	return NULL;
}

int
main(const int argc, const char *argv[])
{
	int first = 1;
	const char *fn = NULL;
	const char *outfn = "output.exif";
	ExifData *d;
	unsigned char *buf;
	unsigned int len;
	FILE *f;
	unsigned char lenbuf[2];

	if (argc > 1 && !strcmp(argv[1], "-o")) {
		outfn = argv[2];
		first += 2;
	}
	if (argc > first) {
		fn = argv[first];
		++first;
	} else {
		fn = find_default_input();
	}
	if (!fn) {
		fprintf(stderr, "Could not locate bundled JPEG test data.\n");
		return 1;
	}
	if (argc > first) {
		fprintf(stderr, "Too many arguments\n");
		return 1;
	}

	d = exif_data_new_from_file(fn);
	if (!d) {
		fprintf(stderr, "Could not load data from '%s'!\n", fn);
		return 1;
	}

	exif_data_save_data(d, &buf, &len);
	exif_data_unref(d);

	if (!buf) {
		fprintf(stderr, "Could not extract EXIF data!\n");
		return 2;
	}

	f = fopen(outfn, "wb");
	if (!f) {
		fprintf(stderr, "Could not open '%s' for writing!\n", outfn);
		free(buf);
		return 1;
	}
	if (fwrite(header, 1, sizeof(header), f) != sizeof(header)) {
		fprintf(stderr, "Could not write to '%s'!\n", outfn);
		free(buf);
		fclose(f);
		return 3;
	}
	exif_set_short(lenbuf, EXIF_BYTE_ORDER_MOTOROLA, len);
	if (fwrite(lenbuf, 1, 2, f) != 2) {
		fprintf(stderr, "Could not write to '%s'!\n", outfn);
		free(buf);
		fclose(f);
		return 3;
	}
	if (fwrite(buf, 1, len, f) != len) {
		fprintf(stderr, "Could not write to '%s'!\n", outfn);
		free(buf);
		fclose(f);
		return 3;
	}
	if (fclose(f) != 0) {
		fprintf(stderr, "Could not close '%s'!\n", outfn);
		free(buf);
		return 3;
	}
	free(buf);

	return 0;
}
