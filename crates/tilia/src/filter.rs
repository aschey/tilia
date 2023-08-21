use std::marker::PhantomData;
use std::sync::atomic::Ordering;

use tracing::metadata::LevelFilter;
use tracing::subscriber::Interest;

use crate::state;

pub struct Filter<F, S>
where
    F: tracing_subscriber::layer::Filter<S>,
{
    inner: F,
    _phantom: PhantomData<S>,
}

impl<F, S> Filter<F, S>
where
    F: tracing_subscriber::layer::Filter<S>,
{
    pub fn new(inner: F) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<S> Default for Filter<LevelFilter, S> {
    fn default() -> Self {
        Self {
            inner: LevelFilter::TRACE,
            _phantom: Default::default(),
        }
    }
}

impl<F, S> tracing_subscriber::layer::Filter<S> for Filter<F, S>
where
    F: tracing_subscriber::layer::Filter<S>,
{
    fn enabled(
        &self,
        meta: &tracing::Metadata<'_>,
        cx: &tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        if state::IS_ENABLED.load(Ordering::SeqCst) {
            self.inner.enabled(meta, cx)
        } else {
            false
        }
    }

    fn callsite_enabled(&self, meta: &'static tracing::Metadata<'static>) -> Interest {
        if state::IS_ENABLED.load(Ordering::SeqCst) {
            self.inner.callsite_enabled(meta)
        } else {
            Interest::never()
        }
    }
}
