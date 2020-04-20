#include "unity.h"
#include "libqiniu_ng.h"
#include <string.h>
#include <stdio.h>
#include "test.h"

#ifdef USE_NA_BUCKET
#define BUCKET_NAME (QINIU_NG_CHARS("na-bucket"))
#else
#define BUCKET_NAME (QINIU_NG_CHARS("z0-bucket"))
#endif

struct callback_context {
    int file_index;
    char *etag;
    int *completed;
};

static long long last_print_time;

#if defined(_WIN32) || defined(WIN32)
#include <windows.h>
static HANDLE mutex;
static void print_progress(uint64_t uploaded, uint64_t total, void* data) {
    struct callback_context *context = (struct callback_context *) data;
    switch (WaitForSingleObject(mutex, INFINITE)) {
    case WAIT_OBJECT_0:
        if (last_print_time + 5 < (long long) time(NULL)) {
            printf("%02d : %d: progress: %llu / %llu\n", context->file_index, GetCurrentThreadId(), uploaded, total);
            fflush(NULL);
            last_print_time = (long long) time(NULL);
        }
        ReleaseMutex(mutex);
        break;
    case WAIT_ABANDONED:
        break;
    }
}
#else
#include <unistd.h>
#include <pthread.h>
static pthread_mutex_t mutex;
static void print_progress(uint64_t uploaded, uint64_t total, void* data) {
    struct callback_context *context = (struct callback_context *) data;
    pthread_mutex_lock(&mutex);
    if (last_print_time + 5 < (long long) time(NULL)) {
        printf("%02d : %d: progress: %llu / %llu\n", context->file_index, (int) pthread_self(), uploaded, total);
        fflush(NULL);
        last_print_time = (long long) time(NULL);
    }
    pthread_mutex_unlock(&mutex);
}
#endif

static void on_completed(qiniu_ng_upload_response_t upload_response, qiniu_ng_err_t err, void *data) {
    if (qiniu_ng_err_any_error(&err)) {
        qiniu_ng_err_fputs(err, stderr);
        TEST_FAIL_MESSAGE("on_completed callback receives failure");
    }

    char hash[ETAG_SIZE + 1];
    size_t hash_size;
    memset(hash, 0, ETAG_SIZE + 1);
    struct callback_context *context = (struct callback_context *) data;
    qiniu_ng_str_t hashstr = qiniu_ng_upload_response_get_hash(upload_response);
    TEST_ASSERT_TRUE_MESSAGE(qiniu_ng_str_get_bytes(hashstr, ETAG_SIZE, &hash[0], &hash_size), "qiniu_ng_str_get_bytes() returns unexpected value");
    qiniu_ng_str_free(&hashstr);
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        hash_size, ETAG_SIZE,
        "hash_size != ETAG_SIZE");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        hash, (const char *) context->etag,
        "hash != etag");
    qiniu_ng_upload_response_free(&upload_response);

#if defined(_WIN32) || defined(WIN32)
    switch (WaitForSingleObject(mutex, INFINITE)) {
    case WAIT_OBJECT_0:
        (*context->completed)++;
        ReleaseMutex(mutex);
        break;
    case WAIT_ABANDONED:
        break;
    }
#else
    pthread_mutex_lock(&mutex);
    (*context->completed)++;
    pthread_mutex_unlock(&mutex);
#endif
}

static void generate_file_key(const qiniu_ng_char_t *file_key, int max_size, int file_id, int file_size) {
#if defined(_WIN32) || defined(WIN32)
    swprintf((wchar_t *) file_key, max_size, L"测试-%dm-%d-%lld-%d", file_size, file_id, (long long) time(NULL), rand());
#else
    snprintf((char *) file_key, max_size, "测试-%dm-%d-%lld-%d", file_size, file_id, (long long) time(NULL), rand());
#endif
}

static void prepare_for_uploading(void) {
#if defined(_WIN32) || defined(WIN32)
    mutex = CreateMutex(NULL, FALSE, NULL);
#else
    pthread_mutex_init(&mutex, NULL);
#endif
    last_print_time = (long long) time(NULL);
}

static void upload_done(void) {
    last_print_time = (long long) time(NULL);
#if defined(_WIN32) || defined(WIN32)
    ReleaseMutex(mutex);
#else
    pthread_mutex_destroy(&mutex);
#endif
}

