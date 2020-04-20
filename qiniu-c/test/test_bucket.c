#include "unity.h"
#include "libqiniu_ng.h"
#include <string.h>
#include "test.h"

void test_qiniu_ng_bucket_get_name(void) {
    env_load("..", false);
    qiniu_ng_client_t client = qiniu_ng_client_new_default(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));

    qiniu_ng_bucket_t bucket = qiniu_ng_bucket_new(client, QINIU_NG_CHARS("z0-bucket"));
    qiniu_ng_str_t bucket_name = qiniu_ng_bucket_get_name(bucket);
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        qiniu_ng_str_get_cstr(bucket_name), QINIU_NG_CHARS("z0-bucket"),
        "qiniu_ng_str_get_cstr(bucket_name) != \"z0-bucket\"");
    qiniu_ng_str_free(&bucket_name);
    qiniu_ng_bucket_free(&bucket);

    qiniu_ng_bucket_t bucket_2 = qiniu_ng_bucket_new(client, QINIU_NG_CHARS("z1-bucket"));
    qiniu_ng_str_t bucket_name_2 = qiniu_ng_bucket_get_name(bucket_2);
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        qiniu_ng_str_get_cstr(bucket_name_2), QINIU_NG_CHARS("z1-bucket"),
        "qiniu_ng_str_get_cstr(bucket_name_2) != \"z1-bucket\"");
    qiniu_ng_str_free(&bucket_name_2);
    qiniu_ng_bucket_free(&bucket_2);

    qiniu_ng_client_free(&client);
}

void test_qiniu_ng_bucket_get_region(void) {
    env_load("..", false);
    qiniu_ng_client_t client = qiniu_ng_client_new_default(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));
    qiniu_ng_bucket_t bucket = qiniu_ng_bucket_new(client, QINIU_NG_CHARS("z0-bucket"));

    qiniu_ng_region_t region;
    const qiniu_ng_char_t *io_url;

    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_bucket_get_region(bucket, &region, NULL),
        "qiniu_ng_bucket_get_region() failed");
    qiniu_ng_str_list_t io_urls = qiniu_ng_region_get_io_urls(region, false);
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        qiniu_ng_str_list_len(io_urls), 1,
        "qiniu_ng_str_list_len(io_urls) != 1");
    io_url = qiniu_ng_str_list_get(io_urls, 0);
    TEST_ASSERT_NOT_NULL_MESSAGE(
        io_url,
        "io_url == null");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        io_url, QINIU_NG_CHARS("http://iovip.qbox.me"),
        "io_url != \"http://iovip.qbox.me\"");

    qiniu_ng_str_list_free(&io_urls);
    qiniu_ng_region_free(&region);
    qiniu_ng_bucket_free(&bucket);
    qiniu_ng_client_free(&client);
}

void test_qiniu_ng_bucket_get_unexisted_region(void) {
    env_load("..", false);
    qiniu_ng_client_t client = qiniu_ng_client_new_default(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));
    qiniu_ng_bucket_t bucket = qiniu_ng_bucket_new(client, QINIU_NG_CHARS("not-existed-bucket"));

    qiniu_ng_err_t err;
    unsigned short code;
    qiniu_ng_str_t error_message;

    TEST_ASSERT_FALSE_MESSAGE(
        qiniu_ng_bucket_get_region(bucket, NULL, &err),
        "qiniu_ng_bucket_get_region() returns unexpected value");
    TEST_ASSERT_FALSE_MESSAGE(
        qiniu_ng_err_os_error_extract(&err, NULL),
        "qiniu_ng_err_os_error_extract() returns unexpected value");
    TEST_ASSERT_FALSE_MESSAGE(
        qiniu_ng_err_io_error_extract(&err, NULL),
        "qiniu_ng_err_io_error_extract() returns unexpected value");
    TEST_ASSERT_FALSE_MESSAGE(
        qiniu_ng_err_json_error_extract(&err, NULL),
        "qiniu_ng_err_json_error_extract() returns unexpected value");
    TEST_ASSERT_FALSE_MESSAGE(
        qiniu_ng_err_unknown_error_extract(&err, NULL),
        "qiniu_ng_err_unknown_error_extract() returns unexpected value");
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_err_response_status_code_error_extract(&err, &code, &error_message),
        "qiniu_ng_err_response_status_code_error_extract() failed");
    TEST_ASSERT_EQUAL_UINT_MESSAGE(
        code, 631,
        "code != 631");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        qiniu_ng_str_get_cstr(error_message), QINIU_NG_CHARS("no such bucket"),
        "qiniu_ng_str_get_cstr(error_message) != \"no such bucket\"");
    TEST_ASSERT_FALSE_MESSAGE(
        qiniu_ng_err_response_status_code_error_extract(&err, NULL, NULL),
        "qiniu_ng_err_response_status_code_error_extract returns unexpected value");

    qiniu_ng_str_free(&error_message);
    qiniu_ng_bucket_free(&bucket);
    qiniu_ng_client_free(&client);
}

