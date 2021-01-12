/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::{ref_filter_map, DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::OffscreenCanvasBinding::{
    OffscreenCanvasMethods, OffscreenRenderingContext,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::offscreencanvasrenderingcontext2d::OffscreenCanvasRenderingContext2D;
use crate::script_runtime::JSContext;
use canvas::canvas_session;
use async_std::task;
use canvas::canvas_session::*;
use dom_struct::dom_struct;
use euclid::default::Size2D;
use ferrite_session::*;
use ipc_channel::ipc::IpcSharedMemory;
use js::rust::HandleValue;
use std::cell::Cell;

#[unrooted_must_root_lint::must_root]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub enum OffscreenCanvasContext {
    OffscreenContext2d(Dom<OffscreenCanvasRenderingContext2D>),
    //WebGL(Dom<WebGLRenderingContext>),
    //WebGL2(Dom<WebGL2RenderingContext>),
}

#[dom_struct]
pub struct OffscreenCanvas {
    eventtarget: EventTarget,
    width: Cell<u64>,
    height: Cell<u64>,
    context: DomRefCell<Option<OffscreenCanvasContext>>,
    placeholder: Option<Dom<HTMLCanvasElement>>,
}

impl OffscreenCanvas {
    pub fn new_inherited(
        width: u64,
        height: u64,
        placeholder: Option<&HTMLCanvasElement>,
    ) -> OffscreenCanvas {
        OffscreenCanvas {
            eventtarget: EventTarget::new_inherited(),
            width: Cell::new(width),
            height: Cell::new(height),
            context: DomRefCell::new(None),
            placeholder: placeholder.map(Dom::from_ref),
        }
    }

    pub fn new(
        global: &GlobalScope,
        width: u64,
        height: u64,
        placeholder: Option<&HTMLCanvasElement>,
    ) -> DomRoot<OffscreenCanvas> {
        reflect_dom_object(
            Box::new(OffscreenCanvas::new_inherited(width, height, placeholder)),
            global,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        width: u64,
        height: u64,
    ) -> Fallible<DomRoot<OffscreenCanvas>> {
        let offscreencanvas = OffscreenCanvas::new(global, width, height, None);
        Ok(offscreencanvas)
    }

    pub fn get_size(&self) -> Size2D<u64> {
        Size2D::new(self.Width(), self.Height())
    }

    pub fn origin_is_clean(&self) -> bool {
        match *self.context.borrow() {
            Some(OffscreenCanvasContext::OffscreenContext2d(ref context)) => {
                context.origin_is_clean()
            },
            _ => true,
        }
    }

    pub fn context(&self) -> Option<Ref<OffscreenCanvasContext>> {
        ref_filter_map(self.context.borrow(), |ctx| ctx.as_ref())
    }

    pub fn fetch_all_data(&self) -> Option<(Option<IpcSharedMemory>, Size2D<u32>)> {
        let size = self.get_size();

        if size.width == 0 || size.height == 0 {
            return None;
        }

        let data = match self.context.borrow().as_ref() {
            Some(&OffscreenCanvasContext::OffscreenContext2d(ref context)) => {
                let session = context.get_canvas_session().clone();
                let data = task::block_on(async move {
                    debug!("acquiring shared session");
                    let res = canvas_session::enqueue_task(move || async move {
                        run_session_with_result(
                            acquire_shared_session!(session, chan =>
                                    choose!(chan, FromScript,
                                        receive_value_from!(chan, data =>
                                            release_shared_session(chan,
                                                send_value(data,
                                                    terminate()))))),
                        )
                        .await
                    }).await.await;
                    debug!("released shared session");
                    res
                });

                Some(IpcSharedMemory::from_bytes(&data))
            },
            None => None,
        };

        Some((data, size.to_u32()))
    }

    #[allow(unsafe_code)]
    fn get_or_init_2d_context(&self) -> Option<DomRoot<OffscreenCanvasRenderingContext2D>> {
        if let Some(ctx) = self.context() {
            return match *ctx {
                OffscreenCanvasContext::OffscreenContext2d(ref ctx) => Some(DomRoot::from_ref(ctx)),
            };
        }
        let context = OffscreenCanvasRenderingContext2D::new(
            &self.global(),
            self,
            self.placeholder.as_ref().map(|c| &**c),
        );
        *self.context.borrow_mut() = Some(OffscreenCanvasContext::OffscreenContext2d(
            Dom::from_ref(&*context),
        ));
        Some(context)
    }

    pub fn is_valid(&self) -> bool {
        self.Width() != 0 && self.Height() != 0
    }
}

impl OffscreenCanvasMethods for OffscreenCanvas {
    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-getcontext
    fn GetContext(
        &self,
        _cx: JSContext,
        id: DOMString,
        _options: HandleValue,
    ) -> Option<OffscreenRenderingContext> {
        match &*id {
            "2d" => self
                .get_or_init_2d_context()
                .map(OffscreenRenderingContext::OffscreenCanvasRenderingContext2D),
            /*"webgl" | "experimental-webgl" => self
                .get_or_init_webgl_context(cx, options)
                .map(OffscreenRenderingContext::WebGLRenderingContext),
            "webgl2" | "experimental-webgl2" => self
                .get_or_init_webgl2_context(cx, options)
                .map(OffscreenRenderingContext::WebGL2RenderingContext),*/
            _ => None,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-width
    fn Width(&self) -> u64 {
        return self.width.get();
    }

    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-width
    fn SetWidth(&self, value: u64) {
        self.width.set(value);

        if let Some(canvas_context) = self.context() {
            match &*canvas_context {
                OffscreenCanvasContext::OffscreenContext2d(rendering_context) => {
                    rendering_context.set_canvas_bitmap_dimensions(self.get_size());
                },
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-height
    fn Height(&self) -> u64 {
        return self.height.get();
    }

    // https://html.spec.whatwg.org/multipage/#dom-offscreencanvas-height
    fn SetHeight(&self, value: u64) {
        self.height.set(value);

        if let Some(canvas_context) = self.context() {
            match &*canvas_context {
                OffscreenCanvasContext::OffscreenContext2d(rendering_context) => {
                    rendering_context.set_canvas_bitmap_dimensions(self.get_size());
                },
            }
        }
    }
}
