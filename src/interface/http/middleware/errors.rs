use salvo::prelude::*;

#[handler]
pub async fn error_404(_req: &mut Request, res: &mut Response, ctrl: &mut FlowCtrl) {
    if res.status_code.is_none() {
        res.status_code(StatusCode::NOT_FOUND);
        res.render(Text::Plain("404: Not Found"));
        ctrl.skip_rest();
    }
}
