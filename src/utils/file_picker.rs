use rfd::AsyncFileDialog;

use crate::{LocalFile, Tag, http};

pub async fn prompt_image_upload(tag: Tag) -> Option<String> {
    if let Some(file) = AsyncFileDialog::new().pick_file().await {
        let contents = file.read().await.into();
        let filename = file.file_name();

        if let Ok(response) = http()
            .upload_file(
                tag.as_str(),
                LocalFile {
                    name: filename,
                    body: contents,
                },
            )
            .await
        {
            return Some(response.id);
        };
    };

    None
}
