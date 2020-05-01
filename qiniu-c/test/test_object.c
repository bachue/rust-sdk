#include "unity.h"
#include "libqiniu_ng.h"
#include <string.h>
#include "test.h"
#include <curl/curl.h>

static void generate_file_key(const qiniu_ng_char_t *file_key, int max_size, int file_id, int file_size) {
#if defined(_WIN32) || defined(WIN32)
    swprintf((wchar_t *) file_key, max_size, L"测试-%dk-%d-%lld-%d", file_size, file_id, (long long) time(NULL), rand());
#else
    snprintf((char *) file_key, max_size, "测试-%dk-%d-%lld-%d", file_size, file_id, (long long) time(NULL), rand());
#endif
}

void test_qiniu_ng_object_upload_files(void) {
    const qiniu_ng_char_t file_key[256];
    generate_file_key(file_key, 256, 0, 1);

    const qiniu_ng_char_t *file_path = create_temp_file(1024);
    char etag[ETAG_SIZE + 1];
    memset(&etag, 0, (ETAG_SIZE + 1) * sizeof(char));
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_etag_from_file_path(file_path, (char *) &etag[0], NULL),
        "qiniu_ng_etag_from_file_path() failed");

    qiniu_ng_client_t client = qiniu_ng_client_new_default(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));
    qiniu_ng_bucket_t bucket = qiniu_ng_bucket_new(client, GETENV(QINIU_NG_CHARS("upload_bucket")));
    qiniu_ng_object_t object = qiniu_ng_object_new(bucket, file_key);
    qiniu_ng_upload_response_t upload_response;
    qiniu_ng_err_t err;
    if (!qiniu_ng_object_upload_file_path(object, file_path, NULL, &upload_response, &err)) {
        qiniu_ng_err_fputs(err, stderr);
        TEST_FAIL_MESSAGE("qiniu_ng_object_upload_file_path() failed");
    }

    qiniu_ng_str_t key = qiniu_ng_upload_response_get_key(upload_response);
    TEST_ASSERT_FALSE_MESSAGE(qiniu_ng_str_is_null(key), "qiniu_ng_str_is_null(key) != false");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(file_key, qiniu_ng_str_get_cstr(key), "object.key != key");
    qiniu_ng_str_free(&key);

    char hash[ETAG_SIZE + 1];
    size_t hash_size;
    memset(hash, 0, ETAG_SIZE + 1);
    qiniu_ng_str_t hashstr = qiniu_ng_upload_response_get_hash(upload_response);
    TEST_ASSERT_TRUE_MESSAGE(qiniu_ng_str_get_bytes(hashstr, ETAG_SIZE, &hash[0], &hash_size), "qiniu_ng_str_get_bytes() returns unexpected value");
    qiniu_ng_str_free(&hashstr);
    TEST_ASSERT_EQUAL_INT_MESSAGE(hash_size, ETAG_SIZE, "hash_size != ETAG_SIZE");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(hash, (const char *) &etag, "hash != etag");

    qiniu_ng_upload_response_free(&upload_response);
    TEST_ASSERT_TRUE_MESSAGE(qiniu_ng_object_delete(object, NULL), "qiniu_ng_object_delete() failed");

    FILE *file = OPEN_FILE_FOR_READING(file_path);
    TEST_ASSERT_NOT_NULL_MESSAGE(file, "file == null");
    if (!qiniu_ng_object_upload_file(object, file, NULL, &upload_response, &err)) {
        qiniu_ng_err_fputs(err, stderr);
        TEST_FAIL_MESSAGE("qiniu_ng_object_upload_file() failed");
    }
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        fclose(file), 0,
        "fclose(file) != 0");

    key = qiniu_ng_upload_response_get_key(upload_response);
    TEST_ASSERT_FALSE_MESSAGE(
        qiniu_ng_str_is_null(key),
        "qiniu_ng_str_is_null(key) != false");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(file_key, qiniu_ng_str_get_cstr(key), "object.key != key");
    qiniu_ng_str_free(&key);

    memset(hash, 0, ETAG_SIZE + 1);
    hashstr = qiniu_ng_upload_response_get_hash(upload_response);
    TEST_ASSERT_TRUE_MESSAGE(qiniu_ng_str_get_bytes(hashstr, ETAG_SIZE, &hash[0], &hash_size), "qiniu_ng_str_get_bytes() returns unexpected value");
    qiniu_ng_str_free(&hashstr);
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        hash_size, ETAG_SIZE,
        "hash_size != ETAG_SIZE");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        hash, (const char *) &etag,
        "hash != etag");

    qiniu_ng_upload_response_free(&upload_response);

    TEST_ASSERT_TRUE_MESSAGE(qiniu_ng_object_delete(object, NULL), "qiniu_ng_object_delete() failed");
    qiniu_ng_object_free(&object);

    qiniu_ng_bucket_free(&bucket);
    qiniu_ng_client_free(&client);
    DELETE_FILE(file_path);
    free((void *) file_path);
}