void test_qiniu_ng_bucket_get_regions(void) {
    env_load("..", false);
    qiniu_ng_client_t client = qiniu_ng_client_new_default(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));
    qiniu_ng_bucket_t bucket = qiniu_ng_bucket_new(client, QINIU_NG_CHARS("z0-bucket"));

    qiniu_ng_regions_t regions;
    qiniu_ng_region_t region;
    qiniu_ng_str_list_t io_urls;
    const qiniu_ng_char_t *io_url;

    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_bucket_get_regions(bucket, &regions, NULL),
        "qiniu_ng_bucket_get_regions() failed");
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        qiniu_ng_regions_len(regions), 2,
        "qiniu_ng_regions_len(regions) != 2");

    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_regions_get(regions, 0, &region),
        "qiniu_ng_regions_get(regions, 0, &region) failed");
    io_urls = qiniu_ng_region_get_io_urls(region, true);
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        qiniu_ng_str_list_len(io_urls), 1,
        "qiniu_ng_str_list_len(io_urls) != 1");
    io_url = qiniu_ng_str_list_get(io_urls, 0);
    TEST_ASSERT_NOT_NULL_MESSAGE(
        io_url,
        "io_url == null");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        io_url, QINIU_NG_CHARS("https://iovip.qbox.me"),
        "io_url != \"https://iovip.qbox.me\"");
    qiniu_ng_str_list_free(&io_urls);
    qiniu_ng_region_free(&region);

    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_regions_get(regions, 1, &region),
        "qiniu_ng_regions_get(regions, 1, &region) failed");
    io_urls = qiniu_ng_region_get_io_urls(region, true);
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        qiniu_ng_str_list_len(io_urls), 1,
        "qiniu_ng_str_list_len(io_urls) != 1");
    io_url = qiniu_ng_str_list_get(io_urls, 0);
    TEST_ASSERT_NOT_NULL_MESSAGE(
        io_url,
        "io_url == null");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        io_url, QINIU_NG_CHARS("https://iovip-z1.qbox.me"),
        "io_url != \"https://iovip-z1.qbox.me\"");
    qiniu_ng_str_list_free(&io_urls);
    qiniu_ng_region_free(&region);

    qiniu_ng_regions_free(&regions);
    qiniu_ng_bucket_free(&bucket);
    qiniu_ng_client_free(&client);
}