void test_qiniu_ng_batch_upload_file_paths(void) {
#define FILES_COUNT (16)
    env_load("..", false);
    qiniu_ng_config_t config = qiniu_ng_config_new_default();
    qiniu_ng_upload_manager_t upload_manager = qiniu_ng_upload_manager_new(config);
    qiniu_ng_client_t client = qiniu_ng_client_new(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")), config);
    qiniu_ng_bucket_t bucket = qiniu_ng_bucket_new(client, BUCKET_NAME);

    qiniu_ng_upload_policy_builder_t policy_builder = qiniu_ng_upload_policy_builder_new_for_bucket(BUCKET_NAME, config);
    qiniu_ng_upload_policy_builder_set_insert_only(policy_builder);
    qiniu_ng_upload_token_t token = qiniu_ng_upload_token_new_from_policy_builder(policy_builder, GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));
    qiniu_ng_upload_policy_builder_free(&policy_builder);
    qiniu_ng_batch_uploader_t batch_uploader;
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_batch_uploader_new_for_upload_token(upload_manager, token, &batch_uploader, NULL),
        "qiniu_ng_batch_uploader_new_for_upload_token() returns unexpected value"
    );
    qiniu_ng_batch_uploader_set_expected_jobs_count(batch_uploader, FILES_COUNT);
    qiniu_ng_upload_token_free(&token);

    prepare_for_uploading();

    const qiniu_ng_char_t file_keys[FILES_COUNT][256];
    const qiniu_ng_char_t *file_paths[FILES_COUNT];
    struct callback_context contexts[FILES_COUNT];
    char etags[FILES_COUNT][ETAG_SIZE + 1];
    int completed = 0;
    for (int i = 0; i < FILES_COUNT; i++) {
        generate_file_key(file_keys[i], 256, i, 17);
        file_paths[i] = create_temp_file(17 * 1024 * 1024 + i * 1024);
        memset(&etags[i], 0, (ETAG_SIZE + 1) * sizeof(char));
        TEST_ASSERT_TRUE_MESSAGE(
            qiniu_ng_etag_from_file_path(file_paths[i], (char *) &etags[i][0], NULL),
            "qiniu_ng_etag_from_file_path() failed");

        contexts[i].file_index = i;
        contexts[i].etag = &etags[i][0];
        contexts[i].completed = &completed;

        qiniu_ng_batch_upload_params_t params = {
            .key = file_keys[i],
            .file_name = file_keys[i],
            .on_uploading_progress = print_progress,
            .on_completed = on_completed,
            .callback_data = (void *) &contexts[i],
        };
        TEST_ASSERT_TRUE_MESSAGE(
            qiniu_ng_batch_uploader_upload_file_path(batch_uploader, file_paths[i], &params, NULL),
            "qiniu_ng_batch_uploader_upload_file_path() failed");
    }

    qiniu_ng_batch_uploader_start(batch_uploader);
    TEST_ASSERT_EQUAL_INT_MESSAGE(completed, FILES_COUNT, "completed != FILES_COUNT");

    for (int i = 0; i < FILES_COUNT; i++) {
        qiniu_ng_object_t object = qiniu_ng_object_new(bucket, file_keys[i]);
        TEST_ASSERT_TRUE_MESSAGE(qiniu_ng_object_delete(object, NULL), "qiniu_ng_object_free() failed");
        qiniu_ng_object_free(&object);
        DELETE_FILE(file_paths[i]);
    }

    upload_done();
    qiniu_ng_batch_uploader_free(&batch_uploader);
    qiniu_ng_bucket_free(&bucket);
    qiniu_ng_client_free(&client);
    qiniu_ng_upload_manager_free(&upload_manager);
    qiniu_ng_config_free(&config);
#undef FILES_COUNT
}

