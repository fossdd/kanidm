use askama::Template;
use axum::{
    extract::State,
    http::uri::Uri,
    response::{IntoResponse, Response},
    Extension,
};
use axum_htmx::extractors::HxRequest;
use axum_htmx::HxPushUrl;

use kanidm_proto::internal::AppLink;

use super::HtmlTemplate;
use crate::https::views::errors::HtmxError;
use crate::https::{extractors::VerifiedClientInformation, middleware::KOpId, ServerState};

#[derive(Template)]
#[template(path = "apps.html")]
struct AppsView {
    apps_partial: AppsPartialView,
}

#[derive(Template)]
#[template(path = "apps_partial.html")]
struct AppsPartialView {
    apps: Vec<AppLink>,
}

pub(crate) async fn view_apps_get(
    State(state): State<ServerState>,
    Extension(kopid): Extension<KOpId>,
    HxRequest(hx_request): HxRequest,
    VerifiedClientInformation(client_auth_info): VerifiedClientInformation,
) -> axum::response::Result<Response> {
    // Because this is the route where the login page can land, we need to actually alter
    // our response as a result. If the user comes here directly we need to render the full
    // page, otherwise we need to render the partial.

    let app_links = state
        .qe_r_ref
        .handle_list_applinks(client_auth_info, kopid.eventid)
        .await
        .map_err(|old| HtmxError::new(&kopid, old))?;

    let apps_partial = AppsPartialView { apps: app_links };

    Ok(if hx_request {
        (
            // On the redirect during a login we don't push urls. We set these headers
            // so that the url is updated, and we swap the correct element.
            HxPushUrl(Uri::from_static("/ui/apps")),
            HtmlTemplate(apps_partial),
        )
            .into_response()
    } else {
        let apps_view = AppsView { apps_partial };
        HtmlTemplate(apps_view).into_response()
    })
}