void test_qiniu_ng_bucket_builder(void) {
    env_load("..", false);
    qiniu_ng_client_t client = qiniu_ng_client_new_default(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));

    qiniu_ng_region_builder_t region_builder = qiniu_ng_region_builder_new();
    qiniu_ng_region_builder_set_region_id(region_builder, qiniu_ng_region_z0);
    qiniu_ng_region_t region_1 = qiniu_ng_region_build(region_builder);
    qiniu_ng_region_builder_reset(region_builder);
    qiniu_ng_region_builder_set_region_id(region_builder, qiniu_ng_region_z1);
    qiniu_ng_region_t region_2 = qiniu_ng_region_build(region_builder);
    qiniu_ng_region_builder_reset(region_builder);
    qiniu_ng_region_builder_set_region_id(region_builder, qiniu_ng_region_z2);
    qiniu_ng_region_t region_3 = qiniu_ng_region_build(region_builder);
    qiniu_ng_region_builder_free(&region_builder);

    qiniu_ng_bucket_builder_t bucket_builder = qiniu_ng_bucket_builder_new(client, QINIU_NG_CHARS("z2-bucket"));
    qiniu_ng_bucket_builder_set_region(bucket_builder, region_1);
    qiniu_ng_bucket_builder_set_region(bucket_builder, region_2);
    qiniu_ng_bucket_builder_set_region(bucket_builder, region_3);
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_bucket_builder_prepend_domain(bucket_builder, QINIU_NG_CHARS("domain2.example.com")),
        "qiniu_ng_bucket_builder_prepend_domain() returns unexpected value");
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_bucket_builder_prepend_domain(bucket_builder, QINIU_NG_CHARS("domain1.example.com")),
        "qiniu_ng_bucket_builder_prepend_domain() returns unexpected value");
    qiniu_ng_bucket_t bucket = qiniu_ng_bucket_build(bucket_builder);
    qiniu_ng_bucket_builder_free(&bucket_builder);

    qiniu_ng_regions_t regions;
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_bucket_get_regions(bucket, &regions, NULL),
        "qiniu_ng_bucket_get_regions() failed");
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        qiniu_ng_regions_len(regions), 3,
        "qiniu_ng_regions_len(regions) != 1");
    qiniu_ng_region_t region;
    qiniu_ng_region_id_t id;

    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_regions_get(regions, 0, &region),
        "qiniu_ng_regions_get(regions, 0, &region) failed");
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_region_get_region_id(region, &id),
        "qiniu_ng_region_get_region_id() failed");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        qiniu_ng_region_id_name(id), "z0",
        "qiniu_ng_region_id_name(id) != \"z0\"");
    qiniu_ng_region_free(&region);
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_regions_get(regions, 1, &region),
        "qiniu_ng_regions_get(regions, 1, &region) failed");
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_region_get_region_id(region, &id),
        "qiniu_ng_region_get_region_id() failed");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        qiniu_ng_region_id_name(id), "z1",
        "qiniu_ng_region_id_name(id) != \"z1\"");
    qiniu_ng_region_free(&region);
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_regions_get(regions, 2, &region),
        "qiniu_ng_regions_get(regions, 2, &region) failed");
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_region_get_region_id(region, &id),
        "qiniu_ng_region_get_region_id() failed");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        qiniu_ng_region_id_name(id), "z2",
        "qiniu_ng_region_id_name(id) != \"z2\"");
    qiniu_ng_region_free(&region);
    qiniu_ng_regions_free(&regions);


    qiniu_ng_str_list_t domains;
    const qiniu_ng_char_t *domain = NULL;
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_bucket_get_domains(bucket, &domains, NULL),
        "qiniu_ng_bucket_get_domains() failed");

    qiniu_ng_bucket_free(&bucket);
    qiniu_ng_region_free(&region_1);
    qiniu_ng_region_free(&region_2);
    qiniu_ng_region_free(&region_3);

    TEST_ASSERT_EQUAL_INT_MESSAGE(
        qiniu_ng_str_list_len(domains), 2,
        "qiniu_ng_str_list_len(domains) != 2");
    domain = qiniu_ng_str_list_get(domains, 0);
    TEST_ASSERT_NOT_NULL_MESSAGE(domain, "domain == null");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        domain, QINIU_NG_CHARS("domain1.example.com"),
        "domain != \"domain1.example.com\"");
    domain = qiniu_ng_str_list_get(domains, 1);
    TEST_ASSERT_NOT_NULL_MESSAGE(domain, "domain == null");
    TEST_ASSERT_EQUAL_STRING_MESSAGE(
        domain, QINIU_NG_CHARS("domain2.example.com"),
        "domain != \"domain2.example.com\"");
    qiniu_ng_str_list_free(&domains);

    qiniu_ng_client_free(&client);
}

