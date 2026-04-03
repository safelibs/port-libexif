#ifndef LIBEXIF_TEST_PUBLIC_API_H
#define LIBEXIF_TEST_PUBLIC_API_H

#include <libexif/exif-data.h>
#include <libexif/exif-entry.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

typedef struct {
	ExifIfd ifd;
	ExifContent *content;
} TestIfdLookup;

typedef struct {
	unsigned int tag;
	unsigned int format;
	unsigned long components;
	unsigned int size;
	char tag_name[128];
	char format_name[128];
	char value[1024];
} TestEntryInfo;

static inline void
test_find_ifd_content_cb(ExifContent *content, void *user_data)
{
	TestIfdLookup *lookup = user_data;

	if (!lookup || lookup->content)
		return;
	if (exif_content_get_ifd(content) == lookup->ifd)
		lookup->content = content;
}

static inline ExifContent *
test_find_ifd_content(ExifData *data, ExifIfd ifd)
{
	TestIfdLookup lookup;

	lookup.ifd = ifd;
	lookup.content = NULL;
	exif_data_foreach_content(data, test_find_ifd_content_cb, &lookup);

	return lookup.content;
}

static inline ExifEntry *
test_find_entry_in_ifd(ExifData *data, ExifIfd ifd, ExifTag tag)
{
	ExifContent *content = test_find_ifd_content(data, ifd);

	if (!content)
		return NULL;

	return exif_content_get_entry(content, tag);
}

static inline int
test_entry_info_from_dump(ExifEntry *entry, TestEntryInfo *info)
{
	FILE *tmp = NULL;
	int saved_stdout = -1;
	char line[2048];
	int rc = -1;

	if (!entry || !info)
		return -1;

	memset(info, 0, sizeof(*info));

	saved_stdout = dup(fileno(stdout));
	if (saved_stdout < 0)
		goto done;

	tmp = tmpfile();
	if (!tmp)
		goto done;

	fflush(stdout);
	if (dup2(fileno(tmp), fileno(stdout)) < 0)
		goto done;

	exif_entry_dump(entry, 0);
	fflush(stdout);

	if (dup2(saved_stdout, fileno(stdout)) < 0)
		goto done;
	close(saved_stdout);
	saved_stdout = -1;

	rewind(tmp);
	while (fgets(line, sizeof(line), tmp)) {
		if (sscanf(line, "Tag: 0x%x ('%127[^']')", &info->tag,
			   info->tag_name) == 2)
			continue;
		if (sscanf(line, "  Format: %u ('%127[^']')", &info->format,
			   info->format_name) == 2)
			continue;
		if (sscanf(line, "  Components: %lu", &info->components) == 1)
			continue;
		if (sscanf(line, "  Size: %u", &info->size) == 1)
			continue;
		if (!strncmp(line, "  Value: ", 9)) {
			size_t len;

			strncpy(info->value, line + 9, sizeof(info->value) - 1);
			info->value[sizeof(info->value) - 1] = '\0';
			len = strlen(info->value);
			if (len && info->value[len - 1] == '\n')
				info->value[len - 1] = '\0';
		}
	}

	rc = 0;

done:
	if (saved_stdout >= 0) {
		fflush(stdout);
		(void) dup2(saved_stdout, fileno(stdout));
		close(saved_stdout);
	}
	if (tmp)
		fclose(tmp);

	return rc;
}

#endif
