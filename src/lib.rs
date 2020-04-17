// Copyright 2020 Peter Taylor
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//!
//! `write-logger` no_std logger.
//!

#![deny(rust_2018_idioms)]
#![cfg_attr(not(test), no_std)]
#![feature(const_fn)]
#![feature(const_mut_refs)]

// #[macro_use]
use lazy_static;

extern crate alloc;

mod config;
mod logging;
mod writelog;

pub use self::config::{Config, ConfigBuilder, LevelPadding, ThreadLogMode, ThreadPadding};
pub use log::*;
pub use writelog::WriteLogger;

use core::cell::UnsafeCell;
use rtfm::{Exclusive, Mutex};

pub struct Wrapper<T>(UnsafeCell<T>);

unsafe impl<T> Send for Wrapper<T> {}
unsafe impl<T> Sync for Wrapper<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::BorrowMut;
    use std::fmt::{Display, Formatter};
    use std::ops::{Deref, Sub};

    use clock_trait::prelude::*;
    use time::{Duration, NumericalDuration};

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    pub struct Instant {
        time: Duration,
    }

    impl clock_trait::Instant for Instant {
        fn now() -> Self {
            Instant {
                time: Duration::new(5025, 678_000_000),
            }
        }

        fn elapsed(self) -> Duration {
            Self::now() - self
        }
    }

    impl Display for Instant {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let hours = self.time.whole_hours().hours();
            let minutes = self.time.whole_minutes().minutes() - hours;
            let seconds = self.time.whole_seconds().seconds() - minutes - hours;
            let milliseconds = self.time.subsec_milliseconds().milliseconds();
            write!(
                f,
                "{:02}:{:02}:{:02}.{:03}",
                hours.whole_hours(),
                minutes.whole_minutes(),
                seconds.whole_seconds(),
                milliseconds.whole_milliseconds()
            )
        }
    }

    impl Sub for Instant {
        type Output = Duration;
        fn sub(self, other: Self) -> Self::Output {
            self.time - other.time
        }
    }

    static mut OUTPUT: String = String::new();
    lazy_static! {
        pub static ref LOGGER: WriteLogger<Instant, Exclusive<'static, String>> =
            WriteLogger::<Instant, _>::new(LevelFilter::Trace, Config::new(), unsafe {
                Wrapper(UnsafeCell::new(Exclusive(OUTPUT.borrow_mut())))
            });
    }

    #[test]
    fn test() {
        // static WRITER: Wrapper<Exclusive<'static, String>> =
        //     unsafe { Wrapper(UnsafeCell::new(Exclusive(&mut OUTPUT))) };
        LOGGER.deref().set_logger();

        error!("Test Error");
        // warn!("Test Warning");
        // info!("Test Information");
        // debug!("Test Debug");
        // trace!("Test Trace");

        unsafe {
            assert_eq!(
                OUTPUT,
                "01:23:45.678 [ERROR] write_log::tests: Test Error\n"
            )
        };
    }
}
