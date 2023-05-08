use crate::cli::args::Opts;
use crate::cli::input::{get_stdin_input, get_target_hosts, get_target_paths};
use crate::cli::logger::init_log;
use crate::lua::parsing::files::filename_to_string;
use crate::{show_msg, Lotus, MessageLevel, RequestOpts};
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use url::Url;
#[path = "input_handler.rs"]
mod input_handler;
use input_handler::custom_input_lua;

pub struct ScanArgs {
    pub target_data: TargetData,
    pub exit_after: i32,
    pub is_request: bool,
    pub req_opts: RequestOpts,
    pub lotus_obj: Lotus,
    pub requests_limit: i32,
    pub delay: u64,
    pub fuzz_workers: usize,
    pub verbose: bool,
}

pub struct TargetData {
    pub urls: Vec<String>,
    pub hosts: Vec<String>,
    pub paths: Vec<String>,
    pub custom: Vec<serde_json::Value>,
}

pub fn args_scan() -> ScanArgs {
    let (
        urls,
        hosts,
        paths,
        custom,
        exit_after,
        is_request,
        req_opts,
        lotus_obj,
        requests_limit,
        delay,
        fuzz_workers,
        verbose,
    ) = match Opts::from_args() {
        Opts::SCAN(url_opts) => {
            // setup logger
            init_log(url_opts.log).unwrap();
            let req_opts = RequestOpts {
                headers: url_opts.headers,
                proxy: url_opts.proxy,
                timeout: url_opts.timeout,
                redirects: url_opts.redirects,
            };
            let lotus_obj = Lotus {
                script_path: url_opts.script_path,
                output: url_opts.output,
                workers: url_opts.workers,
                script_workers: url_opts.scripts_workers,
                stop_after: Arc::new(Mutex::new(1)),
                env_vars: url_opts.env_vars,
            };
            let input_data = if let Some(urls_file) = url_opts.urls {
                let read_file = filename_to_string(urls_file.to_str().unwrap());
                if let Err(..) = read_file {
                    show_msg("Cannot Read the urls file", MessageLevel::Error);
                    std::process::exit(1);
                }
                read_file
                    .unwrap()
                    .lines()
                    .map(|line| line.to_string())
                    .collect::<Vec<String>>()
            } else {
                match get_stdin_input() {
                    Ok(input_data) => input_data,
                    Err(..) => {
                        show_msg("No input in Stdin", MessageLevel::Error);
                        std::process::exit(1);
                    }
                }
            };
            let input_handler = if let Some(custom_input) = url_opts.input_handler {
                let lua_code = filename_to_string(custom_input.to_str().unwrap());
                if let Err(err) = lua_code {
                    show_msg(
                        &format!("Unable to read custom input lua script: {}", err),
                        MessageLevel::Error,
                    );
                    vec![]
                } else {
                    let lua_output = custom_input_lua(input_data.clone(), &lua_code.unwrap());
                    if let Ok(lua_output) = lua_output {
                        lua_output
                    } else {
                        show_msg(&format!("{}", lua_output.unwrap_err()), MessageLevel::Error);
                        vec![]
                    }
                }
            } else {
                vec![]
            };
            log::debug!("{:?}", input_handler);
            let mut urls = vec![];
            let mut paths = vec![];
            let mut hosts = vec![];
            if input_handler.is_empty() {
                urls = input_data
                    .iter()
                    .filter_map(|target_url| Url::parse(target_url).ok())
                    .map(|url| url.to_string())
                    .collect();
                paths = match get_target_paths(urls.clone()) {
                    Ok(paths) => paths,
                    Err(err) => {
                        show_msg(
                            &format!("Failed to get target paths: {}", err),
                            MessageLevel::Error,
                        );
                        vec![]
                    }
                };
                hosts = get_target_hosts(urls.clone());
            }
            (
                urls,
                hosts,
                paths,
                input_handler,
                url_opts.exit_after,
                url_opts.is_request,
                req_opts,
                lotus_obj,
                url_opts.requests_limit,
                url_opts.delay,
                url_opts.fuzz_workers,
                url_opts.verbose,
            )
        }
        _ => {
            std::process::exit(1);
        }
    };

    ScanArgs {
        target_data: TargetData {
            urls,
            hosts,
            paths,
            custom,
        },
        exit_after,
        is_request,
        req_opts,
        lotus_obj,
        requests_limit,
        delay,
        fuzz_workers,
        verbose,
    }
}