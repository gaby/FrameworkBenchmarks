use std::{borrow::Cow, convert::Infallible, io, sync::Arc};

use axum::{extract::FromRequestParts, http::request::Parts};
use futures::{stream::futures_unordered::FuturesUnordered, StreamExt, TryStreamExt};
use rand::{rngs::SmallRng, rng, SeedableRng};
use tokio::pin;
use tokio_postgres::{connect, Client, NoTls, Statement};

use crate::common::{self, random_id, random_ids};

use super::models::{Fortune, World};

#[derive(Debug)]
#[allow(dead_code)]
pub enum PgError {
    Io(io::Error),
    Pg(tokio_postgres::Error),
}

impl From<io::Error> for PgError {
    fn from(err: io::Error) -> Self {
        PgError::Io(err)
    }
}

impl From<tokio_postgres::Error> for PgError {
    fn from(err: tokio_postgres::Error) -> Self {
        PgError::Pg(err)
    }
}

/// Postgres interface
pub struct PgConnection {
    client: Client,
    fortune: Statement,
    world: Statement,
    updates: Statement,
}

impl PgConnection {
    pub async fn connect(db_url: String) -> Arc<PgConnection> {
        let (cl, conn) = connect(&db_url, NoTls)
            .await
            .expect("cannot connect to postgresql.");

        // Spawn connection
        tokio::spawn(async move {
            if let Err(error) = conn.await {
                eprintln!("database connection error: {error}");
            }
        });

        // Prepare statements for the connection.
        let fortune = cl.prepare(common::SELECT_ALL_FORTUNES).await.unwrap();
        let world = cl.prepare(common::SELECT_WORLD_BY_ID).await.unwrap();
        let updates = cl.prepare(common::UPDATE_WORLDS).await.unwrap();

        Arc::new(PgConnection {
            client: cl,
            fortune,
            world,
            updates,
        })
    }
}

impl PgConnection {
    pub async fn fetch_world_by_id(&self, id: i32) -> Result<World, PgError> {
        self.client
            .query_one(&self.world, &[&id])
            .await
            .map(|row| {
                Ok(World {
                    id: row.get(0),
                    randomnumber: row.get(1),
                })
            })?
    }

    pub async fn fetch_random_worlds(&self, num: usize) -> Result<Vec<World>, PgError> {
        let mut rng = SmallRng::from_rng(&mut rng());

        let futures = FuturesUnordered::new();

        for id in random_ids(&mut rng, num) {
            futures.push(self.fetch_world_by_id(id));
        }

        futures.try_collect().await
    }

    pub async fn update_worlds(&self, num: usize) -> Result<Vec<World>, PgError> {
        let mut worlds = self.fetch_random_worlds(num).await?;

        // Update the worlds with new random numbers
        let mut rng = SmallRng::from_rng(&mut rng());
        let mut ids = Vec::with_capacity(num);
        let mut nids = Vec::with_capacity(num);

        for w in &mut worlds {
            w.randomnumber = random_id(&mut rng);
            ids.push(&w.id);
            nids.push(&w.randomnumber);
        }

        // Update the random worlds in the database.
        self.client
            .execute(&self.updates, &[&ids, &nids])
            .await?;

        Ok(worlds)
    }

    pub async fn fetch_all_fortunes(&self) -> Result<Vec<Fortune>, PgError> {
        let mut fortunes = vec![Fortune {
            id: 0,
            message: Cow::Borrowed("Additional fortune added at request time."),
        }];

        let rows = self
            .client
            .query_raw::<_, _, &[i32; 0]>(&self.fortune, &[])
            .await?;

        pin!(rows);

        while let Some(row) = rows.next().await.transpose()? {
            fortunes.push(Fortune {
                id: row.get(0),
                message: Cow::Owned(row.get(1)),
            });
        }

        fortunes.sort_by(|it, next| it.message.cmp(&next.message));
        Ok(fortunes)
    }
}

pub struct DatabaseConnection(pub Arc<PgConnection>);

impl FromRequestParts<Arc<PgConnection>> for DatabaseConnection {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut Parts,
        pg_connection: &Arc<PgConnection>,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(pg_connection.clone()))
    }
}
