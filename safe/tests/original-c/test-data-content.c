/* test-data-content.c
 *
 * Exercise exported ExifData, ExifContent and ExifLoader APIs on their
 * positive paths.
 *
 * Copyright 2026
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
#include <libexif/exif-entry.h>
#include <libexif/exif-loader.h>
#include <libexif/exif-mem.h>
#include <libexif/exif-utils.h>
#include "test-public-api.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define TEST_UNKNOWN_TAG ((ExifTag) 0xc7a1)

#define CHECK(cond, msg) \
	do { \
		if (!(cond)) { \
			fprintf(stderr, "%s\n", msg); \
			exit(EXIT_FAILURE); \
		} \
	} while (0)

static void
count_entries_cb(ExifEntry *entry, void *user_data)
{
	unsigned int *count = user_data;

	(void) entry;
	(*count)++;
}

static unsigned int
count_entries(ExifContent *content)
{
	unsigned int count = 0;

	exif_content_foreach_entry(content, count_entries_cb, &count);

	return count;
}

static void
add_unknown_short_entry(ExifContent *content, ExifByteOrder order)
{
	ExifMem *mem;
	ExifEntry *entry;

	mem = exif_mem_new_default();
	CHECK(mem != NULL, "Out of memory");

	entry = exif_entry_new_mem(mem);
	CHECK(entry != NULL, "Out of memory");

	entry->tag = TEST_UNKNOWN_TAG;
	entry->format = EXIF_FORMAT_SHORT;
	entry->components = 1;
	entry->size = exif_format_get_size(entry->format);
	entry->data = exif_mem_alloc(mem, entry->size);
	CHECK(entry->data != NULL, "Out of memory");

	exif_set_short(entry->data, order, 42);
	exif_content_add_entry(content, entry);

	exif_entry_unref(entry);
	exif_mem_unref(mem);
}

static void
build_source_data(unsigned char **raw_data, unsigned int *raw_size)
{
	ExifData *data;
	ExifContent *ifd0;
	ExifEntry *entry;

	*raw_data = NULL;
	*raw_size = 0;

	data = exif_data_new();
	CHECK(data != NULL, "Error running exif_data_new()");

	exif_data_set_byte_order(data, EXIF_BYTE_ORDER_INTEL);

	ifd0 = test_find_ifd_content(data, EXIF_IFD_0);
	CHECK(ifd0 != NULL, "Error finding IFD 0");

	entry = exif_entry_new();
	CHECK(entry != NULL, "Error running exif_entry_new()");
	exif_content_add_entry(ifd0, entry);
	exif_entry_initialize(entry, EXIF_TAG_IMAGE_DESCRIPTION);
	exif_entry_unref(entry);

	add_unknown_short_entry(ifd0, exif_data_get_byte_order(data));

	exif_data_save_data(data, raw_data, raw_size);
	exif_data_unref(data);

	CHECK(*raw_data != NULL, "Error running exif_data_save_data()");
	CHECK(*raw_size > 0, "No EXIF data was saved");
}

static void
wrap_loader_data(const unsigned char *raw_data, unsigned int raw_size,
		 unsigned char **wrapped_data, unsigned int *wrapped_size)
{
	*wrapped_size = raw_size + 2;
	*wrapped_data = malloc(*wrapped_size);
	CHECK(*wrapped_data != NULL, "Out of memory");

	exif_set_short(*wrapped_data, EXIF_BYTE_ORDER_MOTOROLA, raw_size);
	memcpy(*wrapped_data + 2, raw_data, raw_size);
}

static void
test_data_content_fix(void)
{
	ExifData *data;
	ExifContent *ifd0;
	ExifEntry *entry;
	ExifEntry *x_resolution;
	ExifEntry *duplicate;
	unsigned int count_before;
	char value[64];

	data = exif_data_new();
	CHECK(data != NULL, "Error running exif_data_new()");

	CHECK(exif_data_get_data_type(data) == EXIF_DATA_TYPE_UNKNOWN,
	      "Unexpected default ExifData type");
	CHECK(!strcmp(exif_data_option_get_name(
				EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS),
		      "Ignore unknown tags"),
	      "Unexpected EXIF option name");
	CHECK(strstr(exif_data_option_get_description(
				EXIF_DATA_OPTION_FOLLOW_SPECIFICATION),
		     "follows the specification") != NULL,
	      "Unexpected EXIF option description");

	exif_data_set_data_type(data, EXIF_DATA_TYPE_COMPRESSED);
	CHECK(exif_data_get_data_type(data) == EXIF_DATA_TYPE_COMPRESSED,
	      "Failed to update ExifData type");

	ifd0 = test_find_ifd_content(data, EXIF_IFD_0);
	CHECK(ifd0 != NULL, "Error finding IFD 0");

	entry = exif_entry_new();
	CHECK(entry != NULL, "Error running exif_entry_new()");
	exif_content_add_entry(ifd0, entry);
	exif_entry_initialize(entry, EXIF_TAG_PLANAR_CONFIGURATION);
	CHECK(exif_content_get_entry(ifd0, EXIF_TAG_PLANAR_CONFIGURATION) == entry,
	      "Entry lookup returned the wrong tag");
	CHECK(exif_entry_get_ifd(entry) == EXIF_IFD_0,
	      "Entry was added to the wrong IFD");
	exif_entry_unref(entry);

	count_before = count_entries(ifd0);
	CHECK(count_before == 1, "Unexpected entry count before exif_content_fix()");

	exif_content_fix(ifd0);

	CHECK(exif_content_get_entry(ifd0, EXIF_TAG_PLANAR_CONFIGURATION) == NULL,
	      "IFD fix did not remove a tag that is not recorded");

	x_resolution = test_find_entry_in_ifd(data, EXIF_IFD_0, EXIF_TAG_X_RESOLUTION);
	CHECK(x_resolution != NULL, "IFD fix did not add XResolution");
	CHECK(exif_data_get_entry(data, EXIF_TAG_X_RESOLUTION) == x_resolution,
	      "ExifData lookup returned the wrong entry");
	CHECK(exif_entry_get_ifd(x_resolution) == EXIF_IFD_0,
	      "XResolution was added to the wrong IFD");
	CHECK(test_find_entry_in_ifd(data, EXIF_IFD_0, EXIF_TAG_Y_RESOLUTION) != NULL,
	      "IFD fix did not add YResolution");
	CHECK(test_find_entry_in_ifd(data, EXIF_IFD_0, EXIF_TAG_RESOLUTION_UNIT) != NULL,
	      "IFD fix did not add ResolutionUnit");
	CHECK(test_find_entry_in_ifd(data, EXIF_IFD_0,
				     EXIF_TAG_YCBCR_POSITIONING) != NULL,
	      "IFD fix did not add YCbCrPositioning for compressed data");
	CHECK(exif_content_get_value(ifd0, EXIF_TAG_X_RESOLUTION,
				     value, sizeof(value)) == value,
	      "ExifContent value lookup failed");
	CHECK(value[0] != '\0', "XResolution should have a display value");

	duplicate = exif_entry_new();
	CHECK(duplicate != NULL, "Error running exif_entry_new()");
	duplicate->tag = EXIF_TAG_X_RESOLUTION;
	count_before = count_entries(ifd0);
	exif_content_add_entry(ifd0, duplicate);
	CHECK(count_entries(ifd0) == count_before,
	      "Duplicate entry should not have been added");
	CHECK(duplicate->parent == NULL,
	      "Duplicate entry unexpectedly acquired a parent");
	exif_entry_unref(duplicate);

	exif_content_remove_entry(ifd0, x_resolution);
	CHECK(test_find_entry_in_ifd(data, EXIF_IFD_0, EXIF_TAG_X_RESOLUTION) == NULL,
	      "Failed to remove XResolution");

	exif_data_fix(data);
	CHECK(test_find_entry_in_ifd(data, EXIF_IFD_0, EXIF_TAG_X_RESOLUTION) != NULL,
	      "ExifData fix did not restore a mandatory tag");

	exif_data_unref(data);
}

static void
test_load_options_and_loader(void)
{
	unsigned char *raw_data;
	unsigned int raw_size;
	unsigned char *wrapped_data;
	unsigned int wrapped_size;
	ExifData *loaded;
	ExifData *fixed;
	ExifData *from_loader;
	ExifContent *ifd0;
	ExifLoader *loader;
	const unsigned char *loader_buf;
	unsigned int loader_size;

	raw_data = NULL;
	raw_size = 0;
	wrapped_data = NULL;
	wrapped_size = 0;
	loader_buf = NULL;
	loader_size = 0;

	build_source_data(&raw_data, &raw_size);

	loaded = exif_data_new();
	CHECK(loaded != NULL, "Error running exif_data_new()");
	exif_data_set_data_type(loaded, EXIF_DATA_TYPE_COMPRESSED);
	exif_data_unset_option(loaded, EXIF_DATA_OPTION_FOLLOW_SPECIFICATION);
	exif_data_unset_option(loaded, EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS);
	exif_data_load_data(loaded, raw_data, raw_size);

	ifd0 = test_find_ifd_content(loaded, EXIF_IFD_0);
	CHECK(ifd0 != NULL, "Error finding IFD 0 after loading");
	CHECK(exif_content_get_entry(ifd0, TEST_UNKNOWN_TAG) != NULL,
	      "Unknown tag was dropped even though ignore-unknown was disabled");
	CHECK(exif_content_get_entry(ifd0, EXIF_TAG_X_RESOLUTION) == NULL,
	      "Mandatory tag was added even though follow-spec was disabled");
	exif_data_unref(loaded);

	fixed = exif_data_new();
	CHECK(fixed != NULL, "Error running exif_data_new()");
	exif_data_set_data_type(fixed, EXIF_DATA_TYPE_COMPRESSED);
	exif_data_load_data(fixed, raw_data, raw_size);

	ifd0 = test_find_ifd_content(fixed, EXIF_IFD_0);
	CHECK(ifd0 != NULL, "Error finding IFD 0 after fixed load");
	CHECK(exif_content_get_entry(ifd0, TEST_UNKNOWN_TAG) == NULL,
	      "Unknown tag was not ignored by the default options");
	CHECK(exif_content_get_entry(ifd0, EXIF_TAG_X_RESOLUTION) != NULL,
	      "Mandatory tag was not added by the default options");
	CHECK(exif_content_get_entry(ifd0, EXIF_TAG_YCBCR_POSITIONING) != NULL,
	      "Compressed-data load did not add YCbCrPositioning");
	exif_data_unref(fixed);

	wrap_loader_data(raw_data, raw_size, &wrapped_data, &wrapped_size);

	loader = exif_loader_new();
	CHECK(loader != NULL, "Error running exif_loader_new()");
	CHECK(exif_loader_write(loader, wrapped_data, 1) == 1,
	      "Failed to prime the ExifLoader");
	exif_loader_write(loader, wrapped_data + 1, wrapped_size - 1);

	exif_loader_get_buf(loader, &loader_buf, &loader_size);
	CHECK(loader_buf != NULL, "ExifLoader did not retain EXIF data");
	CHECK(loader_size == raw_size, "ExifLoader returned the wrong EXIF size");
	CHECK(!memcmp(loader_buf, raw_data, raw_size),
	      "ExifLoader buffer contents differ from the source EXIF data");

	from_loader = exif_loader_get_data(loader);
	CHECK(from_loader != NULL, "ExifLoader did not produce ExifData");
	CHECK(test_find_entry_in_ifd(from_loader, EXIF_IFD_0,
				    EXIF_TAG_IMAGE_DESCRIPTION) != NULL,
	      "ExifLoader data did not include the saved ImageDescription tag");
	exif_data_unref(from_loader);

	exif_loader_reset(loader);
	exif_loader_get_buf(loader, &loader_buf, &loader_size);
	CHECK(loader_buf == NULL, "ExifLoader reset did not clear its buffer");
	CHECK(loader_size == 0, "ExifLoader reset did not clear its size");
	CHECK(exif_loader_get_data(loader) == NULL,
	      "ExifLoader returned data after reset");
	exif_loader_unref(loader);

	free(wrapped_data);
	free(raw_data);
}

int
main(void)
{
	test_data_content_fix();
	test_load_options_and_loader();

	return 0;
}
