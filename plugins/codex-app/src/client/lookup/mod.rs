//! 제목 조회에 필요한 짧은 App Server 수명만 소유한다. Desktop socket에는 catalog 표면이 없다.

mod server;

use anyhow::Result;
use serde::Deserialize;
use serde_json::json;
use server::AppServer;
use std::path::Path;

/// 제목 선택에 필요한 App Server thread projection이다.
#[derive(Debug, Deserialize)]
pub struct ThreadSummary {
    pub id: String,
    pub name: Option<String>,
    pub preview: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ThreadList {
    data: Vec<ThreadSummary>,
}

/// 현재 cwd의 비아카이브 thread 목록을 읽고 App Server를 닫는다.
pub async fn threads(cwd: &Path) -> Result<Vec<ThreadSummary>> {
    let mut server = AppServer::connect(cwd).await?;

    let result = server
        .request::<ThreadList>(
            "thread/list",
            json!({
                "cursor":null, "limit":25, "sortKey":"recency_at", "sortDirection":"desc",
                "archived":false, "cwd":cwd
            }),
        )
        .await;

    server.close().await;
    Ok(result?.data)
}
