#include <libexif/exif-log.h>

#include <stdarg.h>
#include <stdlib.h>

struct _ExifLog {
	unsigned int ref_count;
	ExifLogFunc func;
	void *data;
	ExifMem *mem;
};

static const char *exif_log_title(ExifLogCode code)
{
	switch (code) {
	case EXIF_LOG_CODE_DEBUG:
		return "Debugging information";
	case EXIF_LOG_CODE_NO_MEMORY:
		return "Not enough memory";
	case EXIF_LOG_CODE_CORRUPT_DATA:
		return "Corrupt data";
	default:
		return NULL;
	}
}

static const char *exif_log_message(ExifLogCode code)
{
	switch (code) {
	case EXIF_LOG_CODE_DEBUG:
		return "Debugging information is available.";
	case EXIF_LOG_CODE_NO_MEMORY:
		return "The system cannot provide enough memory.";
	case EXIF_LOG_CODE_CORRUPT_DATA:
		return "The data provided does not follow the specification.";
	default:
		return NULL;
	}
}

const char *
exif_log_code_get_title(ExifLogCode code)
{
	return exif_log_title(code);
}

const char *
exif_log_code_get_message(ExifLogCode code)
{
	return exif_log_message(code);
}

ExifLog *
exif_log_new_mem(ExifMem *mem)
{
	ExifLog *log = (ExifLog *)calloc(1, sizeof(*log));

	if (!log) {
		return NULL;
	}

	log->ref_count = 1;
	log->mem = mem;
	return log;
}

ExifLog *
exif_log_new(void)
{
	return exif_log_new_mem(NULL);
}

void
exif_log_ref(ExifLog *log)
{
	if (!log) {
		return;
	}

	log->ref_count++;
}

void
exif_log_unref(ExifLog *log)
{
	if (!log) {
		return;
	}

	if (log->ref_count > 0) {
		log->ref_count--;
	}
	if (log->ref_count == 0) {
		exif_log_free(log);
	}
}

void
exif_log_free(ExifLog *log)
{
	free(log);
}

void
exif_log_set_func(ExifLog *log, ExifLogFunc func, void *data)
{
	if (!log) {
		return;
	}

	log->func = func;
	log->data = data;
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
