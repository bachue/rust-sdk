#include "unity.h"
#include "libqiniu_ng.h"
#include "test.h"

void test_qiniu_ng_region_query(void) {
    qiniu_ng_config_t config;
    qiniu_ng_config_init(&config);

    env_load("..", false);
    qiniu_ng_regions_t regions;
    qiniu_ng_err err;
    TEST_ASSERT_TRUE(qiniu_ng_region_query("z0-bucket", getenv("access_key"), &config, &regions, &err));
    TEST_ASSERT_EQUAL_INT(qiniu_ng_regions_len(regions), 2);

    qiniu_ng_region_t region;
    qiniu_ng_string_list_t urls;
    TEST_ASSERT_TRUE(qiniu_ng_regions_get(regions, 0, &region));
    urls = qiniu_ng_region_get_up_urls(region, true);
    size_t urls_len = qiniu_ng_string_list_len(urls);
    TEST_ASSERT_TRUE(urls_len > 4);

    for (size_t i = 0; i < urls_len; i++) {
        const char *p;
        TEST_ASSERT_TRUE(qiniu_ng_string_list_get(urls, i, &p));
    }

    qiniu_ng_string_list_free(urls);
    qiniu_ng_region_free(region);

    TEST_ASSERT_TRUE(qiniu_ng_regions_get(regions, 1, &region));
    urls = qiniu_ng_region_get_io_urls(region, true);
    urls_len = qiniu_ng_string_list_len(urls);
    TEST_ASSERT_EQUAL_INT(urls_len, 1);
    for (size_t i = 0; i < urls_len; i++) {
        const char *p;
        TEST_ASSERT_TRUE(qiniu_ng_string_list_get(urls, i, &p));
    }
    qiniu_ng_region_free(region);

    qiniu_ng_regions_free(regions);
}
