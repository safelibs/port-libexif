#include <errno.h>
#include <inttypes.h>
#include <libexif/exif-content.h>
#include <libexif/exif-data.h>
#include <libexif/exif-entry.h>
#include <libexif/exif-mem.h>
#include <libexif/exif-mnote-data.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef enum {
    WORKLOAD_PARSE_FILE = 0,
    WORKLOAD_PARSE_MEMORY,
    WORKLOAD_SAVE_DATA,
    WORKLOAD_SWAP_BYTE_ORDER,
    WORKLOAD_FORMAT_ENTRIES,
    WORKLOAD_FORMAT_MAKERNOTES
} workload_t;

typedef struct {
    char *path;
    unsigned char *bytes;
    size_t size;
    ExifData *data;
} fixture_t;

typedef struct {
    uint64_t checksum;
    uint64_t count;
    char buffer[1024];
} format_context_t;

static volatile uint64_t g_sink = 0;

static void
fail(const char *message)
{
    fprintf(stderr, "bench-driver: %s\n", message);
    exit(1);
}

static void
fail_errno(const char *context, const char *path)
{
    fprintf(stderr, "bench-driver: %s %s: %s\n", context, path, strerror(errno));
    exit(1);
}

static char *
join_path(const char *root, const char *name)
{
    size_t root_len = strlen(root);
    size_t name_len = strlen(name);
    size_t need_sep = (root_len != 0 && root[root_len - 1] != '/') ? 1u : 0u;
    char *path = malloc(root_len + need_sep + name_len + 1);

    if (path == NULL) {
        fail("out of memory joining path");
    }

    memcpy(path, root, root_len);
    if (need_sep != 0) {
        path[root_len] = '/';
    }
    memcpy(path + root_len + need_sep, name, name_len + 1);
    return path;
}

static char *
trim_line(char *line)
{
    char *start = line;
    char *end;

    while (*start == ' ' || *start == '\t' || *start == '\n' || *start == '\r') {
        start++;
    }
    if (*start == '\0' || *start == '#') {
        return NULL;
    }

    end = start + strlen(start);
    while (end > start &&
           (end[-1] == ' ' || end[-1] == '\t' || end[-1] == '\n' || end[-1] == '\r')) {
        end--;
    }
    *end = '\0';
    return start;
}

static fixture_t *
load_manifest(const char *fixture_root, const char *manifest_path, size_t *fixture_count)
{
    FILE *manifest = fopen(manifest_path, "r");
    fixture_t *fixtures = NULL;
    size_t count = 0;
    size_t capacity = 0;
    char line[4096];

    if (manifest == NULL) {
        fail_errno("failed to open manifest", manifest_path);
    }

    while (fgets(line, sizeof(line), manifest) != NULL) {
        char *entry = trim_line(line);
        fixture_t *grown;

        if (entry == NULL) {
            continue;
        }

        if (count == capacity) {
            size_t new_capacity = capacity == 0 ? 8u : capacity * 2u;
            grown = realloc(fixtures, new_capacity * sizeof(*fixtures));
            if (grown == NULL) {
                fclose(manifest);
                fail("out of memory growing fixture list");
            }
            fixtures = grown;
            capacity = new_capacity;
        }

        fixtures[count].path = join_path(fixture_root, entry);
        fixtures[count].bytes = NULL;
        fixtures[count].size = 0;
        fixtures[count].data = NULL;
        count++;
    }

    if (ferror(manifest) != 0) {
        fclose(manifest);
        fail_errno("failed to read manifest", manifest_path);
    }

    fclose(manifest);

    if (count == 0) {
        fail("manifest did not contain any fixtures");
    }

    *fixture_count = count;
    return fixtures;
}

static void
load_fixture_bytes(fixture_t *fixture)
{
    FILE *file;
    long file_size;
    size_t read_size;

    file = fopen(fixture->path, "rb");
    if (file == NULL) {
        fail_errno("failed to open fixture", fixture->path);
    }
    if (fseek(file, 0, SEEK_END) != 0) {
        fclose(file);
        fail_errno("failed to seek fixture", fixture->path);
    }
    file_size = ftell(file);
    if (file_size < 0) {
        fclose(file);
        fail_errno("failed to stat fixture", fixture->path);
    }
    if (fseek(file, 0, SEEK_SET) != 0) {
        fclose(file);
        fail_errno("failed to rewind fixture", fixture->path);
    }

    fixture->bytes = malloc((size_t) file_size);
    if (fixture->bytes == NULL) {
        fclose(file);
        fail("out of memory loading fixture bytes");
    }
    fixture->size = (size_t) file_size;
    read_size = fread(fixture->bytes, 1, fixture->size, file);
    if (read_size != fixture->size) {
        fclose(file);
        fail_errno("failed to read fixture", fixture->path);
    }

    fclose(file);
}