static size_t write_ignore_callback(char *ptr, size_t size, size_t nmemb, void *userdata) {
    (void)(ptr);
    (void)(size);
    (void)(nmemb);
    (void)(userdata);
    return size * nmemb;
}

static long curl_get_url(const qiniu_ng_char_t *url) {
    CURL *curl = curl_easy_init();
    long status_code;
    TEST_ASSERT_NOT_NULL_MESSAGE(curl, "curl_easy_init() returns NULL");
    TEST_ASSERT_EQUAL_INT_MESSAGE(curl_easy_setopt(curl, CURLOPT_URL, url), CURLE_OK, "curl_easy_setopt(CURLOPT_URL) != ok");
    TEST_ASSERT_EQUAL_INT_MESSAGE(curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_ignore_callback), CURLE_OK, "curl_easy_setopt(CURLOPT_WRITEFUNCTION) != ok");
    TEST_ASSERT_EQUAL_INT_MESSAGE(curl_easy_perform(curl), CURLE_OK, "curl_easy_perform() != OK");
    TEST_ASSERT_EQUAL_INT_MESSAGE(curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &status_code), CURLE_OK, "curl_easy_getinfo() != OK");
    curl_easy_cleanup(curl);
    return status_code;
}

void test_qiniu_ng_object_get_urls(void) {
    const qiniu_ng_char_t file_key[256];
    generate_file_key(file_key, 256, 0, 1);

    const qiniu_ng_char_t *file_path = create_temp_file(1024);
    char etag[ETAG_SIZE + 1];
    memset(&etag, 0, (ETAG_SIZE + 1) * sizeof(char));
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_etag_from_file_path(file_path, (char *) &etag[0], NULL),
        "qiniu_ng_etag_from_file_path() failed");

    qiniu_ng_client_t client = qiniu_ng_client_new_default(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));
    qiniu_ng_bucket_t bucket = qiniu_ng_bucket_new(client, GETENV(QINIU_NG_CHARS("upload_bucket")));
    qiniu_ng_object_t object = qiniu_ng_object_new(bucket, file_key);
    qiniu_ng_err_t err;
    if (!qiniu_ng_object_upload_file_path(object, file_path, NULL, NULL, &err)) {
        qiniu_ng_err_fputs(err, stderr);
        TEST_FAIL_MESSAGE("qiniu_ng_object_upload_file_path() failed");
    }

    qiniu_ng_bucket_t bucket_got = qiniu_ng_object_get_bucket(object);
    qiniu_ng_str_t bucket_name = qiniu_ng_bucket_get_name(bucket_got);
    TEST_ASSERT_EQUAL_STRING_MESSAGE(qiniu_ng_str_get_cstr(bucket_name), GETENV(QINIU_NG_CHARS("upload_bucket")), "bucket_name != upload_bucket");
    qiniu_ng_str_free(&bucket_name);
    qiniu_ng_bucket_free(&bucket_got);

    qiniu_ng_str_t object_key = qiniu_ng_object_get_key(object);
    TEST_ASSERT_EQUAL_STRING_MESSAGE(qiniu_ng_str_get_cstr(object_key), file_key, "file_key != object_key");
    qiniu_ng_str_free(&object_key);

    qiniu_ng_header_info_t header_info;
    TEST_ASSERT_TRUE_MESSAGE(qiniu_ng_object_head(object, &header_info, NULL), "qiniu_ng_object_head() returns unexpected value");
    qiniu_ng_str_t content_type = qiniu_ng_header_info_get_content_type(header_info);
    TEST_ASSERT_EQUAL_STRING_MESSAGE(qiniu_ng_str_get_cstr(content_type), QINIU_NG_CHARS("application/octet-stream"), "content_type != 'application/octet-stream'");
    qiniu_ng_str_free(&content_type);
    qiniu_ng_str_t size = qiniu_ng_header_info_get_size(header_info);
    TEST_ASSERT_EQUAL_STRING_MESSAGE(qiniu_ng_str_get_cstr(size), QINIU_NG_CHARS("1024"), "size != '1024'");
    qiniu_ng_str_free(&size);
    qiniu_ng_str_t etag_string = qiniu_ng_header_info_get_etag(header_info);
    char etag_to_verify[ETAG_SIZE + 3];
    memset(&etag_to_verify, 0, (ETAG_SIZE + 3) * sizeof(char));
    size_t etag_string_len;
    TEST_ASSERT_TRUE_MESSAGE(qiniu_ng_str_get_bytes(etag_string, ETAG_SIZE + 2, etag_to_verify, &etag_string_len), "qiniu_ng_str_get_bytes() returns unexpected value");
    qiniu_ng_str_free(&etag_string);
    TEST_ASSERT_EQUAL_UINT_MESSAGE(etag_string_len, ETAG_SIZE + 2, "etag_string_len != ETAG_SIZE + 2");
    TEST_ASSERT_EQUAL_INT_MESSAGE(strncmp(etag, &etag_to_verify[1], ETAG_SIZE), 0, "etag != etag_to_verify[1:(1+ETAG_SIZE)]");
    qiniu_ng_header_info_free(&header_info);

    qiniu_ng_str_t object_url;
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_object_get_url_with_lifetime(object, 3600, &object_url, NULL),
        "qiniu_ng_object_get_url_with_lifetime() returns unexpected value");
    TEST_ASSERT_EQUAL_INT_MESSAGE(curl_get_url(qiniu_ng_str_get_cstr(object_url)), 200, "curl_get_url() does not return 200");
    qiniu_ng_str_free(&object_url);

    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_object_get_public_url(object, &object_url, NULL),
        "qiniu_ng_object_get_public_url() returns unexpected value");
    TEST_ASSERT_EQUAL_INT_MESSAGE(curl_get_url(qiniu_ng_str_get_cstr(object_url)), 200, "curl_get_url() does not return 200");
    qiniu_ng_str_free(&object_url);

    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_object_get_private_url_with_lifetime(object, 3600, &object_url, NULL),
        "qiniu_ng_object_get_private_url_with_lifetime() returns unexpected value");
    TEST_ASSERT_EQUAL_INT_MESSAGE(curl_get_url(qiniu_ng_str_get_cstr(object_url)), 200, "curl_get_url() does not return 200");
    qiniu_ng_str_free(&object_url);

    TEST_ASSERT_TRUE_MESSAGE(qiniu_ng_object_delete(object, NULL), "qiniu_ng_object_delete() failed");
    qiniu_ng_object_free(&object);
    qiniu_ng_bucket_free(&bucket);
    qiniu_ng_client_free(&client);
    DELETE_FILE(file_path);
    free((void *) file_path);
}
