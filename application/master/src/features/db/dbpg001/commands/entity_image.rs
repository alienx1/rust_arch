//! DBPG001 image actions: logo of an organization or a company (object storage + `image_key`)

use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::{ImageOwner, StoredImage, db_company, db_organization};
use crate::features::db::image_object_store::{ImageObjectStore, ImageStoreError};

use super::dbpg001_executor::Dbpg001CommandExecutor;

impl From<ImageStoreError> for Dbpg001Error {
    fn from(err: ImageStoreError) -> Self {
        match err {
            ImageStoreError::Invalid(m) => Self::Validation(m),
            ImageStoreError::Storage(m) => Self::Internal(m),
        }
    }
}

impl Dbpg001CommandExecutor {
    /// Stores `bytes` as the owner's new image and forgets the previous one. The row version is
    /// left alone: an open edit form must still be able to save.
    pub(super) async fn set_image(&self, owner: ImageOwner, id: Uuid, bytes: Vec<u8>) -> Result<(), Dbpg001Error> {
        let store = self.image_store()?;
        let old = self.image_key(owner, id).await?;
        let key = store.store(owner, id, bytes).await?;
        if let Err(e) = self.store_image_key(owner, id, Some(key.clone())).await {
            store.forget(&key).await;
            return Err(e);
        }
        if let Some(old) = old {
            store.forget(&old).await;
        }
        Ok(())
    }

    /// Removes the owner's image (no image is fine)
    pub(super) async fn remove_image(&self, owner: ImageOwner, id: Uuid) -> Result<(), Dbpg001Error> {
        if let Some(old) = self.image_key(owner, id).await? {
            self.store_image_key(owner, id, None).await?;
            self.image_store()?.forget(&old).await;
        }
        Ok(())
    }

    /// The owner's image bytes; `NotFound` when it has none
    pub(super) async fn read_image(&self, owner: ImageOwner, id: Uuid) -> Result<StoredImage, Dbpg001Error> {
        let key = self
            .image_key(owner, id)
            .await?
            .ok_or_else(|| Dbpg001Error::NotFound(format!("{id} has no image")))?;
        Ok(self.image_store()?.read(&key).await?)
    }

    /// Best-effort removal of the image object of a row that has just been deleted
    pub(super) async fn forget_object(&self, key: &str) {
        if let Ok(store) = self.image_store() {
            store.forget(key).await;
        }
    }

    fn image_store(&self) -> Result<&ImageObjectStore, Dbpg001Error> {
        self.images.as_ref().ok_or_else(|| Dbpg001Error::Internal("Image storage is not configured".to_string()))
    }

    /// `NotFound` when the owner itself does not exist
    async fn image_key(&self, owner: ImageOwner, id: Uuid) -> Result<Option<String>, Dbpg001Error> {
        match owner {
            ImageOwner::Organization => db_organization::Entity::find_by_id(id)
                .one(&self.db)
                .await?
                .map(|o| o.image_key)
                .ok_or_else(|| Dbpg001Error::NotFound(format!("Organization {id} not found"))),
            ImageOwner::Company => db_company::Entity::find_by_id(id)
                .one(&self.db)
                .await?
                .map(|c| c.image_key)
                .ok_or_else(|| Dbpg001Error::NotFound(format!("Company {id} not found"))),
            ImageOwner::Employee => Err(Dbpg001Error::Internal("Employee images belong to DBPG003".to_string())),
        }
    }

    async fn store_image_key(&self, owner: ImageOwner, id: Uuid, key: Option<String>) -> Result<(), Dbpg001Error> {
        match owner {
            ImageOwner::Organization => {
                let mut row = db_organization::ActiveModel { id: Set(id), ..Default::default() };
                row.image_key = Set(key);
                row.update(&self.db).await?;
            }
            ImageOwner::Company => {
                let mut row = db_company::ActiveModel { id: Set(id), ..Default::default() };
                row.image_key = Set(key);
                row.update(&self.db).await?;
            }
            ImageOwner::Employee => {
                return Err(Dbpg001Error::Internal("Employee images belong to DBPG003".to_string()));
            }
        }
        Ok(())
    }
}
