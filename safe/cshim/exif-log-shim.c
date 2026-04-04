#include <libexif/exif-log.h>

#include <stdarg.h>

struct _ExifLog {
	unsigned int ref_count;
	ExifLogFunc func;
	void *data;
	ExifMem *mem;
};

void
exif_log_shim_anchor(void)
{
}

#ifdef NO_VERBOSE_TAG_STRINGS
#undef exif_log
#endif
void
exif_log(ExifLog *log, ExifLogCode code, const char *domain, const char *format, ...)
{
	va_list args;

	va_start(args, format);
	exif_logv(log, code, domain, format, args);
	va_end(args);
}

void
exif_logv(ExifLog *log, ExifLogCode code, const char *domain, const char *format, va_list args)
{
	if (!log || !log->func) {
		return;
	}

	log->func(log, code, domain, format, args, log->data);
}
