use std::{
    env,
    io::{self, Write},
    net::{IpAddr, TcpStream},
    process,
    sync::mpsc::{channel, Sender},
    thread,
};

const MAX: u16 = 65535;

struct Arguments {
    ipaddr: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        match args.len() {
            2 if args[1] == "-h" || args[1] == "-help" => {
                println!(
                    "Usage:\n  <program> <IP>                      # default 4 threads\n  <program> -j <threads> <IP>         # custom thread count\n  <program> -h | -help                # show this help message"
                );
                Err("help")
            }
            2 => {
                // Single IP address, default to 4 threads
                let ipaddr = args[1].parse().map_err(|_| "Invalid IP address")?;
                Ok(Arguments { ipaddr, threads: 4 })
            }

            4 if args[1] == "-j" => {
                // Usage: -j <threads> <ip>
                let threads = args[2].parse().map_err(|_| "Invalid thread count")?;
                let ipaddr = args[3].parse().map_err(|_| "Invalid IP address")?;
                Ok(Arguments { ipaddr, threads })
            }

            _ => Err("Invalid arguments"),
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port = start_port + 1;
    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                println!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {}
        }
        if (MAX - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err.contains("help") {
            process::exit(0);
        } else {
            eprintln!("{} problem parsing arguements: {}", program, err);
            process::exit(0)
        }
    });
    let num_threads = arguments.threads;
    let addr = arguments.ipaddr;

    let (tx, rx) = channel();

    for i in 0..num_threads {
        let tx = tx.clone();
        thread::spawn(move || {
            scan(tx, i, addr, num_threads);
        });
    }

    // Drop the original sender so `rx` doesn't block forever
    drop(tx);

    let mut out = vec![];
    for p in rx {
        out.push(p);
    }

    println!();
    out.sort();
    for v in out {
        println!("{} is open", v);
    }
    println!("Oh My God!!")
}
