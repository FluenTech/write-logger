use crate::{Config, LevelPadding, ThreadPadding};

use log::{LevelFilter, Record};

use core::fmt::{self, Display, Error, Formatter, Write};
use core::ops::Deref;

use clock_trait::Instant;

#[inline(always)]
pub fn try_log<Clock, W>(config: &Config, record: &Record<'_>, write: &mut W) -> Result<(), Error>
where
    W: Write + Sized,
    Clock: Instant + Display,
{
    if should_skip(config, record) {
        return Ok(());
    }

    if config.time <= record.level() && config.time != LevelFilter::Off {
        write_time::<Clock, _>(write, config)?;
    }

    if config.level <= record.level() && config.level != LevelFilter::Off {
        write_level(record, write, config)?;
    }

    if config.thread <= record.level() && config.thread != LevelFilter::Off {
        write_thread_name(write, config)?;
    }

    if config.target <= record.level() && config.target != LevelFilter::Off {
        write_target(record, write)?;
    }

    if config.location <= record.level() && config.location != LevelFilter::Off {
        write_location(record, write)?;
    }

    write_args(record, write)
}

// static SYSTEM_CLOCK: Clock =  Clock::init(48_000_000);
//
// struct Clock {
//     ticks: Mutex<Data = u64>,
//     frequency: u64,
// }
//
// impl Clock {
//     pub const fn init(frequency: u64) -> Clock {
//         Clock{ticks: Mutex::new(0u64), frequency}
//     }
//
//     pub fn to_nanos(&self) -> u64 {
//         let ticks = self.ticks.lock();
//         self.frequency * 1_000_000_000 / ticks.deref()
//     }
// }
//
// struct Instant(core::time::Duration);
//
// impl Instant {
//     pub fn now() -> Instant {
//         unsafe {
//             Instant(core::time::Duration::from_nanos(
//                 SYSTEM_CLOCK.to_nanos(),
//             ))
//         }
//     }
//
//     pub fn duration_since_epoch(&self) -> Duration {
//         todo!()
//     }
// }
//
// struct Duration(core::time::Duration);
//
// impl Duration {
//     pub fn as_hours(&self) -> u64 {
//         self.as_mins() / 60
//     }
//
//     pub fn as_mins(&self) -> u64 {
//         self.as_secs() / 60
//     }
//     pub fn as_secs(&self) -> u64 {
//         self.0.as_secs()
//     }
// }
//
// impl core::fmt::Display for Duration {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "{}:{}:{}",
//             self.as_hours(),
//             self.as_mins(),
//             self.as_secs()
//         )
//     }
// }

#[inline(always)]
pub fn write_time<Clock, W>(write: &mut W, config: &Config) -> Result<(), Error>
where
    W: Write + Sized,
    Clock: Instant + Display,
{
    let cur_time = Clock::now();

    write!(write, "{} ", cur_time)?;
    Ok(())
}

#[inline(always)]
pub fn write_level<W>(record: &Record<'_>, write: &mut W, config: &Config) -> Result<(), Error>
where
    W: Write + Sized,
{
    match config.level_padding {
        LevelPadding::Left => write!(write, "[{: >5}] ", record.level())?,
        LevelPadding::Right => write!(write, "[{: <5}] ", record.level())?,
        LevelPadding::Off => write!(write, "[{}] ", record.level())?,
    };
    Ok(())
}

#[inline(always)]
pub fn write_target<W>(record: &Record<'_>, write: &mut W) -> Result<(), Error>
where
    W: Write + Sized,
{
    write!(write, "{}: ", record.target())?;
    Ok(())
}

#[inline(always)]
pub fn write_location<W>(record: &Record<'_>, write: &mut W) -> Result<(), Error>
where
    W: Write + Sized,
{
    let file = record.file().unwrap_or("<unknown>");
    if let Some(line) = record.line() {
        write!(write, "[{}:{}] ", file, line)?;
    } else {
        write!(write, "[{}:<unknown>] ", file)?;
    }
    Ok(())
}

pub fn write_thread_name<W>(write: &mut W, config: &Config) -> Result<(), Error>
where
    W: Write + Sized,
{
    let name = module_path!();
    match config.thread_padding {
        ThreadPadding::Left { 0: qty } => {
            write!(write, "({name:>0$}) ", qty, name = name)?;
        }
        ThreadPadding::Right { 0: qty } => {
            write!(write, "({name:<0$}) ", qty, name = name)?;
        }
        ThreadPadding::Off => {
            write!(write, "({}) ", name)?;
        }
    }

    Ok(())
}

// pub fn write_thread_id<W>(write: &mut W, config: &Config) -> Result<(), Error>
// where
//     W: Write + Sized,
// {
//     let id = format!("{:?}", thread::current().id());
//     let id = id.replace("ThreadId(", "");
//     let id = id.replace(")", "");
//     match config.thread_padding {
//         ThreadPadding::Left{0: qty} => {
//             write!(write, "({id:>0$}) ", qty, id=id)?;
//         }
//         ThreadPadding::Right{0: qty} => {
//             write!(write, "({id:<0$}) ", qty, id=id)?;
//         }
//         ThreadPadding::Off => {
//             write!(write, "({}) ", id)?;
//         }
//     }
//     Ok(())
// }

#[inline(always)]
pub fn write_args<W>(record: &Record<'_>, write: &mut W) -> Result<(), Error>
where
    W: Write + Sized,
{
    writeln!(write, "{}", record.args())?;
    Ok(())
}

#[inline(always)]
pub fn should_skip(config: &Config, record: &Record<'_>) -> bool {
    // If a module path and allowed list are available
    // match (record.target(), &*config.filter_allow) {
    //     (path, allowed) if allowed.len() > 0 => {
    //         // Check that the module path matches at least one allow filter
    //         if let None = allowed.iter().find(|v| path.starts_with(&***v)) {
    //             // If not, skip any further writing
    //             return true;
    //         }
    //     }
    //     _ => {}
    // }
    //
    // // If a module path and ignore list are available
    // match (record.target(), &*config.filter_ignore) {
    //     (path, ignore) if ignore.len() > 0 => {
    //         // Check that the module path does not match any ignore filters
    //         if let Some(_) = ignore.iter().find(|v| path.starts_with(&***v)) {
    //             // If not, skip any further writing
    //             return true;
    //         }
    //     }
    //     _ => {}
    // }

    return false;
}
