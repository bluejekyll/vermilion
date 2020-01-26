// Copyright 2019-2020 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::process::Stdio;

use async_trait::async_trait;
use clap::{App, ArgMatches, SubCommand};
use tokio::io::AsyncReadExt;

use crate::control::AsyncCtlEnd;
use crate::fork::StdIoConf;
use crate::msg::Message;
use crate::pipe::Read;
use crate::procs::Process;

/// Inter-process-communucation service.
///
/// Recv messages from the Leader and pass to supervisors...
///
/// Rules:
///  - may only receive message from the leader.
///  - may only deliver messages to the supervisors and launcher
///    - must validate message from leader
///    - never deliver messages to the leader except from launcer, pid's etc.
#[derive(Debug)]
pub struct Ipc;

#[async_trait]
impl Process<Read> for Ipc {
    const NAME: &'static str = "ipc";

    /// CLI SubCommand arguments
    fn sub_command() -> App<'static, 'static> {
        SubCommand::with_name(Self::NAME)
            .about("Inter-process communication for Vermilion processes")
        // .arg(
        //     Arg::with_name(procs::CONTROL_IN)
        //         .short("c")
        //         .long(procs::CONTROL_IN)
        //         .value_name("NUMBER")
        //         .validator_os(|i| {
        //             i32::from_str_radix(&i.to_string_lossy(), 10)
        //                 .map(|_| ())
        //                 .map_err(|_| OsString::from("number was expected"))
        //         })
        //         .help("control input filedescriptor (used when forking the process)")
        //         .takes_value(true),
        // )
    }

    /// This should be the ctl in port from the leader
    async fn run(mut control: AsyncCtlEnd<Read>, args: &ArgMatches<'_>) {
        println!("Ipc started");

        let mut message = Message::recv_msg(&mut control)
            .await
            .expect("no msg received");

        let fd = message.take_pipe().expect("take_pipe fails");
        println!("received filedescriptor: {:?}", fd);

        let mut reader = fd
            .into_async_pipe_end()
            .expect("could not make async pipe end");

        let mut buf = [0u8; 1024];
        let len = reader.read(&mut buf).await.expect("failed to read");
        let line = String::from_utf8_lossy(&buf[..len]);
        println!("LOG_LINE: {}", line);

        for i in 0..60 {
            println!("{} awaiting input: {}", Ipc::NAME, i);
            std::thread::sleep(std::time::Duration::from_secs(1))
        }
    }

    fn get_stdio() -> StdIoConf {
        StdIoConf {
            // we need a new input line
            stdin: Stdio::piped(),
            // StdErr and stdout will be piped to the logger
            stderr: Stdio::piped(),
            stdout: Stdio::piped(),
        }
    }
}
