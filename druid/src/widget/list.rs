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

//! Simple list view widget.

use std::cmp::Ordering;
use std::f64;
use std::sync::Arc;

#[cfg(feature = "im")]
use crate::im::Vector;
use crate::kurbo::{Point, Rect, Size};
use crate::widget::Axis;
use crate::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    UpdateCtx, Widget, WidgetPod,
};

/// A list widget for a variable-size collection of items.
pub struct List<T> {
    closure: Box<dyn Fn() -> Box<dyn Widget<T>>>,
    children: Vec<WidgetPod<T, Box<dyn Widget<T>>>>,
    axis: Axis,
    flex: bool,
}

impl<T: Data> List<T> {
    /// Create a new list widget. Closure will be called every time when a new child
    /// needs to be constructed.
    pub fn new<W: Widget<T> + 'static>(closure: impl Fn() -> W + 'static, axis: Axis) -> Self {
        List {
            closure: Box::new(move || Box::new(closure())),
            children: Vec::new(),
            axis,
            flex: false,
        }
    }

    /// Create a list where items are in a left -> right row
    pub fn horizontal<W: Widget<T> + 'static>(closure: impl Fn() -> W + 'static) -> Self {
        Self::new(closure, Axis::Horizontal)
    }

    /// Create a list where items are in a top -> bottom column
    pub fn vertical<W: Widget<T> + 'static>(closure: impl Fn() -> W + 'static) -> Self {
        Self::new(closure, Axis::Vertical)
    }

    /// If set to `true`, each element will be given an equal share of the space available.
    pub fn with_flex(mut self, flex: bool) -> Self {
        self.flex = flex;
        self
    }

    /// If set to `true`, each element will be given an equal share of the space available.
    pub fn set_flex(&mut self, flex: bool) -> &mut Self {
        self.flex = flex;
        self
    }

    /// When the widget is created or the data changes, create or remove children as needed
    ///
    /// Returns `true` if children were added or removed.
    fn update_child_count(&mut self, data: &impl ListIter<T>, _env: &Env) -> bool {
        let len = self.children.len();
        match len.cmp(&data.data_len()) {
            Ordering::Greater => self.children.truncate(data.data_len()),
            Ordering::Less => data.for_each(|_, i| {
                if i >= len {
                    let child = WidgetPod::new((self.closure)());
                    self.children.push(child);
                }
            }),
            Ordering::Equal => (),
        }
        len != data.data_len()
    }
}

/// This iterator enables writing List widget for any `Data`.
pub trait ListIter<T>: Data {
    /// Iterate over each data child.
    fn for_each(&self, cb: impl FnMut(&T, usize));

    /// Iterate over each data child. Keep track of changed data and update self.
    fn for_each_mut(&mut self, cb: impl FnMut(&mut T, usize));

    /// Return data length.
    fn data_len(&self) -> usize;
}

