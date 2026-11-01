//! SOAP 1.1 executeCommand, compatible with the AC/TC RPC envelope.
//! Authentication uses existing SRP accounts; commands run in the world console queue.
use std::{net::SocketAddr, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};
use anyhow::{ensure, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use http_body_util::{BodyExt, Full, Limited};
use hyper::{body::{Bytes, Incoming}, header, Request, Response, StatusCode};
use hyper_util::rt::{TokioIo, TokioTimer};
use rz_common::Config;
use rz_database::{auth, srp6};
use tokio::{net::TcpListener, sync::{mpsc, oneshot, Semaphore}, task::JoinSet};
use crate::{commands::ConsoleResult, Ctx};

const ENVELOPE: &str = "http://schemas.xmlsoap.org/soap/envelope/";
const MAX_BODY: usize = 16 * 1024;
const MAX_COMMAND: usize = 4096;
const DEADLINE: Duration = Duration::from_secs(15);
type Reply = Response<Full<Bytes>>;

pub struct ConsoleRequest {
    pub command: String,
    pub reply: oneshot::Sender<ConsoleResult>,
}

pub async fn bind(conf: &Config) -> Result<Option<TcpListener>> {
    if !conf.bool("SOAP.Enabled", false) { return Ok(None); }
    let ip: std::net::IpAddr = conf.string("SOAP.IP", "127.0.0.1").parse().context("SOAP.IP invalide")?;
    let port: u16 = conf.string("SOAP.Port", "7878").parse().context("SOAP.Port invalide")?;
    ensure!(port != 0, "SOAP.Port doit être positif");
    let address = SocketAddr::new(ip, port);
    let listener = TcpListener::bind(address).await.with_context(|| format!("écoute SOAP sur {address}"))?;
    tracing::info!("SOAP prêt sur http://{address} (compte administrateur requis)");
    Ok(Some(listener))
}

pub async fn serve(listener: TcpListener, ctx: Arc<Ctx>, queue: mpsc::Sender<ConsoleRequest>) {
    let slots = Arc::new(Semaphore::new(8));
    let mut tasks = JoinSet::new();
    let mut stopping = ctx.shutdown.subscribe();
    loop {
        tokio::select! {
            _ = stopping.changed() => break,
            Some(_) = tasks.join_next(), if !tasks.is_empty() => {},
            accepted = listener.accept() => {
                let Ok((stream, peer)) = accepted else { break; };
                let Ok(permit) = slots.clone().try_acquire_owned() else { continue; };
                let ctx = ctx.clone(); let queue = queue.clone();
                tasks.spawn(async move {
                    let _permit = permit;
                    let shutdown = Arc::new(AtomicBool::new(false));
                    let flag = shutdown.clone(); let service_ctx = ctx.clone();
                    let service = hyper::service::service_fn(move |request| {
                        let ctx = service_ctx.clone(); let queue = queue.clone(); let flag = flag.clone();
                        async move { Ok::<_, std::convert::Infallible>(handle(request, peer, ctx, queue, flag).await) }
                    });
                    let mut http = hyper::server::conn::http1::Builder::new();
                    http.keep_alive(false).max_headers(32).max_buf_size(16 * 1024)
                        .timer(TokioTimer::new()).header_read_timeout(Duration::from_secs(5));
                    // A timed-out caller does not cancel an already queued world command.
                    let _ = tokio::time::timeout(DEADLINE, http.serve_connection(TokioIo::new(stream), service)).await;
                    // Flush the shutdown response before closing the world/listener.
                    if shutdown.load(Ordering::SeqCst) { let _ = ctx.shutdown.send(true); }
                });
            }
        }
    }
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
}

fn credentials(value: &str) -> Option<(String, String)> {
    let (scheme, data) = value.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("Basic") || data.len() > 256 { return None; }
    let decoded = STANDARD.decode(data).ok()?;
    let text = std::str::from_utf8(&decoded).ok()?;
    let (user, password) = text.split_once(':')?;
    if user.is_empty() || user.chars().count() > 32 || password.chars().count() > 16
        || text.chars().any(char::is_control) { return None; }
    Some((user.into(), password.into()))
}

async fn authorize(ctx: &Ctx, user: &str, password: &str, peer: SocketAddr) -> Result<StatusCode, anyhow::Error> {
    let Some(account) = auth::find_account(&ctx.login, user).await? else { return Ok(StatusCode::UNAUTHORIZED); };
    if !srp6::check_password(&account.username, password, &account.salt, &account.verifier) { return Ok(StatusCode::UNAUTHORIZED); }
    let ip = peer.ip().to_string();
    if (account.locked && account.last_ip != ip) || auth::is_banned(&ctx.login, account.id, &ip).await?
        || auth::gm_level(&ctx.login, account.id, ctx.settings.realm_id as i32).await? < 3 {
        return Ok(StatusCode::FORBIDDEN);
    }
    // Never log the command arguments: account commands can contain passwords.
    tracing::info!(account_id = account.id, %peer, "SOAP administrateur authentifié");
    Ok(StatusCode::OK)
}

fn parse_command(xml: &str) -> Result<(String, String)> {
    ensure!(xml.len() <= MAX_BODY, "Requête SOAP trop volumineuse");
    // roxmltree rejects DTDs/entities by default; no external resource resolution.
    let doc = roxmltree::Document::parse(xml).context("XML SOAP invalide")?;
    let root = doc.root_element();
    ensure!(root.has_tag_name((ENVELOPE, "Envelope")), "SOAP 1.1 Envelope requis");
    let children: Vec<_> = root.children().filter(|n| n.is_element()).collect();
    ensure!(children.len() == 1 || (children.len() == 2 && children[0].has_tag_name((ENVELOPE, "Header"))
        && children[0].children().all(|n| !n.is_element() && n.text().unwrap_or("").trim().is_empty())), "En-tête SOAP non pris en charge");
    let body = *children.last().context("SOAP Body absent")?;
    ensure!(body.has_tag_name((ENVELOPE, "Body")), "SOAP Body requis");
    let operations: Vec<_> = body.children().filter(|n| n.is_element()).collect();
    ensure!(operations.len() == 1, "Une commande SOAP par requête");
    let operation = operations[0];
    let namespace = operation.tag_name().namespace().unwrap_or("");
    ensure!(operation.tag_name().name() == "executeCommand" && matches!(namespace, "urn:AC" | "urn:TC"), "Méthode SOAP inconnue");
    let args: Vec<_> = operation.children().filter(|n| n.is_element()).collect();
    ensure!(args.len() == 1 && args[0].tag_name().name() == "command"
        && matches!(args[0].tag_name().namespace(), None | Some("urn:AC" | "urn:TC"))
        && !args[0].children().any(|n| n.is_element()), "Argument command requis");
    let text: String = args[0].children().filter(|n| n.is_text()).filter_map(|n| n.text()).collect();
    ensure!(!text.chars().any(char::is_control), "Commande sur une seule ligne requise");
    let command = text.trim().strip_prefix('.').unwrap_or(text.trim()).trim();
    ensure!(!command.is_empty() && command.len() <= MAX_COMMAND, "Commande vide ou trop longue");
    Ok((namespace.into(), command.into()))
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}
fn response(status: StatusCode, body: String) -> Reply {
    let mut reply = Response::new(Full::new(Bytes::from(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><SOAP-ENV:Envelope xmlns:SOAP-ENV=\"{ENVELOPE}\"><SOAP-ENV:Body>{body}</SOAP-ENV:Body></SOAP-ENV:Envelope>"))));
    *reply.status_mut() = status;
    reply.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/xml; charset=utf-8"));
    reply.headers_mut().insert(header::CONNECTION, header::HeaderValue::from_static("close"));
    reply.headers_mut().insert(header::CACHE_CONTROL, header::HeaderValue::from_static("no-store"));
    if status == StatusCode::UNAUTHORIZED { reply.headers_mut().insert(header::WWW_AUTHENTICATE, header::HeaderValue::from_static("Basic realm=\"RappelzCore SOAP\"")); }
    reply
}
fn fault(status: StatusCode, message: &str) -> Reply {
    let code = if status == StatusCode::SERVICE_UNAVAILABLE { "Server" } else { "Client" };
    response(status, format!("<SOAP-ENV:Fault><faultcode>SOAP-ENV:{code}</faultcode><faultstring>{}</faultstring></SOAP-ENV:Fault>", escape(message)))
}

