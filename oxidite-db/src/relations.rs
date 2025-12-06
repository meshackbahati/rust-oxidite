use crate::{Database, Model, Result};
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
        let query = format!(
            "SELECT * FROM {} WHERE {} = {}",
            C::table_name(),
            self.foreign_key,
            self.parent_id
        );
        let rows = db.query(&query).await?;
        
        let mut models = Vec::new();
        for row in rows {
            models.push(C::from_row(&row)?);
        }
        Ok(models)
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
        let query = format!(
            "SELECT * FROM {} WHERE {} = {}",
            C::table_name(),
            self.foreign_key,
            self.parent_id
        );
        let row = db.query_one(&query).await?;
        
        match row {
            Some(row) => Ok(Some(C::from_row(&row)?)),
            None => Ok(None),
        }
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