#[cfg(feature = "im")]
impl<T: Data> ListIter<T> for Vector<T> {
    fn for_each(&self, mut cb: impl FnMut(&T, usize)) {
        for (i, item) in self.iter().enumerate() {
            cb(item, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut T, usize)) {
        for (i, item) in self.iter_mut().enumerate() {
            cb(item, i);
        }
    }

    fn data_len(&self) -> usize {
        self.len()
    }
}

// S == shared data type
#[cfg(feature = "im")]
impl<S: Data, T: Data> ListIter<(S, T)> for (S, Vector<T>) {
    fn for_each(&self, mut cb: impl FnMut(&(S, T), usize)) {
        for (i, item) in self.1.iter().enumerate() {
            let d = (self.0.to_owned(), item.to_owned());
            cb(&d, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut (S, T), usize)) {
        for (i, item) in self.1.iter_mut().enumerate() {
            let mut d = (self.0.clone(), item.clone());
            cb(&mut d, i);

            if !self.0.same(&d.0) {
                self.0 = d.0;
            }
            if !item.same(&d.1) {
                *item = d.1;
            }
        }
    }

    fn data_len(&self) -> usize {
        self.1.len()
    }
}

impl<T: Data> ListIter<T> for Arc<Vec<T>> {
    fn for_each(&self, mut cb: impl FnMut(&T, usize)) {
        for (i, item) in self.iter().enumerate() {
            cb(item, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut T, usize)) {
        let mut new_data = Vec::with_capacity(self.data_len());
        let mut any_changed = false;

        for (i, item) in self.iter().enumerate() {
            let mut d = item.to_owned();
            cb(&mut d, i);

            if !any_changed && !item.same(&d) {
                any_changed = true;
            }
            new_data.push(d);
        }

        if any_changed {
            *self = Arc::new(new_data);
        }
    }

    fn data_len(&self) -> usize {
        self.len()
    }
}

// S == shared data type
impl<S: Data, T: Data> ListIter<(S, T)> for (S, Arc<Vec<T>>) {
    fn for_each(&self, mut cb: impl FnMut(&(S, T), usize)) {
        for (i, item) in self.1.iter().enumerate() {
            let d = (self.0.clone(), item.to_owned());
            cb(&d, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut (S, T), usize)) {
        let mut new_data = Vec::with_capacity(self.1.len());
        let mut any_shared_changed = false;
        let mut any_el_changed = false;

        for (i, item) in self.1.iter().enumerate() {
            let mut d = (self.0.clone(), item.to_owned());
            cb(&mut d, i);

            if !any_shared_changed && !self.0.same(&d.0) {
                any_shared_changed = true;
            }
            if any_shared_changed {
                self.0 = d.0;
            }
            if !any_el_changed && !item.same(&d.1) {
                any_el_changed = true;
            }
            new_data.push(d.1);
        }

        if any_el_changed {
            self.1 = Arc::new(new_data);
        }
    }

    fn data_len(&self) -> usize {
        self.1.len()
    }
}

impl<C: Data, T: ListIter<C>> Widget<T> for List<C> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        let mut children = self.children.iter_mut();
        data.for_each_mut(|child_data, _| {
            if let Some(child) = children.next() {
                child.event(ctx, event, child_data, env);
            }
        });
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            if self.update_child_count(data, env) {
                ctx.children_changed();
            }
        }

        let mut children = self.children.iter_mut();
        data.for_each(|child_data, _| {
            if let Some(child) = children.next() {
                child.lifecycle(ctx, event, child_data, env);
            }
        });
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        // we send update to children first, before adding or removing children;
        // this way we avoid sending update to newly added children, at the cost
        // of potentially updating children that are going to be removed.
        let mut children = self.children.iter_mut();
        data.for_each(|child_data, _| {
            if let Some(child) = children.next() {
                child.update(ctx, child_data, env);
            }
        });

        if self.update_child_count(data, env) {
            ctx.children_changed();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let axis = self.axis; // keep the borrow checker happy
        let mut minor = axis.minor(bc.min());
        let mut major = 0.0;

        let mut paint_rect = Rect::ZERO;
        let mut children = self.children.iter_mut();

        // TODO quantize to whole pixels
        // major constraint
        let mc = if self.flex {
            let div = axis.major(bc.max()) / (data.data_len() as f64);
            (div, div)
        } else {
            (0., f64::INFINITY)
        };

        // keep track of children's sizes for warning the user.
        data.for_each(|child_data, _| {
            let child = match children.next() {
                Some(child) => child,
                None => {
                    return;
                }
            };
            let child_bc = match axis {
                Axis::Horizontal => BoxConstraints::new(
                    Size::new(mc.0, bc.min().height),
                    Size::new(mc.1, bc.max().height),
                ),
                Axis::Vertical => BoxConstraints::new(
                    Size::new(bc.min().width, mc.0),
                    Size::new(bc.max().width, mc.1),
                ),
            };
            let child_size = child.layout(ctx, &child_bc, child_data, env);
            let rect = Rect::from_origin_size(Point::from(axis.pack(major, 0.0)), child_size);
            child.set_layout_rect(ctx, child_data, env, rect);
            paint_rect = paint_rect.union(child.paint_rect());
            minor = minor.max(axis.minor(child_size));
            major += axis.major(child_size);
        });

        // TODO I don't understand this logic. If we end up constraining here then our layout for
        // the child elements is broken, no?
        let my_size = bc.constrain(Size::from(axis.pack(major, minor)));
        let insets = paint_rect - Rect::ZERO.with_size(my_size);
        ctx.set_paint_insets(insets);
        my_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let mut children = self.children.iter_mut();
        data.for_each(|child_data, _| {
            if let Some(child) = children.next() {
                child.paint(ctx, child_data, env);
            }
        });
    }
}
