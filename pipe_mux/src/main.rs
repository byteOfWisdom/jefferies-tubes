use crossbeam_channel::{bounded, Receiver, Sender};
use std::env;
use std::fs;
use std::io;
use std::io::{BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
use std::time::Duration;

const BUFFSIZE: usize = 2 * 8192; // for some reason bigger buffers don't work

type Data = [u8; BUFFSIZE];
type Rx = Receiver<(Data, usize)>;
type Tx = Sender<(Data, usize)>;

fn handle_client(mut stream: UnixStream, recv_from: Rx) {
    stream.set_nonblocking(true).unwrap();

    loop {
        match recv_from.try_recv() {
            Ok(inc) => match stream.write_all(&inc.0[0..inc.1]) {
                Ok(_) => {}
                Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => {
                    break;
                }
                Err(_) => {}
            },
            Err(_) => {} //most likely nothing in pipe
        }
    }

    println!("connection closed");
}

struct ChanMux {
    outputs: Vec<Tx>,
    input: Rx,
    terminate_recv: Receiver<()>,
    terminate_send: Sender<()>,
    run_handle: Option<thread::JoinHandle<()>>,
}

impl ChanMux {
    pub fn new(rx: Rx) -> Self {
        let (term_tx, term_rx) = bounded(0);
        return ChanMux {
            outputs: Vec::new(),
            terminate_recv: term_rx,
            terminate_send: term_tx,
            input: rx,
            run_handle: None,
        };
    }

    pub fn start(&mut self) {
        let instance_txs = self.outputs.clone();
        let instance_rx = self.input.clone();
        let instance_terminate = self.terminate_recv.clone();
        self.run_handle = Some(thread::spawn(move || {
            mux(instance_rx, instance_txs, instance_terminate)
        }));
    }

    pub fn new_rx(&mut self) -> Rx {
        let (tx, rx): (Tx, Rx) = bounded(3);
        self.outputs.push(tx);
        self.terminate_send.send(()).unwrap();
        self.start();
        return rx;
    }
}

fn mux(from: Rx, to: Vec<Tx>, stop: Receiver<()>) {
    let timeout = Duration::from_millis(100);
    while match stop.try_recv() {
        Ok(_) => false,
        Err(_) => true,
    } {
        match from.recv_timeout(timeout) {
            Ok(msg) => to.iter().for_each(|x| {
                let _ = x.try_send(msg);
            }),
            Err(_) => {}
        }
    }
}

fn publish_stdin(to: Tx) {
    let mut stdin = BufReader::new(io::stdin().lock());
    let mut m_buff: Data = [0; BUFFSIZE];

    loop {
        match stdin.read(&mut m_buff) {
            Ok(n) => {
                let _ = to.try_send((m_buff, n));
            }
            Err(_) => {}
        };
    }
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    let sock_name = match argv.len() {
        2 => String::from(argv[1].clone().trim()),
        _ => String::from(""),
    };

    let sock_path = format!("/tmp/donate_{}.sock", sock_name);
    match fs::remove_file(&sock_path) {
        Ok(_) => {}
        Err(_) => {}
    };

    let listener = UnixListener::bind(sock_path).unwrap();
    let (send_mux, recv_mux) = bounded(3); //unbounded();
    let mut _pub_stdin_handle = thread::spawn(move || publish_stdin(send_mux));

    let mut chan_muxer = ChanMux::new(recv_mux);
    chan_muxer.start();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("new connection!");
                let rcv_instance = chan_muxer.new_rx();
                thread::spawn(move || handle_client(stream, rcv_instance));
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}
