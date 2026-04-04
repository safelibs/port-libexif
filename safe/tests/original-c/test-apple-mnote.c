/* test-apple-mnote.c
 *
 * Copyright 2026
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 */

#include <config.h>

#include <libexif/exif-data.h>
#include <libexif/exif-format.h>
#include <locale.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "test-public-api.h"

#define EXIF_TAG_EXIF_IFD_POINTER 0x8769
#define MAKER_NOTE_TIFF_OFFSET 44U
#define MAKER_NOTE_SIZE 56U

static void
fail(const char *message)
{
	fprintf(stderr, "%s\n", message);
	exit(1);
}

static void
expect_true(int condition, const char *message)
{
	if (!condition)
		fail(message);
}

static void
set_le16(unsigned char *dst, unsigned short value)
{
	dst[0] = (unsigned char) (value & 0xff);
	dst[1] = (unsigned char) ((value >> 8) & 0xff);
}

static void
set_le32(unsigned char *dst, unsigned int value)
{
	dst[0] = (unsigned char) (value & 0xff);
	dst[1] = (unsigned char) ((value >> 8) & 0xff);
	dst[2] = (unsigned char) ((value >> 16) & 0xff);
	dst[3] = (unsigned char) ((value >> 24) & 0xff);
}

static size_t
build_payload(unsigned char *payload, size_t payload_size)
{
	unsigned char *tiff;
	unsigned char *entry;
	unsigned char *note;

	expect_true(payload_size >= 106U, "payload buffer too small");
	memset(payload, 0, payload_size);

	memcpy(payload, "Exif\0\0", 6);
	tiff = payload + 6;
	memcpy(tiff, "II", 2);
	set_le16(tiff + 2, 0x2a);
	set_le32(tiff + 4, 8);

	set_le16(tiff + 8, 1);
	entry = tiff + 10;
	set_le16(entry + 0, EXIF_TAG_EXIF_IFD_POINTER);
	set_le16(entry + 2, EXIF_FORMAT_LONG);
	set_le32(entry + 4, 1);
	set_le32(entry + 8, 26);
	set_le32(tiff + 22, 0);

	set_le16(tiff + 26, 1);
	entry = tiff + 28;
	set_le16(entry + 0, EXIF_TAG_MAKER_NOTE);
	set_le16(entry + 2, EXIF_FORMAT_UNDEFINED);
	set_le32(entry + 4, MAKER_NOTE_SIZE);
	set_le32(entry + 8, MAKER_NOTE_TIFF_OFFSET);
	set_le32(tiff + 40, 0);

	note = tiff + MAKER_NOTE_TIFF_OFFSET;
	memcpy(note, "Apple iOS\0", 10);
	note[10] = 0;
	note[11] = 0;
	note[12] = 'I';
	note[13] = 'I';
	set_le16(note + 14, 3);

	set_le16(note + 16, 0x000a);
	set_le16(note + 18, EXIF_FORMAT_SLONG);
	set_le32(note + 20, 1);
	set_le32(note + 24, 1);

	set_le16(note + 28, 0x0003);
	set_le16(note + 30, EXIF_FORMAT_SHORT);
	set_le32(note + 32, 2);
	set_le16(note + 36, 3);
	set_le16(note + 38, 4);

	set_le16(note + 40, 0x0015);
	set_le16(note + 42, EXIF_FORMAT_ASCII);
	set_le32(note + 44, 4);
	memcpy(note + 48, "abc\0", 4);
	set_le32(note + 52, 0);

	return 106U;
}

static void
expect_string(const char *actual, const char *expected, const char *label)
{
	if (!actual || strcmp(actual, expected)) {
		fprintf(stderr, "%s mismatch: expected '%s', got '%s'\n",
			label, expected, actual ? actual : "(null)");
		exit(1);
	}
}

