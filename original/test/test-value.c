/* test-value.c
 *
 * Creates all the types of tags supported in exif_entry_initialize() and
 * ensures that exif_entry_get_value() properly truncates the output of each
 * one according to the buffer size available.
 *
 * Copyright 2002 Lutz Mueller <lutz@users.sourceforge.net>
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
 */

#include <libexif/exif-data.h>
#include "test-public-api.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/*
 * List of tags to test, one per default initialized type.
 * There should be one for every block in exif_entry_initialize() and
 * exif_entry_get_value().
 */
ExifTag trunc_test_tags[] = {
	EXIF_TAG_PIXEL_X_DIMENSION,
	EXIF_TAG_SUBJECT_LOCATION,
	EXIF_TAG_IMAGE_WIDTH,
	EXIF_TAG_ORIENTATION,
	EXIF_TAG_SAMPLES_PER_PIXEL,
	EXIF_TAG_BITS_PER_SAMPLE,
	EXIF_TAG_X_RESOLUTION,
	EXIF_TAG_WHITE_POINT,
	EXIF_TAG_REFERENCE_BLACK_WHITE,
	EXIF_TAG_DATE_TIME,
	EXIF_TAG_IMAGE_DESCRIPTION,
	EXIF_TAG_EXIF_VERSION,
	EXIF_TAG_FLASH_PIX_VERSION,
	EXIF_TAG_COPYRIGHT,
	EXIF_TAG_FILE_SOURCE,
	EXIF_TAG_COMPONENTS_CONFIGURATION,
	EXIF_TAG_SCENE_TYPE,
	EXIF_TAG_YCBCR_SUB_SAMPLING,
	EXIF_TAG_PLANAR_CONFIGURATION,
};

/*
 * These tags produce different outputs depending on the amount of buffer space
 * available.
 */
ExifTag nonuniform_test_tags[] = {
	EXIF_TAG_RESOLUTION_UNIT,
	EXIF_TAG_COLOR_SPACE,
	EXIF_TAG_METERING_MODE,
};

/*
 * These tags need a nonzero rational number to be interesting.
 * They must have space for a rational or srational created automatically by
 * exif_entry_initialize().
 */
ExifTag rational_test_tags[] = {
	EXIF_TAG_FNUMBER,
	EXIF_TAG_APERTURE_VALUE,
	EXIF_TAG_MAX_APERTURE_VALUE,
	EXIF_TAG_FOCAL_LENGTH,
	EXIF_TAG_SUBJECT_DISTANCE,
	EXIF_TAG_EXPOSURE_TIME,
	EXIF_TAG_SHUTTER_SPEED_VALUE,
	EXIF_TAG_BRIGHTNESS_VALUE,
	EXIF_TAG_EXPOSURE_BIAS_VALUE,
};

/*
 * Verify that the entry is properly truncated to the buffer length within
 * exif_entry_get_value().  If uniform is zero, then only check that the
 * resulting string fits within the buffer and don't check its content.
 */
static void check_entry_trunc(ExifEntry *e, ExifTag tag, int uniform)
{
	unsigned int i;
	char v[1024], full[1024];  /* Large enough to never truncate output */

	printf ("Tag 0x%x\n", (int) tag);

	/* Get the full, untruncated string to use as the expected value */
	exif_entry_get_value (e, full, sizeof(full));
	printf ("Full: '%s'\n", full);

	for (i = strlen(full); i > 0; i--) {
		/* Make sure the buffer isn't NUL-terminated to begin with */
		memset(v, '*', sizeof(v));
		exif_entry_get_value (e, v, i);
		/* Truncate the full string by one on each iteration */
		full[i-1] = '\0';
		if ((strlen(v) >= i) || (uniform && strcmp(full, v))) {
			printf("Bad truncation!\n");
			printf("Length %2i: '%s'\n", i, v);
			exit(1);
		}
	}
}

int
main ()
{
	ExifData *data;
	ExifEntry *e;
	ExifContent *ifd0;
	unsigned i;

	data = exif_data_new ();
	if (!data) {
		fprintf (stderr, "Error running exif_data_new()\n");
		exit(13);
	}
	ifd0 = test_find_ifd_content(data, EXIF_IFD_0);
	if (!ifd0) {
		fprintf (stderr, "Error finding IFD 0\n");
		exit(13);
	}

	/* Full initialization/truncation tests */
	for (i=0; i < sizeof(trunc_test_tags)/sizeof(trunc_test_tags[0]); ++i) {
		e = exif_entry_new ();
		if (!e) {
			fprintf (stderr, "Error running exif_entry_new()\n");
			exit(13);
		}
		exif_content_add_entry (ifd0, e);
		exif_entry_initialize (e, trunc_test_tags[i]);
		check_entry_trunc(e, trunc_test_tags[i], 1);
		exif_content_remove_entry (ifd0, e);
		exif_entry_unref (e);
	}

	/* Nonuniform initialization/truncation tests */
	for (i=0; i < sizeof(nonuniform_test_tags)/sizeof(nonuniform_test_tags[0]);
		 ++i) {
		e = exif_entry_new ();
		if (!e) {
			fprintf (stderr, "Error running exif_entry_new()\n");
			exit(13);
		}
		exif_content_add_entry (ifd0, e);
		exif_entry_initialize (e, nonuniform_test_tags[i]);
		check_entry_trunc(e, nonuniform_test_tags[i], 0);
		exif_content_remove_entry (ifd0, e);
		exif_entry_unref (e);
	}

	/* Rational number initialization/truncation tests */
	for (i=0; i < sizeof(rational_test_tags)/sizeof(rational_test_tags[0]);
		 ++i) {
		e = exif_entry_new ();
		if (!e) {
			fprintf (stderr, "Error running exif_entry_new()\n");
			exit(13);
		}
		exif_content_add_entry (ifd0, e);
		exif_entry_initialize (e, rational_test_tags[i]);
		check_entry_trunc(e, rational_test_tags[i], 1);
		exif_content_remove_entry (ifd0, e);
		exif_entry_unref (e);
	}
	exif_data_unref (data);

	return 0;
}
