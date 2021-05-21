// Copyright 2019 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This is an example of arbitrary transform of widgets.

use druid::widget::{AARotation, Flex, Radio, ViewSwitcher, TransformBox, FreeAffine, AATransform, BoundedRotation, Slider, CenterRotation};
use druid::{RenderContext, Env, WidgetExt, Widget, WindowDesc, AppLauncher, Data, Lens, Point, EventCtx, LifeCycle, PaintCtx, BoxConstraints, LifeCycleCtx, Size, LayoutCtx, Event, UpdateCtx};
use piet_common::Color;
use druid::kurbo::Circle;

#[derive(Clone, Data, Lens)]
struct TransformState {
    rotation: AARotation,
    rotation2: f64,
}

fn rotated_widget(data: &AARotation, _: &TransformState, _: &Env) -> Box<dyn Widget<TransformState>> {
    TransformBox::with_transform(
        MousePainter(None),
        AATransform::default()
            .rotated(*data)
    ).boxed()
}

struct MousePainter(Option<Point>);

impl<T: Data> Widget<T> for MousePainter {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut T, _env: &Env) {
        if let Event::MouseMove(me) = event {
            if ctx.is_hot() {
                self.0 = Some(me.pos);
            } else {
                self.0 = None;
            }
            ctx.request_paint();
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, _env: &Env) -> Size {
        bc.constrain((250.0, 80.0))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, _env: &Env) {
        let back = ctx.size().to_rounded_rect(5.0);

        ctx.fill(back, &Color::BLACK);
        if let Some(center) = self.0 {
            ctx.stroke(Circle::new(center, 4.0), &Color::RED, 2.0);
        }
    }
}

fn build_root_widget() -> impl Widget<TransformState> {
    let settings = Flex::column()
        .with_child(Radio::new("0°", AARotation::ORIGIN).lens(TransformState::rotation))
        .with_default_spacer()
        .with_child(Radio::new("90°", AARotation::CLOCKWISE).lens(TransformState::rotation))
        .with_default_spacer()
        .with_child(Radio::new("180°", AARotation::HALF_WAY).lens(TransformState::rotation))
        .with_default_spacer()
        .with_child(Radio::new("270°", AARotation::COUNTER_CLOCKWISE).lens(TransformState::rotation))
        .with_default_spacer()
        .with_child(Slider::new().with_range(0.0, 180.0).lens(TransformState::rotation2));

    Flex::row()
        .with_child(settings)
        .with_default_spacer()
        .with_child(ViewSwitcher::<TransformState, _>::new(
            |data: &TransformState, _|data.rotation,
            rotated_widget
            ))
        .with_default_spacer()
        .with_child(TransformBox::with_extractor(
            MousePainter(None),
            |data: &TransformState|BoundedRotation::degree(data.rotation2),
        ))
        .debug_paint_layout()
        .debug_widget_id()
}

pub fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget())
        .title("Transform example!")
        .window_size((400.0, 400.0));

    // create the initial app state
    let initial_state: TransformState = TransformState {
        rotation: AARotation::ORIGIN,
        rotation2: 0.0,
    };

    // start the application. Here we pass in the application state.
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}
