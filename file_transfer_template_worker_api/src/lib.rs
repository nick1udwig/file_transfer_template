use crate::exports::kinode::process::file_transfer_template_worker::{
    DownloadRequest, Guest, Request as WorkerRequest, Response as WorkerResponse,
};
use crate::kinode::process::standard::Address as WitAddress;
use kinode_process_lib::{our_capabilities, spawn, Address, OnExit, Request, Response};

wit_bindgen::generate!({
    path: "target/wit",
    world: "file-transfer-template-worker-api-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

fn start_download(
    our: &WitAddress,
    source: &WitAddress,
    name: &str,
    target: &WitAddress,
    is_requestor: bool,
) -> anyhow::Result<()> {
    // spin up a worker, initialize based on whether it's a downloader or a sender.
    let our_worker = spawn(
        None,
        &format!(
            "{}:{}/pkg/file_transfer_template_worker.wasm",
            our.process.package_name, our.process.publisher_node,
        ),
        OnExit::None,
        our_capabilities(),
        vec![],
        false,
    )?;

    let target = if is_requestor { target } else { source };
    let our_worker_address = Address {
        node: our.node.clone(),
        process: our_worker,
    };

    Response::new()
        .body(WorkerResponse::Download(Ok(())))
        .send()?;

    Request::new()
        .expects_response(5)
        .body(WorkerRequest::Download(DownloadRequest {
            name: name.to_string(),
            target: target.clone(),
            is_requestor,
        }))
        .target(&our_worker_address)
        .send()?;

    Ok(())
}

struct Api;
impl Guest for Api {
    fn start_download(
        our: WitAddress,
        source: WitAddress,
        name: String,
        target: WitAddress,
        is_requestor: bool,
    ) -> Result<(), String> {
        match start_download(&our, &source, &name, &target, is_requestor) {
            Ok(result) => Ok(result),
            Err(e) => Err(format!("{e:?}")),
        }
    }
}
export!(Api);
