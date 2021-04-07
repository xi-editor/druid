use crate::{WidgetPod, Env, Widget, Data, LifeCycle, EventCtx, PaintCtx, LifeCycleCtx, BoxConstraints, Size, LayoutCtx, Event, UpdateCtx, Point};

/// A widget wrapper which disables the inner widget if the provided closure return true.
pub struct DisabledIf<T, W> {
    inner: WidgetPod<T, W>,
    disabled_if: Box<dyn Fn(&T, &Env) -> bool>,
}

impl<T: Data, W: Widget<T>> DisabledIf<T, W> {
    /// Creates a new `DisabledIf` widget with the inner widget and the closure to decide if the
    /// widget should be disabled.
    pub fn new(widget: W, disabled_if: impl Fn(&T, &Env) -> bool + 'static) -> Self {
        DisabledIf {
            inner: WidgetPod::new(widget),
            disabled_if: Box::new(disabled_if),
        }
    }
}

impl<T: Data, W: Widget<T>> Widget<T> for DisabledIf<T, W> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            ctx.set_disabled((self.disabled_if)(data, env));
        }
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &T, data: &T, env: &Env) {
        ctx.set_disabled((self.disabled_if)(data, env));
        self.inner.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let size = self.inner.layout(ctx, bc, data, env);
        self.inner.set_origin(ctx, data, env, Point::ZERO);
        ctx.set_baseline_offset(self.inner.baseline_offset());
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.inner.paint(ctx, data, env);
    }
}
