use std::{collections::HashMap, fs};

use clap::Parser;

use crate::{build_project, CmdExector, JsWorker, Request};

#[derive(Debug, Parser)]
pub struct RunOpts {}

impl CmdExector for RunOpts {
    async fn execute(&self) -> anyhow::Result<()> {
        let filename = build_project(".")?;
        let content = fs::read_to_string(&filename)?;
        let worker = JsWorker::try_new(&content)?;
        // TODO: normally this should run axum and let it load the worker
        let req = Request::builder()
            .method("GET")
            .url("http://example.com")
            .headers(HashMap::new())
            .build();
        let ret = worker.run("hello", req)?;
        println!("Response: {:?}", ret);
        Ok(())
    }
}
