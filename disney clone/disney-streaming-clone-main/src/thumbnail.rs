use smallvec::{smallvec, SmallVec};
use tracing::{trace_span, Span};

use widget_cruncher::widget::prelude::*;
use widget_cruncher::widget::{AsWidgetPod, WebImage, WidgetPod};
use widget_cruncher::{Color, Selector};

pub const CHANGE_SELECTED_ITEM: Selector<(usize, usize)> = Selector::new("change_selected_item");
pub const THUMBNAIL_MAX_SIZE: f64 = 200.0;

pub struct Thumbnail {
    // We store which row and column this is in, to handle arrow selection "manually"
    pub row: usize,
    pub column: usize,

    // An image loaded from a URL, with a spinner placeholder
    pub inner: WidgetPod<WebImage>,

    // Animation state for the "selected" animation
    pub selected: bool,
    pub selected_progress: u32,
}

impl Thumbnail {
    pub fn new(row: usize, column: usize, thumbnail_url: String) -> Self {
        let image = WebImage::new(thumbnail_url);
        Self {
            row,
            column,
            inner: WidgetPod::new(image),
            selected: false,
            selected_progress: 0,
        }
    }
}

// --- TRAIT IMPL ---

impl Widget for Thumbnail {
    fn on_event(&mut self, ctx: &mut EventCtx, event: &Event, env: &Env) {
        ctx.init();
        match event {
            Event::Command(command) => {
                if let Some((row, col)) = command.try_get(CHANGE_SELECTED_ITEM) {
                    if (*row, *col) == (self.row, self.column) {
                        self.selected = true;
                        ctx.request_anim_frame();
                        ctx.request_layout();
                        ctx.request_pan_to_this();
                    } else if self.selected {
                        self.selected = false;
                        ctx.request_anim_frame();
                        ctx.request_layout();
                    }
                }
            }
            // TODO - handle frame interval?
            Event::AnimFrame(_interval) => {
                if self.selected {
                    if self.selected_progress < 5 {
                        self.selected_progress += 1;
                        ctx.request_anim_frame();
                        ctx.request_layout();
                    }
                } else {
                    if self.selected_progress > 0 {
                        self.selected_progress -= 1;
                        ctx.request_anim_frame();
                        ctx.request_layout();
                    }
                }
            }
            _ => {}
        }
        self.inner.on_event(ctx, event, env)
    }

    fn on_status_change(&mut self, _ctx: &mut LifeCycleCtx, _event: &StatusChange, _env: &Env) {}

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, env: &Env) {
        self.inner.lifecycle(ctx, event, env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, _bc: &BoxConstraints, env: &Env) -> Size {
        // We essentially do a linear interpolation
        // between "90% of max size" and "max size"
        let square_side = THUMBNAIL_MAX_SIZE * (0.90 + (self.selected_progress as f64) / 50.0);
        let child_constraints = BoxConstraints::new(
            Size::new(square_side, square_side),
            Size::new(square_side, square_side),
        );

        let outer_size = Size::new(THUMBNAIL_MAX_SIZE, THUMBNAIL_MAX_SIZE);
        let image_size = self.inner.layout(ctx, &child_constraints, env);
        let origin = (outer_size - image_size) / 2.0;
        self.inner.set_origin(ctx, env, origin.to_vec2().to_point());
        outer_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, env: &Env) {
        self.inner.paint(ctx, env);

        if self.selected {
            let border_width = 4.0;
            let border_color = Color::WHITE;
            let border_rect = self.inner.layout_rect();
            ctx.stroke(border_rect, &border_color, border_width);
        }
    }

    fn children(&self) -> SmallVec<[&dyn AsWidgetPod; 16]> {
        smallvec![&self.inner as &dyn AsWidgetPod]
    }

    fn children_mut(&mut self) -> SmallVec<[&mut dyn AsWidgetPod; 16]> {
        smallvec![&mut self.inner as &mut dyn AsWidgetPod]
    }

    // This isn't useful for the application itself, but it makes traces more readable
    // when debugging
    fn make_trace_span(&self) -> Span {
        trace_span!("Thumbnail")
    }
}
