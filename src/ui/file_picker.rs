//    Ruschip - a multi-variant CHIP-8 emulator
//    Copyright (C) 2023 Segmentation Violator <segmentationviolator@proton.me>

//    This program is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.

//    This program is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//    GNU General Public License for more details.

//    You should have received a copy of the GNU General Public License
//    along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::future::Future;
use std::pin::Pin;
use std::sync;
use std::task;

pub struct SelectedFile {
    bytes: Vec<u8>,
    name: String,
}

pub struct FilePicker {
    future: Option<Pin<Box<dyn Future<Output = Option<SelectedFile>>>>>,
}

impl FilePicker {
    #[inline]
    pub fn is_open(&self) -> bool {
        self.future.is_some()
    }

    pub fn load(file: Option<&SelectedFile>) -> Option<Vec<u8>> {
        file.map(|file| file.bytes.clone())
    }

    pub fn new() -> Self {
        Self { future: None }
    }

    pub fn open(&mut self) {
        if self.future.is_some() {
            return;
        }

        self.future = Some(Box::pin(async {
            let file = rfd::AsyncFileDialog::new().pick_file().await?;

            Some(SelectedFile {
                name: file.file_name(),
                bytes: file.read().await,
            })
        }));
    }

    pub fn poll(&mut self) -> Option<SelectedFile> {
        let future = self.future.as_mut()?;
        let waker = task::Waker::from(sync::Arc::new(NoopWaker));
        let mut context = task::Context::from_waker(&waker);

        match future.as_mut().poll(&mut context) {
            task::Poll::Pending => None,
            task::Poll::Ready(file) => {
                self.future = None;
                file
            }
        }
    }
}

impl SelectedFile {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}

struct NoopWaker;

impl task::Wake for NoopWaker {
    fn wake(self: sync::Arc<Self>) {}
}
