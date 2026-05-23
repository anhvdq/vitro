use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

#[serde_as]
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ListQueryParams {
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub page: Option<u32>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub page_size: Option<u32>,
    pub sort: Option<String>,
}

impl ListQueryParams {
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn page_size(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 100)
    }
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct PaginatedResponseDTO<T: Serialize + utoipa::ToSchema> {
    pub data: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
}
