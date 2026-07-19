//! Scene protocol — re-exported from `forgum-platform` so engine-internal
//! code can keep using `crate::protocol::SceneConfig` without a dependency
//! cycle. The canonical definition lives in `forgum-platform::protocol`.

pub use forgum_platform::protocol::SceneConfig;
