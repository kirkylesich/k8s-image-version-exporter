use core::time::Duration;

use crate::backoff::BackoffBuilder;
use crate::blocking_sleep::MaybeBlockingSleeper;
use crate::Backoff;
use crate::BlockingSleeper;
use crate::DefaultBlockingSleeper;

/// BlockingRetryable adds retry support for blocking functions.
///
/// For example:
///
/// - Functions without extra args:
///
/// ```ignore
/// fn fetch() -> Result<String> {
///     Ok("hello, world!".to_string())
/// }
/// ```
///
/// - Closures
///
/// ```ignore
/// || {
///     Ok("hello, world!".to_string())
/// }
/// ```
///
/// # Example
///
/// ```no_run
/// use anyhow::Result;
/// use backon::BlockingRetryable;
/// use backon::ExponentialBuilder;
///
/// fn fetch() -> Result<String> {
///     Ok("hello, world!".to_string())
/// }
///
/// fn main() -> Result<()> {
///     let content = fetch.retry(ExponentialBuilder::default()).call()?;
///     println!("fetch succeeded: {}", content);
///
///     Ok(())
/// }
/// ```
pub trait BlockingRetryable<B: BackoffBuilder, T, E, F: FnMut() -> Result<T, E>> {
    /// Generate a new retry.
    fn retry(self, builder: B) -> BlockingRetry<B::Backoff, T, E, F>;
}

impl<B, T, E, F> BlockingRetryable<B, T, E, F> for F
where
    B: BackoffBuilder,
    F: FnMut() -> Result<T, E>,
{
    fn retry(self, builder: B) -> BlockingRetry<B::Backoff, T, E, F> {
        BlockingRetry::new(self, builder.build())
    }
}

/// Retry structure generated by [`BlockingRetryable`].
pub struct BlockingRetry<
    B: Backoff,
    T,
    E,
    F: FnMut() -> Result<T, E>,
    SF: MaybeBlockingSleeper = DefaultBlockingSleeper,
    RF = fn(&E) -> bool,
    NF = fn(&E, Duration),
> {
    backoff: B,
    retryable: RF,
    notify: NF,
    f: F,
    sleep_fn: SF,
}

impl<B, T, E, F> BlockingRetry<B, T, E, F>
where
    B: Backoff,
    F: FnMut() -> Result<T, E>,
{
    /// Create a new retry.
    fn new(f: F, backoff: B) -> Self {
        BlockingRetry {
            backoff,
            retryable: |_: &E| true,
            notify: |_: &E, _: Duration| {},
            sleep_fn: DefaultBlockingSleeper::default(),
            f,
        }
    }
}

impl<B, T, E, F, SF, RF, NF> BlockingRetry<B, T, E, F, SF, RF, NF>
where
    B: Backoff,
    F: FnMut() -> Result<T, E>,
    SF: MaybeBlockingSleeper,
    RF: FnMut(&E) -> bool,
    NF: FnMut(&E, Duration),
{
    /// Set the sleeper for retrying.
    ///
    /// The sleeper should implement the [`BlockingSleeper`] trait. The simplest way is to use a closure like  `Fn(Duration)`.
    ///
    /// If not specified, we use the [`DefaultBlockingSleeper`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use anyhow::Result;
    /// use backon::BlockingRetryable;
    /// use backon::ExponentialBuilder;
    ///
    /// fn fetch() -> Result<String> {
    ///     Ok("hello, world!".to_string())
    /// }
    ///
    /// fn main() -> Result<()> {
    ///     let retry = fetch
    ///         .retry(ExponentialBuilder::default())
    ///         .sleep(std::thread::sleep);
    ///     let content = retry.call()?;
    ///     println!("fetch succeeded: {}", content);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn sleep<SN: BlockingSleeper>(self, sleep_fn: SN) -> BlockingRetry<B, T, E, F, SN, RF, NF> {
        BlockingRetry {
            backoff: self.backoff,
            retryable: self.retryable,
            notify: self.notify,
            f: self.f,
            sleep_fn,
        }
    }

    /// Set the conditions for retrying.
    ///
    /// If not specified, all errors are considered retryable.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use anyhow::Result;
    /// use backon::BlockingRetryable;
    /// use backon::ExponentialBuilder;
    ///
    /// fn fetch() -> Result<String> {
    ///     Ok("hello, world!".to_string())
    /// }
    ///
    /// fn main() -> Result<()> {
    ///     let retry = fetch
    ///         .retry(ExponentialBuilder::default())
    ///         .when(|e| e.to_string() == "EOF");
    ///     let content = retry.call()?;
    ///     println!("fetch succeeded: {}", content);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn when<RN: FnMut(&E) -> bool>(
        self,
        retryable: RN,
    ) -> BlockingRetry<B, T, E, F, SF, RN, NF> {
        BlockingRetry {
            backoff: self.backoff,
            retryable,
            notify: self.notify,
            f: self.f,
            sleep_fn: self.sleep_fn,
        }
    }

    /// Set to notify for all retry attempts.
    ///
    /// When a retry happens, the input function will be invoked with the error and the sleep duration before pausing.
    ///
    /// If not specified, this operation does nothing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use core::time::Duration;
    ///
    /// use anyhow::Result;
    /// use backon::BlockingRetryable;
    /// use backon::ExponentialBuilder;
    ///
    /// fn fetch() -> Result<String> {
    ///     Ok("hello, world!".to_string())
    /// }
    ///
    /// fn main() -> Result<()> {
    ///     let retry = fetch.retry(ExponentialBuilder::default()).notify(
    ///         |err: &anyhow::Error, dur: Duration| {
    ///             println!("retrying error {:?} with sleeping {:?}", err, dur);
    ///         },
    ///     );
    ///     let content = retry.call()?;
    ///     println!("fetch succeeded: {}", content);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn notify<NN: FnMut(&E, Duration)>(
        self,
        notify: NN,
    ) -> BlockingRetry<B, T, E, F, SF, RF, NN> {
        BlockingRetry {
            backoff: self.backoff,
            retryable: self.retryable,
            notify,
            f: self.f,
            sleep_fn: self.sleep_fn,
        }
    }
}