async fn handle(request: Request<Incoming>, peer: SocketAddr, ctx: Arc<Ctx>, queue: mpsc::Sender<ConsoleRequest>, shutdown: Arc<AtomicBool>) -> Reply {
    if request.method() != hyper::Method::POST {
        let mut reply = fault(StatusCode::METHOD_NOT_ALLOWED, "Utiliser HTTP POST");
        reply.headers_mut().insert(header::ALLOW, header::HeaderValue::from_static("POST")); return reply;
    }
    let values: Vec<_> = request.headers().get_all(header::AUTHORIZATION).iter().collect();
    let Some((user, password)) = (if values.len() == 1 { values[0].to_str().ok().and_then(credentials) } else { None }) else {
        return fault(StatusCode::UNAUTHORIZED, "Authentification requise");
    };
    match authorize(&ctx, &user, &password, peer).await {
        Ok(StatusCode::OK) => {},
        Ok(status) => return fault(status, "Accès SOAP refusé"),
        Err(_) => return fault(StatusCode::SERVICE_UNAVAILABLE, "Authentification temporairement indisponible"),
    }
    let bytes = match Limited::new(request.into_body(), MAX_BODY).collect().await {
        Ok(body) => body.to_bytes(),
        Err(_) => return fault(StatusCode::PAYLOAD_TOO_LARGE, "Corps HTTP invalide ou trop volumineux"),
    };
    let parsed = std::str::from_utf8(&bytes).ok().and_then(|xml| parse_command(xml).ok());
    let Some((namespace, command)) = parsed else { return fault(StatusCode::INTERNAL_SERVER_ERROR, "Requête executeCommand SOAP 1.1 invalide"); };
    let (reply, receiver) = oneshot::channel();
    if queue.try_send(ConsoleRequest { command, reply }).is_err() { return fault(StatusCode::SERVICE_UNAVAILABLE, "Console occupée ou arrêtée"); }
    match receiver.await {
        Ok(result) if result.success => {
            shutdown.store(result.shutdown, Ordering::SeqCst);
            response(StatusCode::OK, format!("<ns1:executeCommandResponse xmlns:ns1=\"{namespace}\"><result>{}</result></ns1:executeCommandResponse>", escape(&result.output)))
        },
        Ok(result) => fault(StatusCode::INTERNAL_SERVER_ERROR, &result.output),
        Err(_) => fault(StatusCode::SERVICE_UNAVAILABLE, "Console arrêtée"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn envelope(ns: &str, command: &str) -> String { format!("<s:Envelope xmlns:s=\"{ENVELOPE}\" xmlns:m=\"{ns}\"><s:Body><m:executeCommand><command>{command}</command></m:executeCommand></s:Body></s:Envelope>") }
    #[test] fn accepts_ac_tc_prefixes_cdata_entities_and_optional_dot() {
        for ns in ["urn:AC", "urn:TC"] {
            assert_eq!(parse_command(&envelope(ns, " .server info ")).unwrap(), (ns.into(), "server info".into()));
            assert_eq!(parse_command(&envelope(ns, "announce &lt;A&gt;&amp;<![CDATA[B]]>")).unwrap().1, "announce <A>&B");
        }
    }
    #[test] fn rejects_wrong_namespace_methods_duplicate_nested_or_empty_commands_and_dtd() {
        let valid = envelope("urn:AC", "server info");
        for bad in [envelope("urn:wrong", "server info"), valid.replace("executeCommand", "other"),
            valid.replace("</s:Body>", "<m:executeCommand><command>exit</command></m:executeCommand></s:Body>"),
            envelope("urn:AC", "<nested>exit</nested>"), envelope("urn:AC", ""), envelope("urn:AC", "help&#10;exit"),
            format!("<!DOCTYPE s:Envelope [<!ENTITY a 'exit'>]>{}", envelope("urn:AC", "&a;")),
            valid.replace(ENVELOPE, "http://www.w3.org/2003/05/soap-envelope")] { assert!(parse_command(&bad).is_err(), "{bad}"); }
        assert!(parse_command(&envelope("urn:AC", &"a".repeat(MAX_COMMAND + 1))).is_err());
        assert!(parse_command(&"a".repeat(MAX_BODY + 1)).is_err());
    }
    #[test] fn basic_credentials_are_bounded_and_reject_malformed_or_duplicate_schemes() {
        assert_eq!(credentials(&format!("bAsIc {}", STANDARD.encode("Admin:pass:word"))), Some(("Admin".into(), "pass:word".into())));
        for value in ["Basic invalid", "Bearer token", "Basic ", "Basic Basic aaa"] { assert!(credentials(value).is_none()); }
        for value in [":pass", "admin\n:pass", "admin:passwordtoolong123456"] { assert!(credentials(&format!("Basic {}", STANDARD.encode(value))).is_none()); }
    }
    #[tokio::test] async fn fault_xml_and_http_auth_challenge_preserve_special_characters() {
        let response = fault(StatusCode::UNAUTHORIZED, "<Échec>&\"'");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(response.headers().contains_key(header::WWW_AUTHENTICATE));
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let doc = roxmltree::Document::parse(std::str::from_utf8(&bytes).unwrap()).unwrap();
        assert_eq!(doc.descendants().find(|n| n.has_tag_name("faultstring")).unwrap().text(), Some("<Échec>&\"'"));
    }
    #[tokio::test] async fn disabled_soap_never_opens_a_listener_and_bad_port_fails_closed() {
        assert!(bind(&Config::parse("SOAP.Enabled = 0\nSOAP.Port = invalid")).await.unwrap().is_none());
        for text in ["SOAP.Port = invalid", "SOAP.Port = 0", "SOAP.IP = invalid"] {
            assert!(bind(&Config::parse(&format!("SOAP.Enabled = 1\n{text}"))).await.is_err());
        }
    }
}
