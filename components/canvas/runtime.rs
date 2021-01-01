use std::future::Future;
use tokio::runtime;

lazy_static! {
    pub static ref RUNTIME: runtime::Runtime = runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .unwrap();
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    RUNTIME.block_on(future)
}