void test_qiniu_ng_bucket_get_regions_and_domains(void) {
    env_load("..", false);
    qiniu_ng_client_t client = qiniu_ng_client_new_default(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));

    qiniu_ng_bucket_t bucket = qiniu_ng_bucket_new(client, QINIU_NG_CHARS("z0-bucket"));

    qiniu_ng_regions_t regions;
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_bucket_get_regions(bucket, &regions, NULL),
        "qiniu_ng_bucket_get_regions() failed");
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        qiniu_ng_regions_len(regions), 2,
        "qiniu_ng_regions_len(regions) != 1");
    qiniu_ng_regions_free(&regions);

    qiniu_ng_str_list_t domains;
    const qiniu_ng_char_t *domain = NULL;
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_bucket_get_domains(bucket, &domains, NULL),
        "qiniu_ng_bucket_get_domains() failed");
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        qiniu_ng_str_list_len(domains), 2,
        "qiniu_ng_str_list_len(domains) != 2");
    domain = qiniu_ng_str_list_get(domains, 0);
    TEST_ASSERT_NOT_NULL_MESSAGE(
        domain,
        "domain == null");
    domain = qiniu_ng_str_list_get(domains, 1);
    TEST_ASSERT_NOT_NULL_MESSAGE(
        domain,
        "domain == null");
    qiniu_ng_str_list_free(&domains);

    qiniu_ng_bucket_free(&bucket);
    qiniu_ng_client_free(&client);
}

void test_qiniu_ng_bucket_upload_files(void) {
    env_load("..", false);

    const qiniu_ng_char_t *file_path = create_temp_file(1024);
    char etag[ETAG_SIZE + 1];
    memset(&etag, 0, (ETAG_SIZE + 1) * sizeof(char));
    TEST_ASSERT_TRUE_MESSAGE(
        qiniu_ng_etag_from_file_path(file_path, (char *) &etag[0], NULL),
        "qiniu_ng_etag_from_file_path() failed");

    qiniu_ng_client_t client = qiniu_ng_client_new_default(GETENV(QINIU_NG_CHARS("access_key")), GETENV(QINIU_NG_CHARS("secret_key")));
    qiniu_ng_bucket_t bucket = qiniu_ng_bucket_new(client, QINIU_NG_CHARS("z0-bucket"));
    qiniu_ng_upload_response_t upload_response;
    qiniu_ng_err_t err;
    if (!qiniu_ng_bucket_upload_file_path(bucket, file_path, NULL, &upload_response, &err)) {
        qiniu_ng_err_fputs(err, stderr);
        TEST_FAIL_MESSAGE("qiniu_ng_bucket_upload_file_path() failed");
    }

    qiniu_ng_str_t key = qiniu_ng_upload_response_get_key(upload_response);
    TEST_ASSERT_FALSE_MESSAGE(qiniu_ng_str_is_null(key), "qiniu_ng_str_is_null(key) != false");
    qiniu_ng_object_t object = qiniu_ng_object_new(bucket, qiniu_ng_str_get_cstr(key));
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
    qiniu_ng_object_free(&object);

    FILE *file = OPEN_FILE_FOR_READING(file_path);
    TEST_ASSERT_NOT_NULL_MESSAGE(file, "file == null");
    if (!qiniu_ng_bucket_upload_file(bucket, file, NULL, &upload_response, &err)) {
        qiniu_ng_err_fputs(err, stderr);
        TEST_FAIL_MESSAGE("qiniu_ng_bucket_upload_file() failed");
    }
    TEST_ASSERT_EQUAL_INT_MESSAGE(
        fclose(file), 0,
        "fclose(file) != 0");

    key = qiniu_ng_upload_response_get_key(upload_response);
    TEST_ASSERT_FALSE_MESSAGE(
        qiniu_ng_str_is_null(key),
        "qiniu_ng_str_is_null(key) != false");
    object = qiniu_ng_object_new(bucket, qiniu_ng_str_get_cstr(key));
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
