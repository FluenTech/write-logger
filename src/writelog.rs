// Copyright 2016 Victor Brekenfeld
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Module providing the FileLogger Implementation

use super::logging::try_log;
use crate::{Config, Wrapper};
use log::{set_logger, set_max_level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use rtfm::Mutex;

use core::borrow::BorrowMut;
use core::cell::UnsafeCell;
use core::fmt::{Display, Write};
use core::marker::PhantomData;
use core::ops::DerefMut;

use clock_trait::Instant;

/// The WriteLogger struct. Provides a Logger implementation for structs implementing `Write`, e.g. File
pub struct WriteLogger<Clock, W>
where
    W: Mutex,
    W::T: Write,
{
    level: LevelFilter,
    config: Config,
    writable: Wrapper<W>,
    phantom: PhantomData<Clock>,
}

impl<Clock, W> WriteLogger<Clock, W>
where
    W: Mutex,
    W::T: Write,
    Clock: Instant + Display + Sync + Send,
{
    /// init function. Globally initializes the WriteLogger as the one and only used log facility.
    ///
    /// Takes the desired `Level`, `Config` and `Write` struct as arguments. They cannot be changed later on.
    /// Fails if another Logger was already initialized.
    ///
    /// # Examples
    /// ```ignore
    /// # extern crate simplelog;
    /// # use simplelog::*;
    /// # use std::fs::File;
    /// # fn main() {
    /// let _ = WriteLogger::init(LevelFilter::Info, Config::default(), File::create("my_rust_bin.log").unwrap());
    /// # }
    /// ```
    pub fn set_logger(&'static self) -> &Self {
        set_max_level(self.level);
        set_logger(self).unwrap();
        self
    }

    /// allows to create a new logger, that can be independently used, no matter what is globally set.
    ///
    /// no macros are provided for this case and you probably
    /// dont want to use this function, but `init()`, if you dont want to build a `CombinedLogger`.
    ///
    /// Takes the desired `Level`, `Config` and `Write` struct as arguments. They cannot be changed later on.
    ///
    /// # Examples
    /// ```ignore
    /// # extern crate simplelog;
    /// # use simplelog::*;
    /// # use std::fs::File;
    /// # fn main() {
    /// let file_logger = WriteLogger::new(LevelFilter::Info, Config::default(), File::create("my_rust_bin.log").unwrap());
    /// # }
    /// ```
    pub const fn new(
        log_level: LevelFilter,
        config: Config,
        writable: Wrapper<W>,
    ) -> WriteLogger<Clock, W> {
        WriteLogger {
            level: log_level,
            config: config,
            writable: writable,
            phantom: PhantomData,
        }
    }
}

impl<Clock, W> Log for WriteLogger<Clock, W>
where
    W: Mutex,
    W::T: Write,
    Clock: Instant + Display + Sync + Send,
{
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            unsafe {
                (*self.writable.0.get())
                    .lock(|writer| try_log::<Clock, _>(&self.config, record, writer));
            }
        }
    }

    fn flush(&self) {
        // let _ = self.writable.lock().unwrap().flush();
    }
}

// impl<W> SharedLogger for WriteLogger<W>
// where
// // W: Mutex,
// // W::T: Write + Send + 'static,
// // W: core::marker::Sync + core::marker::Send,
// {
//     fn level(&self) -> LevelFilter {
//         self.level
//     }
//
//     fn config(&self) -> Option<&Config> {
//         Some(&self.config)
//     }
//
//     // fn as_log(self: Box<Self>) -> Box<dyn Log> {
//     //     Box::new(*self)
//     // }
// }
