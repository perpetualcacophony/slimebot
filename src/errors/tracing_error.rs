trait Event {
    fn level(&self) -> tracing::Level;
    fn construct(&self);
}

trait Span {
    fn level(&self) -> tracing::Level;
    fn construct(&self) -> tracing::Span;
    fn inner(&self) -> &dyn ToSpanOrEvent;
}

trait Dispatch {
    fn dispatch(&self);
}

enum SpanOrEvent<'a> {
    Span(&'a dyn Span),
    Event(&'a dyn Event),
}

impl<'a> Dispatch for SpanOrEvent<'a> {
    fn dispatch(&self) {
        match self {
            Self::Span(span) => {
                let tracing_span = span.construct();
                let _enter = tracing_span.enter();

                span.inner().dispatch();
            }
            Self::Event(event) => event.construct(),
        }
    }
}

trait ToSpanOrEvent {
    fn to_span_or_event(&self) -> SpanOrEvent<'_>;
}

impl<T: ToSpanOrEvent + ?Sized> Dispatch for T {
    fn dispatch(&self) {
        self.to_span_or_event().dispatch()
    }
}

trait TracingError {
    fn trace(&self);
}

impl<T: Dispatch> TracingError for T {
    fn trace(&self) {
        self.dispatch();
    }
}
