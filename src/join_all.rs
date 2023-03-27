use std::future::Future;
use tokio::runtime::Handle;
use futures::stream::FuturesOrdered;
use futures::StreamExt;

#[allow(dead_code)]
/// Join all futures in a FuturesOrdered set and return a Vec of the results
pub fn join_all_to_vec<Fut>(set: FuturesOrdered<Fut>, runtime: &Handle) -> Vec<Fut::Output>
    where Fut: Future + Send + 'static,
{
    runtime.block_on(async {
        set.collect().await
    })
}