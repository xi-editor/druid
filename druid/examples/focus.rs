use druid::{WindowDesc, AppLauncher, Widget, WidgetExt, Data, Lens, WidgetPod, EventCtx, LifeCycle, PaintCtx, LifeCycleCtx, BoxConstraints, LayoutCtx, Event, Env, UpdateCtx, RenderContext};
use druid::widget::{Flex, TextBox};
use piet_common::{UnitPoint, Color};
use piet_common::kurbo::{Size, Point};
use druid_shell::{HotKey, KbKey, SysMods};

struct FocusWrapper<T, W: Widget<T>> {
    inner: WidgetPod<T, W>,
}

impl<T: Data, W: Widget<T>> FocusWrapper<T, W> {
    pub fn new(widget: W) -> Self {
        FocusWrapper {inner: WidgetPod::new(widget)}
    }
}

impl<W: Widget<AppData>> Widget<AppData> for FocusWrapper<AppData, W> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        if let Event::KeyDown(ke) = event {
            if HotKey::new(None, KbKey::Tab).matches(ke) && ctx.is_focused() {
                ctx.focus_next();
            }
            if HotKey::new(SysMods::Shift, KbKey::Tab).matches(ke) && ctx.is_focused() {
                ctx.focus_prev();
            }
        }
        self.inner.event(ctx, event, data, env)

    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppData, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            ctx.register_for_focus();
        }

        if let LifeCycle::FocusChanged(_) = event {
            ctx.request_paint();
        }

        if let LifeCycle::DisabledChanged(disabled) = event {
            println!("disabled: {} for {:?}", disabled, ctx.widget_id());
        }

        self.inner.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &AppData, data: &AppData, env: &Env) {
        if data.text1.len() == 0 {
            ctx.set_disabled(true);
        } else {
            ctx.set_disabled(false);
        }

        self.inner.update(ctx, data, env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &AppData, env: &Env) -> Size {
        let size = self.inner.layout(ctx, &bc.shrink((8.0, 8.0)), data, env);
        self.inner.set_origin(ctx, data, env, Point::new(4.0, 4.0));
        size + Size::new(8.0, 8.0)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, env: &Env) {
        self.inner.paint(ctx, data, env);

        if ctx.is_focused() {
            let rect = ctx.size().to_rounded_rect(4.0);
            let brush = ctx.solid_brush(Color::AQUA);
            ctx.stroke(rect, &brush, 1.0);
        }
    }
}

#[derive(Clone, Data, Lens)]
struct AppData {
    text1: String,
    text2: String,
}

fn row() -> impl Widget<AppData> {
    Flex::row()
        .with_child(TextBox::new().lens(AppData::text1))
        .with_default_spacer()
        .with_child(TextBox::new().lens(AppData::text2))
}

fn make_widget() -> impl Widget<AppData> {
    Flex::column()
        .with_child(row())
        .with_default_spacer()
        .with_child(FocusWrapper::new(row()))
        .with_default_spacer()
        .with_child(row())
        .with_default_spacer()
        .with_child(FocusWrapper::new(row()))
        .with_default_spacer()
        .align_horizontal(UnitPoint::CENTER)
}

fn main() {
    let window = WindowDesc::new(make_widget())
        .title("Focus Test");

    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(AppData {
            text1: String::new(),
            text2: String::new(),
        })
        .expect("launch failed");
}