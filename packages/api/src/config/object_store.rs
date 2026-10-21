use object_store::Error;
use object_store::aws::{AmazonS3, AmazonS3Builder};

use super::settings;

#[allow(
    async_fn_in_trait,
    reason = "only called from this crate's binary, which needs no Send bound"
)]
pub trait ObjectStoreTrait {
    async fn init() -> Result<Self, Error>
    where
        Self: Sized;
    fn get_client(&self) -> &AmazonS3;
}

pub struct ObjectStore {
    client: AmazonS3,
}
impl ObjectStoreTrait for ObjectStore {
    async fn init() -> Result<Self, Error> {
        let endpoint = settings::object_storage_endpoint();
        let region = settings::object_storage_region();
        let access_key = settings::object_storage_access_key();
        let secret_key = settings::object_storage_secret_key();
        let bucket = settings::object_storage_bucket();

        let client = AmazonS3Builder::new()
            .with_endpoint(endpoint)
            .with_region(region)
            .with_access_key_id(access_key)
            .with_secret_access_key(secret_key)
            .with_bucket_name(bucket)
            .with_allow_http(true)
            .build()?;
        Ok(Self { client })
    }
    fn get_client(&self) -> &AmazonS3 {
        &self.client
    }
}