static ExifData *
load_parsed_fixture(const char *path)
{
    ExifData *data = exif_data_new_from_file(path);

    if (data == NULL) {
        fprintf(stderr, "bench-driver: failed to parse fixture %s\n", path);
        exit(1);
    }

    return data;
}

static void
entry_format_callback(ExifEntry *entry, void *user_data)
{
    format_context_t *ctx = user_data;
    const char *value = exif_entry_get_value(entry, ctx->buffer, sizeof(ctx->buffer));

    if (value == NULL) {
        return;
    }

    ctx->checksum += (unsigned char) ctx->buffer[0];
    ctx->checksum += (uint64_t) strlen(ctx->buffer);
    ctx->count++;
}

static void
content_format_callback(ExifContent *content, void *user_data)
{
    exif_content_foreach_entry(content, entry_format_callback, user_data);
}

static uint64_t
format_entries_once(ExifData *data)
{
    format_context_t ctx;

    memset(&ctx, 0, sizeof(ctx));
    exif_data_foreach_content(data, content_format_callback, &ctx);
    return ctx.checksum + ctx.count;
}

static uint64_t
format_makernotes_once(ExifData *data)
{
    ExifMnoteData *note = exif_data_get_mnote_data(data);
    unsigned int count;
    unsigned int index;
    char buffer[1024];
    uint64_t checksum = 0;

    if (note == NULL) {
        fprintf(stderr, "bench-driver: fixture is missing MakerNote data\n");
        exit(1);
    }

    count = exif_mnote_data_count(note);
    if (count == 0) {
        fprintf(stderr, "bench-driver: fixture has empty MakerNote data\n");
        exit(1);
    }

    for (index = 0; index < count; index++) {
        char *value = exif_mnote_data_get_value(note, index, buffer, sizeof(buffer));

        if (value == NULL) {
            continue;
        }

        checksum += (unsigned char) buffer[0];
        checksum += (uint64_t) strlen(buffer);
    }

    return checksum + count;
}

static workload_t
parse_workload(const char *name)
{
    if (strcmp(name, "parse_file") == 0) {
        return WORKLOAD_PARSE_FILE;
    }
    if (strcmp(name, "parse_memory") == 0) {
        return WORKLOAD_PARSE_MEMORY;
    }
    if (strcmp(name, "save_data") == 0) {
        return WORKLOAD_SAVE_DATA;
    }
    if (strcmp(name, "swap_byte_order") == 0) {
        return WORKLOAD_SWAP_BYTE_ORDER;
    }
    if (strcmp(name, "format_entries") == 0) {
        return WORKLOAD_FORMAT_ENTRIES;
    }
    if (strcmp(name, "format_makernotes") == 0) {
        return WORKLOAD_FORMAT_MAKERNOTES;
    }

    fprintf(stderr, "bench-driver: unknown workload %s\n", name);
    exit(1);
}

static uint64_t
parse_file_benchmark(fixture_t *fixtures, size_t fixture_count, uint64_t iterations)
{
    uint64_t checksum = 0;
    uint64_t iteration;
    size_t index;

    for (iteration = 0; iteration < iterations; iteration++) {
        for (index = 0; index < fixture_count; index++) {
            ExifData *data = exif_data_new_from_file(fixtures[index].path);

            if (data == NULL) {
                fprintf(stderr, "bench-driver: parse_file failed for %s\n", fixtures[index].path);
                exit(1);
            }

            checksum += (uint64_t) data->size;
            exif_data_unref(data);
        }
    }

    return checksum;
}

static uint64_t
parse_memory_benchmark(fixture_t *fixtures, size_t fixture_count, uint64_t iterations)
{
    uint64_t checksum = 0;
    uint64_t iteration;
    size_t index;

    for (iteration = 0; iteration < iterations; iteration++) {
        for (index = 0; index < fixture_count; index++) {
            ExifData *data = exif_data_new_from_data(fixtures[index].bytes,
                                                     (unsigned int) fixtures[index].size);

            if (data == NULL) {
                fprintf(stderr, "bench-driver: parse_memory failed for %s\n", fixtures[index].path);
                exit(1);
            }

            checksum += (uint64_t) data->size;
            exif_data_unref(data);
        }
    }

    return checksum;
}

static uint64_t
save_data_benchmark(fixture_t *fixtures, size_t fixture_count, uint64_t iterations)
{
    ExifMem *mem = exif_mem_new_default();
    uint64_t checksum = 0;
    uint64_t iteration;
    size_t index;

    if (mem == NULL) {
        fail("failed to allocate ExifMem");
    }

    for (iteration = 0; iteration < iterations; iteration++) {
        for (index = 0; index < fixture_count; index++) {
            unsigned char *buffer = NULL;
            unsigned int size = 0;

            exif_data_save_data(fixtures[index].data, &buffer, &size);
            if (buffer == NULL || size == 0) {
                fprintf(stderr, "bench-driver: save_data failed for %s\n", fixtures[index].path);
                exif_mem_unref(mem);
                exit(1);
            }

            checksum += (uint64_t) size;
            checksum += buffer[0];
            exif_mem_free(mem, buffer);
        }
    }

    exif_mem_unref(mem);
    return checksum;
}

