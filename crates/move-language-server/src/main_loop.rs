use std::error::Error;
use std::fmt;

use anyhow::bail;
use anyhow::Result;
use crossbeam_channel::{unbounded, Sender};
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeConfiguration, DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument,
    DidOpenTextDocument, PublishDiagnostics, ShowMessage,
};
use lsp_types::request::WorkspaceConfiguration;
use lsp_types::{
    ConfigurationItem, ConfigurationParams, Diagnostic, MessageType, PublishDiagnosticsParams,
    ShowMessageParams, Url,
};

use serde::de::DeserializeOwned;
use serde::Serialize;
use threadpool::ThreadPool;

use analysis::analysis::Analysis;
use analysis::config::Config;

use crate::dispatcher::PoolDispatcher;
use crate::global_state::{initialize_new_global_state, GlobalState};
use crate::handlers;
use crate::req;
use crate::subscriptions::OpenedFiles;
use analysis::db::FileDiagnostic;
use std::collections::HashSet;
use utils::{leaked_fpath, MoveFile, MoveFilePath};

#[derive(Debug)]
pub struct LspError {
    pub code: i32,
    pub message: String,
}

impl LspError {
    pub const UNKNOWN_FILE: i32 = -32900;

    pub fn new(code: i32, message: String) -> LspError {
        LspError { code, message }
    }
}

impl fmt::Display for LspError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Language Server request failed with {}. ({})",
            self.code, self.message
        )
    }
}

impl Error for LspError {}

#[derive(Debug)]
pub enum ResponseEvent {
    Respond(Response),
    Diagnostic(Vec<FileDiagnostic>),
}

#[derive(Debug)]
pub enum FileSystemEvent {
    AddFile(MoveFile),
    RemoveFile(MoveFilePath),
    ChangeFile(MoveFile),
}

pub enum Event {
    Response(ResponseEvent),
    FileSystem(FileSystemEvent),
    Lsp(Message),
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Event::Lsp(Message::Notification(not)) = self {
            if notification_is::<DidOpenTextDocument>(not)
                || notification_is::<DidChangeTextDocument>(not)
            {
                let document_uri = not.params.pointer("/textDocument/uri").unwrap().to_string();
                return f
                    .debug_struct("Notification")
                    .field("method", &not.method)
                    .field("file", &document_uri)
                    .finish();
            }
        }
        match self {
            Event::Lsp(it) => fmt::Debug::fmt(it, f),
            Event::FileSystem(it) => fmt::Debug::fmt(it, f),
            Event::Response(it) => fmt::Debug::fmt(it, f),
        }
    }
}

