use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, JsonRender, Output, RenderContext,
    RenderError, Renderable, StringWriter,
};

///Custom helper that gives the first non null value of a ` || ` separated list
#[derive(Clone, Copy)]
pub struct FirstNonNullHelper;

impl HelperDef for FirstNonNullHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars,
        ctx: &Context,
        rc: &mut RenderContext<'reg>,
        out: &mut Output,
    ) -> HelperResult {
        let tpl = h
            .template()
            .ok_or(RenderError::new("no values in first helper"))?;

        let rendered_text = tpl.renders(r, ctx, rc).expect("pouuuuet");

        let value = rendered_text
            .split(" || ")
            .into_iter()
            .filter(|v| !v.is_empty())
            .next()
            .unwrap_or_else(|| "");

        out.write(&value)?;
        Ok(())
    }
}

pub fn new_template_engine() -> handlebars::Handlebars {
    let mut template_engine = handlebars::Handlebars::new();

    // we add our custom helper, 'first'
    template_engine.register_helper("first", Box::new(FirstNonNullHelper));
    template_engine
}
