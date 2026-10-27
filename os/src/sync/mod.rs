//! Synchronization and interior mutability primitives

mod condvar;
mod mutex;
mod semaphore;
mod up;

pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;
use lazy_static::*;

lazy_static! {
    /// Global switch of deadlock detection (set by sys_enable_deadlock_detect)
    pub static ref ENABLE_DEADLOCK_DETECT: UPSafeCell<bool> =
        unsafe { UPSafeCell::new(false) };
}