static uint64_t
swap_byte_order_benchmark(fixture_t *fixtures, size_t fixture_count, uint64_t iterations)
{
    uint64_t checksum = 0;
    uint64_t iteration;
    size_t index;

    for (iteration = 0; iteration < iterations; iteration++) {
        for (index = 0; index < fixture_count; index++) {
            ExifByteOrder original = exif_data_get_byte_order(fixtures[index].data);
            ExifByteOrder alternate = original == EXIF_BYTE_ORDER_INTEL
                                          ? EXIF_BYTE_ORDER_MOTOROLA
                                          : EXIF_BYTE_ORDER_INTEL;

            exif_data_set_byte_order(fixtures[index].data, alternate);
            exif_data_set_byte_order(fixtures[index].data, original);
            checksum += (uint64_t) exif_data_get_byte_order(fixtures[index].data);
        }
    }

    return checksum;
}

static uint64_t
format_entries_benchmark(fixture_t *fixtures, size_t fixture_count, uint64_t iterations)
{
    uint64_t checksum = 0;
    uint64_t iteration;
    size_t index;

    for (iteration = 0; iteration < iterations; iteration++) {
        for (index = 0; index < fixture_count; index++) {
            checksum += format_entries_once(fixtures[index].data);
        }
    }

    return checksum;
}

static uint64_t
format_makernotes_benchmark(fixture_t *fixtures, size_t fixture_count, uint64_t iterations)
{
    uint64_t checksum = 0;
    uint64_t iteration;
    size_t index;

    for (iteration = 0; iteration < iterations; iteration++) {
        for (index = 0; index < fixture_count; index++) {
            checksum += format_makernotes_once(fixtures[index].data);
        }
    }

    return checksum;
}

static void
prepare_fixtures(workload_t workload, fixture_t *fixtures, size_t fixture_count)
{
    size_t index;

    for (index = 0; index < fixture_count; index++) {
        switch (workload) {
        case WORKLOAD_PARSE_MEMORY:
            load_fixture_bytes(&fixtures[index]);
            break;
        case WORKLOAD_SAVE_DATA:
        case WORKLOAD_SWAP_BYTE_ORDER:
        case WORKLOAD_FORMAT_ENTRIES:
        case WORKLOAD_FORMAT_MAKERNOTES:
            fixtures[index].data = load_parsed_fixture(fixtures[index].path);
            break;
        case WORKLOAD_PARSE_FILE:
            break;
        }
    }
}

static void
free_fixtures(fixture_t *fixtures, size_t fixture_count)
{
    size_t index;

    for (index = 0; index < fixture_count; index++) {
        free(fixtures[index].path);
        free(fixtures[index].bytes);
        if (fixtures[index].data != NULL) {
            exif_data_unref(fixtures[index].data);
        }
    }

    free(fixtures);
}

int
main(int argc, char **argv)
{
    workload_t workload;
    fixture_t *fixtures;
    size_t fixture_count = 0;
    uint64_t iterations;
    uint64_t checksum = 0;
    char *end = NULL;

    if (argc != 5) {
        fprintf(stderr,
                "usage: %s <workload> <fixture-root> <manifest> <iterations>\n",
                argv[0]);
        return 1;
    }

    workload = parse_workload(argv[1]);
    iterations = strtoull(argv[4], &end, 10);
    if (argv[4][0] == '\0' || end == NULL || *end != '\0' || iterations == 0) {
        fail("iterations must be a positive integer");
    }

    fixtures = load_manifest(argv[2], argv[3], &fixture_count);
    prepare_fixtures(workload, fixtures, fixture_count);

    switch (workload) {
    case WORKLOAD_PARSE_FILE:
        checksum = parse_file_benchmark(fixtures, fixture_count, iterations);
        break;
    case WORKLOAD_PARSE_MEMORY:
        checksum = parse_memory_benchmark(fixtures, fixture_count, iterations);
        break;
    case WORKLOAD_SAVE_DATA:
        checksum = save_data_benchmark(fixtures, fixture_count, iterations);
        break;
    case WORKLOAD_SWAP_BYTE_ORDER:
        checksum = swap_byte_order_benchmark(fixtures, fixture_count, iterations);
        break;
    case WORKLOAD_FORMAT_ENTRIES:
        checksum = format_entries_benchmark(fixtures, fixture_count, iterations);
        break;
    case WORKLOAD_FORMAT_MAKERNOTES:
        checksum = format_makernotes_benchmark(fixtures, fixture_count, iterations);
        break;
    }

    g_sink += checksum;
    printf("workload=%s fixtures=%zu iterations=%" PRIu64 " checksum=%" PRIu64 "\n",
           argv[1],
           fixture_count,
           iterations,
           (uint64_t) g_sink);

    free_fixtures(fixtures, fixture_count);
    return 0;
}
