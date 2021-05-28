// rpc
// 状態管理
// ソース
// Judge score
// 適切な構造で持つ

pub trait Source {}

pub trait Judge {}

pub trait Score: PartialEq + Clone {}

pub mod background {
    pub async fn run() -> anyhow::Result<()> { Ok(()) }
}
