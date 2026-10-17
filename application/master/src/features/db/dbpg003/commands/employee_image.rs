//! DBPG003 image actions: the photo of an employee (object storage + `db_employee.image_key`)

use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

use crate::features::db::dbpg003::commands::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{ImageOwner, StoredImage, db_employee};
use crate::features::db::image_object_store::{ImageObjectStore, ImageStoreError};

impl From<ImageStoreError> for Dbpg003Error {
    fn from(err: ImageStoreError) -> Self {
        match err {
            ImageStoreError::Invalid(m) => Self::Validation(m),
            ImageStoreError::Storage(m) => Self::Internal(m),
        }
    }
}

impl Dbpg003CommandExecutor {
    /// Stores `bytes` as the employee's new photo and forgets the previous one. The row version is
    /// left alone: an open edit form must still be able to save.
    pub(super) async fn set_image(&self, id: Uuid, bytes: Vec<u8>) -> Result<(), Dbpg003Error> {
        let store = self.image_store()?;
        let old = self.image_key(id).await?;
        let key = store.store(ImageOwner::Employee, id, bytes).await?;
        if let Err(e) = self.store_image_key(id, Some(key.clone())).await {
            store.forget(&key).await;
            return Err(e);
        }
        if let Some(old) = old {
            store.forget(&old).await;
        }
        Ok(())
    }

    /// Removes the employee's photo (no photo is fine)
    pub(super) async fn remove_image(&self, id: Uuid) -> Result<(), Dbpg003Error> {
        if let Some(old) = self.image_key(id).await? {
            self.store_image_key(id, None).await?;
            self.image_store()?.forget(&old).await;
        }
        Ok(())
    }

    /// The employee's photo bytes; `NotFound` when there is none
    pub(super) async fn read_image(&self, id: Uuid) -> Result<StoredImage, Dbpg003Error> {
        let key = self
            .image_key(id)
            .await?
            .ok_or_else(|| Dbpg003Error::NotFound(format!("{id} has no image")))?;
        Ok(self.image_store()?.read(&key).await?)
    }

    /// Best-effort removal of the photo object of an employee that has just been deleted
    pub(super) async fn forget_object(&self, key: &str) {
        if let Ok(store) = self.image_store() {
            store.forget(key).await;
        }
    }

    fn image_store(&self) -> Result<&ImageObjectStore, Dbpg003Error> {
        self.images.as_ref().ok_or_else(|| Dbpg003Error::Internal("Image storage is not configured".to_string()))
    }

    /// `NotFound` when the employee does not exist
    async fn image_key(&self, id: Uuid) -> Result<Option<String>, Dbpg003Error> {
        db_employee::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .map(|e| e.image_key)
            .ok_or_else(|| Dbpg003Error::NotFound(format!("Employee {id} not found")))
    }

    async fn store_image_key(&self, id: Uuid, key: Option<String>) -> Result<(), Dbpg003Error> {
        let mut row = db_employee::ActiveModel { id: Set(id), ..Default::default() };
        row.image_key = Set(key);
        row.update(&self.db).await?;
        Ok(())
    }
}