pub fn main_loop(global_state: &mut GlobalState, connection: &Connection) -> Result<()> {
    log::info!("starting example main loop");

    let pool = ThreadPool::new(1);
    let (resp_events_sender, resp_events_receiver) = unbounded::<ResponseEvent>();
    let (fs_events_sender, fs_events_receiver) = unbounded::<FileSystemEvent>();

    let mut loop_state = LoopState::default();

    log::info!("server initialized, serving requests");
    loop {
        let event = crossbeam_channel::select! {
            recv(&connection.receiver) -> message => match message {
                Ok(message) => Event::Lsp(message),
                Err(_) => bail!("client exited without shutdown"),
            },
            recv(&resp_events_receiver) -> event => Event::Response(event.unwrap()),
            recv(fs_events_receiver) -> fs_event => Event::FileSystem(fs_event.unwrap()),
        };
        if let Event::Lsp(Message::Request(req)) = &event {
            if connection.handle_shutdown(&req)? {
                break;
            }
        }
        loop_turn(
            &pool,
            &resp_events_sender,
            &fs_events_sender,
            &connection,
            global_state,
            &mut loop_state,
            event,
        )?;
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct LoopState {
    next_request_id: u64,
    opened_files: OpenedFiles,
    configuration_request_id: Option<RequestId>,
}

impl LoopState {
    fn next_request_id(&mut self) -> RequestId {
        self.next_request_id += 1;
        self.next_request_id.into()
    }

    pub fn with_config_request_id(req_id: &RequestId) -> Self {
        LoopState {
            configuration_request_id: Some(req_id.to_owned()),
            ..LoopState::default()
        }
    }
}

pub fn loop_turn(
    pool: &ThreadPool,
    resp_events_sender: &Sender<ResponseEvent>,
    fs_events_sender: &Sender<FileSystemEvent>,
    connection: &Connection,
    global_state: &mut GlobalState,
    loop_state: &mut LoopState,
    event: Event,
) -> Result<()> {
    if matches!(event, Event::Lsp(_)) {
        log::info!("loop turn = {:#?}", &event);
    }
    let fs_changed = match event {
        Event::Response(task) => {
            on_task(task, &connection.sender);
            false
        }
        Event::FileSystem(fs_event) => {
            global_state.update_from_events(vec![fs_event]);
            true
        }
        Event::Lsp(message) => {
            match message {
                Message::Request(req) => {
                    on_request(
                        global_state,
                        pool,
                        resp_events_sender,
                        &connection.sender,
                        req,
                    )?;
                }
                Message::Notification(not) => {
                    on_notification(&connection.sender, fs_events_sender, loop_state, not)?;
                }
                Message::Response(resp) => {
                    if Some(&resp.id) == loop_state.configuration_request_id.as_ref() {
                        loop_state.configuration_request_id = None;
                        log::info!("config update response: '{:?}", resp);
                        let Response { error, result, .. } = resp;

                        match (error, result) {
                            (Some(err), _) => {
                                log::error!("failed to fetch the server settings: {:?}", err)
                            }
                            (None, Some(configs)) => {
                                if let Some(new_config) = configs.get(0) {
                                    let mut config = Config::default();
                                    config.update(new_config);
                                    *global_state = initialize_new_global_state(config);
                                }
                            }
                            (None, None) => log::error!(
                                "received empty server settings response from the client"
                            ),
                        }
                    }
                }
            };
            false
        }
    };
    if fs_changed {
        log::info!("fs_state_changed = true, recompute diagnostics");
        let analysis = global_state.analysis();

        let mut files = vec![];
        files.extend(loop_state.opened_files.files());
        files.extend(analysis.db().module_files().keys());

        // filter duplicates
        let files = files
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let cloned_task_sender = resp_events_sender.clone();
        pool.execute(move || compute_file_diagnostics(analysis, cloned_task_sender, files));
    }
    Ok(())
}

fn on_request(
    global_state: &mut GlobalState,
    pool: &ThreadPool,
    task_sender: &Sender<ResponseEvent>,
    msg_sender: &Sender<Message>,
    req: Request,
) -> Result<()> {
    let mut pool_dispatcher =
        PoolDispatcher::new(req, pool, global_state, msg_sender, task_sender);
    pool_dispatcher
        .on::<req::Completion>(handlers::handle_completion)?
        .finish();
    Ok(())
}

fn diagnostic_as_string(d: &Diagnostic) -> String {
    format!(
        "({}, {}), ({}, {}): {}",
        d.range.start.line,
        d.range.start.character,
        d.range.end.line,
        d.range.end.character,
        d.message
    )
}

pub fn on_task(task: ResponseEvent, msg_sender: &Sender<Message>) {
    match task {
        ResponseEvent::Respond(response) => {
            msg_sender.send(response.into()).unwrap();
        }
        ResponseEvent::Diagnostic(file_diags) => {
            for file_diag in file_diags {
                let uri = Url::from_file_path(file_diag.fpath).unwrap();

                let mut diagnostics = vec![];
                if file_diag.diagnostic.is_some() {
                    diagnostics.push(file_diag.diagnostic.unwrap());
                }
                log::info!(
                    "Send diagnostic for file {:?}: {:#?}",
                    file_diag.fpath,
                    diagnostics
                        .iter()
                        .map(diagnostic_as_string)
                        .collect::<Vec<String>>()
                );

                let params = PublishDiagnosticsParams::new(uri, diagnostics, None);
                let notif = notification_new::<PublishDiagnostics>(params);
                msg_sender.send(notif.into()).unwrap();
            }
        }
    }
}

fn on_notification(
    msg_sender: &Sender<Message>,
    fs_events_sender: &Sender<FileSystemEvent>,
    loop_state: &mut LoopState,
    not: Notification,
) -> Result<()> {
    let not = match notification_cast::<DidOpenTextDocument>(not) {
        Ok(params) => {
            let uri = params.text_document.uri;
            let fpath = uri
                .to_file_path()
                .map_err(|_| anyhow::anyhow!("invalid uri: {}", uri))?;
            let file = (leaked_fpath(&fpath), params.text_document.text);
            fs_events_sender
                .send(FileSystemEvent::AddFile(file))
                .unwrap();
            loop_state.opened_files.add(leaked_fpath(fpath));
            return Ok(());
        }
        Err(not) => not,
    };
    let not = match notification_cast::<DidChangeTextDocument>(not) {
        Ok(mut params) => {
            let uri = params.text_document.uri;
            let fpath = uri
                .to_file_path()
                .map_err(|_| anyhow::anyhow!("invalid uri: {}", uri))?;
            let new_text = params
                .content_changes
                .pop()
                .ok_or_else(|| anyhow::anyhow!("empty changes".to_string()))?
                .text;
            let changed_file = (leaked_fpath(&fpath), new_text);
            fs_events_sender
                .send(FileSystemEvent::ChangeFile(changed_file))
                .unwrap();
            loop_state.opened_files.add(leaked_fpath(fpath));
            return Ok(());
        }
        Err(not) => not,
    };
    let not = match notification_cast::<DidCloseTextDocument>(not) {
        Ok(params) => {
            let uri = params.text_document.uri;
            let fpath = uri
                .to_file_path()
                .map_err(|_| anyhow::anyhow!("invalid uri: {}", uri))?;
            loop_state.opened_files.remove(leaked_fpath(fpath));
            return Ok(());
        }
        Err(not) => not,
    };
    let not = match notification_cast::<DidChangeConfiguration>(not) {
        Ok(_) => {
            // As stated in https://github.com/microsoft/language-server-protocol/issues/676,
            // this notification's parameters should be ignored and the actual config queried separately.
            let request_id = loop_state.next_request_id();
            let config_item = ConfigurationItem {
                section: Some("move".to_string()),
                scope_uri: None,
            };
            let request = request_new::<WorkspaceConfiguration>(
                request_id.clone(),
                ConfigurationParams {
                    items: vec![config_item],
                },
            );
            log::info!("Sending config request: {:?}", &request);
            msg_sender.send(request.into())?;
            loop_state.configuration_request_id = Some(request_id);

            return Ok(());
        }
        Err(not) => not,
    };
    let not = match notification_cast::<DidChangeWatchedFiles>(not) {
        Ok(params) => {
            for file_event in params.changes {
                let uri = file_event.uri;
                let fpath = uri
                    .to_file_path()
                    .map_err(|_| anyhow::anyhow!("invalid uri: {}", uri))?;
                let fpath = leaked_fpath(fpath);
                loop_state.opened_files.remove(fpath);
                fs_events_sender
                    .send(FileSystemEvent::RemoveFile(fpath))
                    .unwrap();
            }
            return Ok(());
        }
        Err(not) => not,
    };
    if not.method.starts_with("$/") {
        return Ok(());
    }
    Ok(())
}

pub fn compute_file_diagnostics(
    analysis: Analysis,
    task_sender: Sender<ResponseEvent>,
    files: Vec<MoveFilePath>,
) {
    log::info!("Computing diagnostics for files: {:#?}", files);
    let mut diagnostics = vec![];
    for fpath in files {
        // clear previous diagnostics for file
        diagnostics.push(FileDiagnostic::new_empty(fpath));

        let text = match analysis.db().available_files.get(fpath) {
            Some(text) => text,
            None => {
                log::warn!("Trying to check untracked file: {:?}", fpath);
                continue;
            }
        };
        if let Some(d) = analysis.check_file_with_compiler(fpath, text) {
            diagnostics.push(d);
        }
    }
    task_sender
        .send(ResponseEvent::Diagnostic(diagnostics))
        .unwrap();
}

pub fn notification_cast<N>(notification: Notification) -> Result<N::Params, Notification>
where
    N: lsp_types::notification::Notification,
    N::Params: DeserializeOwned,
{
    notification.extract(N::METHOD)
}

pub fn notification_new<N>(params: N::Params) -> Notification
where
    N: lsp_types::notification::Notification,
    N::Params: Serialize,
{
    Notification::new(N::METHOD.to_string(), params)
}

fn notification_is<N: lsp_types::notification::Notification>(
    notification: &Notification,
) -> bool {
    notification.method == N::METHOD
}

pub fn request_new<R>(id: RequestId, params: R::Params) -> Request
where
    R: lsp_types::request::Request,
    R::Params: Serialize,
{
    Request::new(id, R::METHOD.to_string(), params)
}

pub fn show_message(typ: MessageType, message: impl Into<String>, sender: &Sender<Message>) {
    let message = message.into();
    let params = ShowMessageParams { typ, message };
    let not = notification_new::<ShowMessage>(params);
    sender.send(not.into()).unwrap();
}
