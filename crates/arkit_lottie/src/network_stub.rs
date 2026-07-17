use std::sync::Arc;

use crate::{LottieError, LottieNetworkSource, LottieResult};

#[derive(Clone)]
pub(crate) struct LottieSourceLoader;

impl LottieSourceLoader {
    pub(crate) fn new() -> LottieResult<Self> {
        Ok(Self)
    }

    pub(crate) async fn load(&self, _source: LottieNetworkSource) -> LottieResult<Arc<[u8]>> {
        Err(LottieError::network(
            "LottieSourceLoader::load",
            "network Lottie sources require the arkit_lottie `network` feature",
        ))
    }
}
