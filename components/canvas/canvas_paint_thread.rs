/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use webrender_api::{ImageData, ImageDescriptor, ImageKey};

pub enum AntialiasMode {
    Default,
    None,
}

pub enum ImageUpdate {
    Add(ImageKey, ImageDescriptor, ImageData),
    Update(ImageKey, ImageDescriptor, ImageData),
    Delete(ImageKey),
}

pub trait WebrenderApi: Send {
    fn generate_key(&self) -> Result<webrender_api::ImageKey, ()>;
    fn update_images(&self, updates: Vec<ImageUpdate>);
    fn clone(&self) -> Box<dyn WebrenderApi>;
}
