use crate::{is_valid_identifier, Database, Model, Result};
use sqlx::Row;
use std::collections::HashMap;
use std::marker::PhantomData;

/// Represents a one-to-many relationship
pub struct HasMany<P, C> {
    parent_id: i64,
    foreign_key: String,
    _phantom: PhantomData<(P, C)>,
}

impl<P, C> HasMany<P, C>
where
    P: Model,
    C: Model,
{
    pub fn new(parent_id: i64, foreign_key: impl Into<String>) -> Self {
        Self {
            parent_id,
            foreign_key: foreign_key.into(),
            _phantom: PhantomData,
        }
    }

    /// Fetch all related records
    pub async fn get(&self, db: &impl Database) -> Result<Vec<C>> {
        if !is_valid_identifier(&self.foreign_key) {
            return Err(sqlx::Error::Protocol(
                format!("invalid foreign key identifier `{}`", self.foreign_key).into(),
            ));
        }

        let query = format!(
            "SELECT * FROM {} WHERE {} = ?",
            C::table_name(),
            self.foreign_key,
        );
        let rows = db
            .fetch_all(sqlx::query(&query).bind(self.parent_id))
            .await?;

        let mut models = Vec::new();
        for row in rows {
            models.push(C::from_row(&row)?);
        }
        Ok(models)
    }

    /// Eager-load related rows for many parent ids in one query.
    pub async fn eager_load(
        db: &impl Database,
        parent_ids: &[i64],
        foreign_key: impl AsRef<str>,
    ) -> Result<HashMap<i64, Vec<C>>> {
        let foreign_key = foreign_key.as_ref();
        if !is_valid_identifier(foreign_key) {
            return Err(sqlx::Error::Protocol(
                format!("invalid foreign key identifier `{foreign_key}`").into(),
            ));
        }

        if parent_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders = std::iter::repeat("?")
            .take(parent_ids.len())
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            "SELECT * FROM {} WHERE {} IN ({})",
            C::table_name(),
            foreign_key,
            placeholders
        );

        let mut sql_query = sqlx::query(&query);
        for parent_id in parent_ids {
            sql_query = sql_query.bind(*parent_id);
        }

        let rows = db.fetch_all(sql_query).await?;
        let mut grouped = HashMap::<i64, Vec<C>>::new();
        for parent_id in parent_ids {
            grouped.insert(*parent_id, Vec::new());
        }

        for row in rows {
            let fk_value: i64 = row.try_get(foreign_key)?;
            let child = C::from_row(&row)?;
            grouped.entry(fk_value).or_default().push(child);
        }

        Ok(grouped)
    }
}

/// Represents a one-to-one relationship (owned)
pub struct HasOne<P, C> {
    parent_id: i64,
    foreign_key: String,
    _phantom: PhantomData<(P, C)>,
}

impl<P, C> HasOne<P, C>
where
    P: Model,
    C: Model,
{
    pub fn new(parent_id: i64, foreign_key: impl Into<String>) -> Self {
        Self {
            parent_id,
            foreign_key: foreign_key.into(),
            _phantom: PhantomData,
        }
    }

    /// Fetch the related record
    pub async fn get(&self, db: &impl Database) -> Result<Option<C>> {
        if !is_valid_identifier(&self.foreign_key) {
            return Err(sqlx::Error::Protocol(
                format!("invalid foreign key identifier `{}`", self.foreign_key).into(),
            ));
        }

        let query = format!(
            "SELECT * FROM {} WHERE {} = ?",
            C::table_name(),
            self.foreign_key,
        );
        let row = db
            .fetch_one(sqlx::query(&query).bind(self.parent_id))
            .await?;

        match row {
            Some(row) => Ok(Some(C::from_row(&row)?)),
            None => Ok(None),
        }
    }

    /// Eager-load one related row per parent id.
    pub async fn eager_load(
        db: &impl Database,
        parent_ids: &[i64],
        foreign_key: impl AsRef<str>,
    ) -> Result<HashMap<i64, Option<C>>> {
        let foreign_key = foreign_key.as_ref();
        if !is_valid_identifier(foreign_key) {
            return Err(sqlx::Error::Protocol(
                format!("invalid foreign key identifier `{foreign_key}`").into(),
            ));
        }

        if parent_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders = std::iter::repeat("?")
            .take(parent_ids.len())
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            "SELECT * FROM {} WHERE {} IN ({})",
            C::table_name(),
            foreign_key,
            placeholders
        );

        let mut sql_query = sqlx::query(&query);
        for parent_id in parent_ids {
            sql_query = sql_query.bind(*parent_id);
        }

        let rows = db.fetch_all(sql_query).await?;
        let mut grouped = HashMap::<i64, Option<C>>::new();
        for parent_id in parent_ids {
            grouped.insert(*parent_id, None);
        }

        for row in rows {
            let fk_value: i64 = row.try_get(foreign_key)?;
            let child = C::from_row(&row)?;
            grouped.entry(fk_value).or_insert(Some(child));
        }

        Ok(grouped)
    }
}

/// Represents a belongs-to relationship (inverse of HasMany/HasOne)
pub struct BelongsTo<C, P> {
    foreign_key_value: i64,
    _phantom: PhantomData<(C, P)>,
}

impl<C, P> BelongsTo<C, P>
where
    C: Model,
    P: Model,
{
    pub fn new(foreign_key_value: i64) -> Self {
        Self {
            foreign_key_value,
            _phantom: PhantomData,
        }
    }

    /// Fetch the parent record
    pub async fn get(&self, db: &impl Database) -> Result<Option<P>> {
        P::find(db, self.foreign_key_value).await
    }
}
