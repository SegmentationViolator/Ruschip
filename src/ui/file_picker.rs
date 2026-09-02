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

use std::future;
use std::pin;
use std::sync::{self, atomic};
use std::task;

pub struct File {
    bytes: Vec<u8>,
    name: String,
}

pub struct FilePicker {
    future: Option<pin::Pin<Box<dyn future::Future<Output = Option<File>>>>>,
    waker: Option<sync::Arc<Waker>>,
}

struct Waker {
    pub ready: atomic::AtomicBool,
}

impl task::Wake for Waker {
    fn wake(self: sync::Arc<Self>) {
        self.ready.store(true, atomic::Ordering::Relaxed);
    }
}

impl File {
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl FilePicker {
    #[inline]
    pub fn is_open(&self) -> bool {
        self.future.is_some()
    }

    pub fn new() -> Self {
        Self {
            future: None,
            waker: None,
        }
    }

    pub fn open(&mut self) {
        if self.is_open() {
            return;
        }

        self.future = Some(Box::pin(async {
            let file = rfd::AsyncFileDialog::new().pick_file().await?;

            Some(File {
                name: file.file_name(),
                bytes: file.read().await,
            })
        }));
    }

    pub fn poll(&mut self) -> Option<File> {
        if let Some(waker) = self.waker.as_ref()
            && !waker.ready.load(atomic::Ordering::Relaxed)
        {
            return None;
        }

        let future = self.future.as_mut()?;
        let waker = task::Waker::from(
            self.waker
                .get_or_insert_with(|| sync::Arc::new(Waker::new()))
                .clone(),
        );
        let mut context = task::Context::from_waker(&waker);

        match future.as_mut().poll(&mut context) {
            task::Poll::Pending => None,
            task::Poll::Ready(file) => {
                self.future = None;
                self.waker = None;
                file
            }
        }
    }
}

impl Waker {
    pub fn new() -> Self {
        Self {
            ready: atomic::AtomicBool::new(false),
        }
    }
}