static void
check_raw_makernote(ExifData *data, const unsigned char *expected, unsigned int expected_size)
{
	ExifEntry *entry = test_find_entry_in_ifd(data, EXIF_IFD_EXIF, EXIF_TAG_MAKER_NOTE);

	expect_true(entry != NULL, "missing MakerNote entry");
	expect_true(entry->format == EXIF_FORMAT_UNDEFINED, "unexpected MakerNote format");
	expect_true(entry->size == expected_size, "unexpected MakerNote size");
	expect_true(entry->components == expected_size, "unexpected MakerNote component count");
	expect_true(entry->data != NULL, "missing MakerNote payload");
	expect_true(!memcmp(entry->data, expected, expected_size), "MakerNote payload changed");
}

static void
check_note(ExifMnoteData *note)
{
	char value[64];

	expect_true(note != NULL, "missing Apple MakerNote");
	expect_true(exif_mnote_data_count(note) == 3, "unexpected Apple MakerNote count");

	expect_true(exif_mnote_data_get_id(note, 0) == 0x000a, "unexpected HDR id");
	expect_string(exif_mnote_data_get_name(note, 0), "HDR", "HDR name");
	expect_string(exif_mnote_data_get_title(note, 0), "HDR Mode", "HDR title");
	expect_string(exif_mnote_data_get_description(note, 0), "", "HDR description");
	expect_true(exif_mnote_data_get_value(note, 0, value, sizeof(value)) == value,
		    "missing HDR value");
	expect_string(value, "1", "HDR value");

	expect_true(exif_mnote_data_get_id(note, 1) == 0x0003, "unexpected runtime id");
	expect_string(exif_mnote_data_get_name(note, 1), "RUNTIME", "Runtime name");
	expect_string(exif_mnote_data_get_title(note, 1), "Runtime", "Runtime title");
	expect_string(exif_mnote_data_get_description(note, 1), "", "Runtime description");
	expect_true(exif_mnote_data_get_value(note, 1, value, sizeof(value)) == value,
		    "missing runtime value");
	expect_string(value, "3 4 ", "Runtime value");

	expect_true(exif_mnote_data_get_id(note, 2) == 0x0015, "unexpected image unique id");
	expect_string(exif_mnote_data_get_name(note, 2), "IMAGE_UNIQUE_ID", "Image unique id name");
	expect_string(exif_mnote_data_get_title(note, 2), "Image Unique ID", "Image unique id title");
	expect_string(exif_mnote_data_get_description(note, 2), "", "Image unique id description");
	expect_true(exif_mnote_data_get_value(note, 2, value, sizeof(value)) == value,
		    "missing image unique id value");
	expect_string(value, "abc", "Image unique id value");
}

int
main(void)
{
	unsigned char payload[106];
	unsigned char *saved = NULL;
	unsigned int saved_size = 0;
	size_t payload_size;
	unsigned char *expected_note;
	ExifData *data;
	ExifMnoteData *note;

	expect_true(setlocale(LC_ALL, "C") != NULL, "failed to set locale");

	payload_size = build_payload(payload, sizeof(payload));
	expected_note = payload + 6 + MAKER_NOTE_TIFF_OFFSET;

	data = exif_data_new_from_data(payload, (unsigned int) payload_size);
	expect_true(data != NULL, "failed to parse Apple payload");
	note = exif_data_get_mnote_data(data);
	check_note(note);
	check_raw_makernote(data, expected_note, MAKER_NOTE_SIZE);

	exif_data_set_option(data, EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE);
	exif_data_save_data(data, &saved, &saved_size);
	expect_true(saved != NULL, "failed to save Apple payload");
	expect_true(saved_size > 0, "saved Apple payload is empty");
	exif_data_unref(data);

	data = exif_data_new_from_data(saved, saved_size);
	expect_true(data != NULL, "failed to reparse Apple payload");
	note = exif_data_get_mnote_data(data);
	check_note(note);
	check_raw_makernote(data, expected_note, MAKER_NOTE_SIZE);

	free(saved);
	exif_data_unref(data);
	return 0;
}
