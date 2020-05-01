use std::{env, sync::Once};

static INIT: Once = Once::new();

#[derive(Debug, Clone)]
pub struct Env {
    access_key: String,
    secret_key: String,
    public_bucket: String,
    private_bucket: String,
    huadong_bucket: String,
    huabei_bucket: String,
    huanan_bucket: String,
    upload_bucket: String,
    dual_regions_bucket_huadong: String,
    dual_regions_bucket_huabei: String,
}

pub fn get() -> Env {
    INIT.call_once(|| {
        let _ = dotenv::dotenv();
    });
    Env {
        access_key: env::var("access_key").expect("access_key must be set"),
        secret_key: env::var("secret_key").expect("secret_key must be set"),
        public_bucket: env::var("public_bucket").expect("public_bucket must be set"),
        private_bucket: env::var("private_bucket").expect("private_bucket must be set"),
        huadong_bucket: env::var("huadong_bucket").expect("huadong_bucket must be set"),
        huabei_bucket: env::var("huabei_bucket").expect("huabei_bucket must be set"),
        huanan_bucket: env::var("huanan_bucket").expect("huanan_bucket must be set"),
        upload_bucket: env::var("upload_bucket").expect("upload_bucket must be set"),
        dual_regions_bucket_huadong: env::var("dual_regions_bucket_huadong")
            .expect("dual_regions_bucket_huadong must be set"),
        dual_regions_bucket_huabei: env::var("dual_regions_bucket_huabei")
            .expect("dual_regions_bucket_huabei must be set"),
    }
}

impl Env {
    pub fn access_key(&self) -> &str {
        self.access_key.as_str()
    }
    pub fn secret_key(&self) -> &str {
        self.secret_key.as_str()
    }
    pub fn public_bucket(&self) -> &str {
        self.public_bucket.as_str()
    }
    pub fn private_bucket(&self) -> &str {
        self.private_bucket.as_str()
    }
    pub fn huadong_bucket(&self) -> &str {
        self.huadong_bucket.as_str()
    }
    pub fn huabei_bucket(&self) -> &str {
        self.huabei_bucket.as_str()
    }
    pub fn huanan_bucket(&self) -> &str {
        self.huanan_bucket.as_str()
    }
    pub fn upload_bucket(&self) -> &str {
        self.upload_bucket.as_str()
    }
    pub fn dual_regions_bucket_huadong(&self) -> &str {
        self.dual_regions_bucket_huadong.as_str()
    }
    pub fn dual_regions_bucket_huabei(&self) -> &str {
        self.dual_regions_bucket_huabei.as_str()
    }
}
