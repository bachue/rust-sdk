# 如何运行集成测试

## 1. 设置七牛账户

### 方法一

在当前目录中放置 `.env` 文件，内容如下：

```bash
access_key=[access_key]
secret_key=[secret_key]
public_bucket=[public_bucket]
private_bucket=[private_bucket]
huadong_bucket=[huadong_bucket]
huabei_bucket=[huabei_bucket]
huanan_bucket=[huanan_bucket]
upload_bucket=[upload_bucket]
dual_regions_bucket_huadong=[dual_regions_bucket_huadong]
dual_regions_bucket_huabei=[dual_regions_bucket_huabei]
```

### 方法二

设置环境变量 `access_key`，`secret_key`，`public_bucket`，`private_bucket`，`huadong_bucket`，`huabei_bucket`，`huanan_bucket`，`upload_bucket`，`dual_regions_bucket_huadong` 和 `dual_regions_bucket_huabei`。

## 2. 配置七牛账户

1. 按需要创建测试用存储空间

- 一个公开访问的存储空间，赋值给 `public_bucket`
- 一个私有访问的存储空间，赋值给 `private_bucket`
- 一个华东区存储空间，赋值给 `huadong_bucket`
- 一个华北区存储空间，赋值给 `huabei_bucket`
- 一个华南区存储空间，赋值给 `huanan_bucket`
- 一个上传专用存储空间，赋值给 `upload_bucket`
- 两个个双活区域的存储空间，华东区赋值给 `dual_regions_bucket_huadong`，华北区赋值给 `dual_regions_bucket_huabei`
