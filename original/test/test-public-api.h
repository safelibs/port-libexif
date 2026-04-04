#ifndef LIBEXIF_TEST_PUBLIC_API_H
#define LIBEXIF_TEST_PUBLIC_API_H

#include <libexif/exif-data.h>
#include <stddef.h>

typedef struct {
	ExifIfd ifd;
	ExifContent *content;
} TestIfdLookup;

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

#endif