impl<B, T, E, F, SF, RF, NF> BlockingRetry<B, T, E, F, SF, RF, NF>
where
    B: Backoff,
    F: FnMut() -> Result<T, E>,
    SF: BlockingSleeper,
    RF: FnMut(&E) -> bool,
    NF: FnMut(&E, Duration),
{
    /// Call the retried function.
    ///
    /// TODO: implement [`FnOnce`] after it stable.
    pub fn call(mut self) -> Result<T, E> {
        loop {
            let result = (self.f)();

            match result {
                Ok(v) => return Ok(v),
                Err(err) => {
                    if !(self.retryable)(&err) {
                        return Err(err);
                    }

                    match self.backoff.next() {
                        None => return Err(err),
                        Some(dur) => {
                            (self.notify)(&err, dur);
                            self.sleep_fn.sleep(dur);
                        }
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;
    use core::time::Duration;

    use spin::Mutex;

    use super::*;
    use crate::ExponentialBuilder;

    fn always_error() -> anyhow::Result<()> {
        Err(anyhow::anyhow!("test_query meets error"))
    }

    #[test]
    fn test_retry() -> anyhow::Result<()> {
        let result = always_error
            .retry(ExponentialBuilder::default().with_min_delay(Duration::from_millis(1)))
            .call();

        assert!(result.is_err());
        assert_eq!("test_query meets error", result.unwrap_err().to_string());
        Ok(())
    }

    #[test]
    fn test_retry_with_not_retryable_error() -> anyhow::Result<()> {
        let error_times = Mutex::new(0);

        let f = || {
            let mut x = error_times.lock();
            *x += 1;
            Err::<(), anyhow::Error>(anyhow::anyhow!("not retryable"))
        };

        let backoff = ExponentialBuilder::default().with_min_delay(Duration::from_millis(1));
        let result = f
            .retry(backoff)
            // Only retry If error message is `retryable`
            .when(|e| e.to_string() == "retryable")
            .call();

        assert!(result.is_err());
        assert_eq!("not retryable", result.unwrap_err().to_string());
        // `f` always returns error "not retryable", so it should be executed
        // only once.
        assert_eq!(*error_times.lock(), 1);
        Ok(())
    }

    #[test]
    fn test_retry_with_retryable_error() -> anyhow::Result<()> {
        let error_times = Mutex::new(0);

        let f = || {
            // println!("I have been called!");
            let mut x = error_times.lock();
            *x += 1;
            Err::<(), anyhow::Error>(anyhow::anyhow!("retryable"))
        };

        let backoff = ExponentialBuilder::default().with_min_delay(Duration::from_millis(1));
        let result = f
            .retry(backoff)
            // Only retry If error message is `retryable`
            .when(|e| e.to_string() == "retryable")
            .call();

        assert!(result.is_err());
        assert_eq!("retryable", result.unwrap_err().to_string());
        // `f` always returns error "retryable", so it should be executed
        // 4 times (retry 3 times).
        assert_eq!(*error_times.lock(), 4);
        Ok(())
    }

    #[test]
    fn test_fn_mut_when_and_notify() -> anyhow::Result<()> {
        let mut calls_retryable: Vec<()> = vec![];
        let mut calls_notify: Vec<()> = vec![];

        let f = || Err::<(), anyhow::Error>(anyhow::anyhow!("retryable"));

        let backoff = ExponentialBuilder::default().with_min_delay(Duration::from_millis(1));
        let result = f
            .retry(backoff)
            .when(|_| {
                calls_retryable.push(());
                true
            })
            .notify(|_, _| {
                calls_notify.push(());
            })
            .call();

        assert!(result.is_err());
        assert_eq!("retryable", result.unwrap_err().to_string());
        // `f` always returns error "retryable", so it should be executed
        // 4 times (retry 3 times).
        assert_eq!(calls_retryable.len(), 4);
        assert_eq!(calls_notify.len(), 3);
        Ok(())
    }
}
