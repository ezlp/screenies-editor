//! clipboard.rs — put the rendered image on the system clipboard so the
//! user can paste straight into Discord / forum ("Copy ke Clipboard").

use crate::error::AppError;
use image::RgbaImage;
use std::borrow::Cow;

pub fn copy_image(img: &RgbaImage) -> Result<(), AppError> {
    let mut cb = arboard::Clipboard::new().map_err(|e| AppError::Io(e.to_string()))?;
    cb.set_image(arboard::ImageData {
        width: img.width() as usize,
        height: img.height() as usize,
        bytes: Cow::Borrowed(img.as_raw()),
    })
    .map_err(|e| AppError::Io(e.to_string()))
}