void test_qiniu_ng_batch_upload_files(void) {
#define FILES_COUNT (16)
    env_load("..", false);
    qiniu_ng_config_t config = qiniu_ng_config_new_default();
    qiniu_ng_upload_manager_t upload_manager = qiniu_ng_upload_manager_new(config);
    qiniu_ng_client_t client = qiniu_ng_client_new(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")), config);
    qiniu_ng_bucket_t bucket = qiniu_ng_bucket_new(client, BUCKET_NAME);

    qiniu_ng_upload_policy_builder_t policy_builder = qiniu_ng_upload_policy_builder_new_for_bucket(BUCKET_NAME, config);
    qiniu_ng_upload_policy_builder_set_insert_only(policy_builder);
    qiniu_ng_upload_token_t token = qiniu_ng_upload_token_new_from_policy_builder(policy_builder, GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));
    qiniu_ng_upload_policy_builder_free(&policy_builder);
    qiniu_ng_batch_uploader_t batch_uploader;
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_batch_uploader_new_for_upload_token(upload_manager, token, &batch_uploader, NULL),
        "qiniu_ng_batch_uploader_new_for_upload_token() returns unexpected value"
    );
    qiniu_ng_batch_uploader_set_expected_jobs_count(batch_uploader, FILES_COUNT);
    qiniu_ng_upload_token_free(&token);

    prepare_for_uploading();

    const qiniu_ng_char_t file_keys[FILES_COUNT][256];
    const qiniu_ng_char_t *file_paths[FILES_COUNT];
    FILE *files[FILES_COUNT];
    struct callback_context contexts[FILES_COUNT];
    char etags[FILES_COUNT][ETAG_SIZE + 1];
    int completed = 0;
    for (int i = 0; i < FILES_COUNT; i++) {
        generate_file_key(file_keys[i], 256, i, 17);
        file_paths[i] = create_temp_file(17 * 1024 * 1024 + i * 1024);

        files[i] = OPEN_FILE_FOR_READING(file_paths[i]);
        TEST_ASSERT_NOT_NULL_MESSAGE(files[i], "files[i] == NULL");

        memset(&etags[i], 0, (ETAG_SIZE + 1) * sizeof(char));
        TEST_ASSERT_TRUE_MESSAGE(
            qiniu_ng_etag_from_file_path(file_paths[i], (char *) &etags[i][0], NULL),
            "qiniu_ng_etag_from_file_path() failed");

        contexts[i].file_index = i;
        contexts[i].etag = &etags[i][0];
        contexts[i].completed = &completed;

        qiniu_ng_batch_upload_params_t params = {
            .key = file_keys[i],
            .file_name = file_keys[i],
            .on_uploading_progress = print_progress,
            .on_completed = on_completed,
            .callback_data = (void *) &contexts[i],
        };
        TEST_ASSERT_TRUE_MESSAGE(
            qiniu_ng_batch_uploader_upload_file(batch_uploader, files[i], &params, NULL),
            "qiniu_ng_batch_uploader_upload_file() failed");
    }

    qiniu_ng_batch_uploader_start(batch_uploader);
    TEST_ASSERT_EQUAL_INT_MESSAGE(completed, FILES_COUNT, "completed != FILES_COUNT");

    for (int i = 0; i < FILES_COUNT; i++) {
        qiniu_ng_object_t object = qiniu_ng_object_new(bucket, file_keys[i]);
        TEST_ASSERT_TRUE_MESSAGE(qiniu_ng_object_delete(object, NULL), "qiniu_ng_object_free() failed");
        qiniu_ng_object_free(&object);

        fclose(files[i]);
        DELETE_FILE(file_paths[i]);
    }

    upload_done();
    qiniu_ng_batch_uploader_free(&batch_uploader);
    qiniu_ng_bucket_free(&bucket);
    qiniu_ng_client_free(&client);
    qiniu_ng_upload_manager_free(&upload_manager);
    qiniu_ng_config_free(&config);
#undef FILES_COUNT
}

void test_qiniu_ng_batch_upload_file_path_failed_by_mime(void) {
    env_load("..", false);
    qiniu_ng_upload_manager_t upload_manager = qiniu_ng_upload_manager_new_default();
    qiniu_ng_credential_t credential = qiniu_ng_credential_new(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));
    qiniu_ng_batch_uploader_t batch_uploader = qiniu_ng_batch_uploader_new(upload_manager, BUCKET_NAME, credential);

    qiniu_ng_char_t *file_path = create_temp_file(0);
    qiniu_ng_batch_upload_params_t params = {
        .mime = "invalid",
    };
    qiniu_ng_err_t err;
    qiniu_ng_str_t error;
    TEST_ASSERT_FALSE_MESSAGE(
        qiniu_ng_batch_uploader_upload_file_path(batch_uploader, file_path, &params, &err),
        "qiniu_ng_batch_uploader_upload_file_path() failed");
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_err_bad_mime_type_error_extract(&err, &error),
        "qiniu_ng_err_bad_mime_type_error_extract() failed");
    qiniu_ng_str_free(&error);
    TEST_ASSERT_FALSE_MESSAGE(
        qiniu_ng_err_bad_mime_type_error_extract(&err, &error),
        "qiniu_ng_err_bad_mime_type_error_extract() returns unexpected value");

    DELETE_FILE(file_path);
    free((void *) file_path);

    qiniu_ng_batch_uploader_free(&batch_uploader);
    qiniu_ng_credential_free(&credential);
    qiniu_ng_upload_manager_free(&upload_manager);
}

void test_qiniu_ng_batch_upload_file_path_failed_by_non_existed_path(void) {
    env_load("..", false);
    qiniu_ng_upload_manager_t upload_manager = qiniu_ng_upload_manager_new_default();
    qiniu_ng_credential_t credential = qiniu_ng_credential_new(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));
    qiniu_ng_batch_uploader_t batch_uploader = qiniu_ng_batch_uploader_new(upload_manager, BUCKET_NAME, credential);

    qiniu_ng_err_t err;
    int32_t code;
    TEST_ASSERT_FALSE_MESSAGE(
        qiniu_ng_batch_uploader_upload_file_path(batch_uploader, QINIU_NG_CHARS("/不存在的文件"), NULL, &err),
        "qiniu_ng_batch_uploader_upload_file_path() failed");
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_err_os_error_extract(&err, &code),
        "qiniu_ng_err_os_error_extract() failed");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        strerror(code), "No such file or directory",
        "strerror(code) != \"No such file or directory\"");
    TEST_ASSERT_FALSE_MESSAGE(
        qiniu_ng_err_os_error_extract(&err, &code),
        "qiniu_ng_err_os_error_extract() returns unexpected value");

    qiniu_ng_batch_uploader_free(&batch_uploader);
    qiniu_ng_credential_free(&credential);
    qiniu_ng_upload_manager_free(&upload_manager);
}
