//! Shared handler for the source examples in `src/bin`.

#[derive(Debug)]
pub struct Logger;

impl<V: std::fmt::Debug + Sync, R: Sync> shipstern::Handler<V, R> for Logger {
    async fn handle(&self, value: &V, _raw: &R) -> shipstern::HandlerResult<()> {
        println!("{value:?}");
        Ok(())
    }
}
