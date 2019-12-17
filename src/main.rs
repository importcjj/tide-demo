use async_std::io;
use async_std::task;

use mobc::{ConnectionManager, runtime::DefaultExecutor, Pool, AnyFuture};

struct FooManager;

struct FooConnection;

impl FooConnection {
    async fn query(&self) -> String {
        "nori".to_string()
    }
}

impl ConnectionManager for FooManager {
    type Connection = FooConnection;
    type Error = std::io::Error;
    type Executor = DefaultExecutor;

    fn get_executor(&self) -> Self::Executor {
        DefaultExecutor::current()
    }

    fn connect(&self) -> AnyFuture<Self::Connection, Self::Error> {
        Box::pin(futures::future::ok(FooConnection))
    }

    fn is_valid(&self, conn: Self::Connection) -> AnyFuture<Self::Connection, Self::Error> {
        Box::pin(futures::future::ok(conn))
    }

    fn has_broken(&self, conn: &mut Option<Self::Connection>) -> bool {
        false
    }
}

/// Shared application state.
#[derive(Debug)]
struct State {
    pool: Pool<FooManager>
}

fn main() -> io::Result<()> {
    task::block_on(async {
        let pool = Pool::new(FooManager).await.unwrap();
        let mut app = tide::with_state(State { pool });
        app.at("/submit").post(|req: tide::Request<State>| {
            async move {
                let conn = &req.state().pool.get().await.unwrap();
                let name = conn.query().await;
                tide::Response::new(200).body_string(name.to_string())
            }
        });
        app.listen("127.0.0.1:8080").await?;
        Ok(())
    })
}