// Copyright 2020 The xi-editor Authors.
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

//! Example of dynamic text styling

use druid::widget::prelude::*;
use druid::widget::{Checkbox, Flex, Label, MainAxisAlignment, Painter, Parse, Stepper, TextBox};
use druid::{
    theme, AppLauncher, Color, Data, Key, Lens, LensExt, LensWrap, LocalizedString, PlatformError,
    RenderContext, Widget, WidgetExt, WindowDesc,
};
use std::fmt::Display;

// This is a custom key we'll use with Env to set and get our text size.
const MY_CUSTOM_TEXT_SIZE: Key<f64> = Key::new("styled_text.custom_text_size");
const MY_CUSTOM_FONT: Key<&str> = Key::new("styled_text.custom_font");

#[derive(Clone, Lens, Data)]
struct AppData {
    text: String,
    size: f64,
    mono: bool,
}
impl Display for AppData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Size {:.1}{}: {}",
            self.size,
            if self.mono { " mono" } else { "" },
            self.text
        )
    }
}

pub fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder).title(
        LocalizedString::new("styled-text-demo-window-title").with_placeholder("Type Styler"),
    );
    let data = AppData {
        text: "Here's some sample text".to_string(),
        size: 24.0,
        mono: false,
    };

    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)?;

    Ok(())
}

fn ui_builder() -> impl Widget<AppData> {
    let my_painter = Painter::new(|ctx, _, _| {
        let bounds = ctx.size().to_rect();
        if ctx.is_hot() {
            ctx.fill(bounds, &Color::rgba8(0, 0, 0, 128));
        }

        if ctx.is_active() {
            ctx.stroke(bounds, &Color::WHITE, 2.0);
        }
    });

    // This is druid's default text style.
    // It's set by theme::LABEL_COLOR, theme::TEXT_SIZE_NORMAL, and theme::FONT_NAME
    let label =
        Label::new(|data: &String, _env: &_| format!("Default: {}", data)).lens(AppData::text);

    // The text_color, text_size, and font builder methods can override the
    // defaults provided by the theme by passing in a Key or a concrete value.
    //
    // In this example, text_color receives a Key from the theme, text_size
    // gets a custom key which we set with the env_scope wrapper, and the
    // default font key (theme::FONT_NAME) is overridden in the env_scope
    // wrapper. (Like text_color and text_size, the font can be set using the
    // with_font builder method, but overriding here makes it easy to fall back
    // to the default font)
    let styled_label = Label::new(|data: &AppData, _env: &_| format!("{}", data))
        .with_text_color(theme::PRIMARY_LIGHT)
        .with_text_size(MY_CUSTOM_TEXT_SIZE)
        .with_font(MY_CUSTOM_FONT)
        .background(my_painter)
        .on_click(|_, data, _| {
            data.size *= 1.1;
        })
        .env_scope(|env: &mut druid::Env, data: &AppData| {
            env.set(MY_CUSTOM_TEXT_SIZE, data.size);
            if data.mono {
                env.set(MY_CUSTOM_FONT, "monospace");
            } else {
                env.set(MY_CUSTOM_FONT, env.get(theme::FONT_NAME).to_string());
            }
        });

    let stepper = Stepper::new()
        .with_range(0.0, 100.0)
        .with_step(1.0)
        .with_wraparound(false)
        .lens(AppData::size);

    let stepper_textbox = LensWrap::new(
        Parse::new(TextBox::new()),
        AppData::size.map(|x| Some(*x), |x, y| *x = y.unwrap_or(24.0)),
    );

    let stepper_row = Flex::row().with_child(stepper_textbox).with_child(stepper);

    let mono_checkbox = Checkbox::new("Monospace").lens(AppData::mono);

    let input = TextBox::new().fix_width(200.0).lens(AppData::text);

    Flex::column()
        .main_axis_alignment(MainAxisAlignment::Center)
        .with_child(label)
        .with_spacer(8.0)
        .with_child(styled_label)
        .with_spacer(32.0)
        .with_child(stepper_row)
        .with_spacer(8.0)
        .with_child(mono_checkbox)
        .with_spacer(8.0)
        .with_child(input.padding(5.0))
}
