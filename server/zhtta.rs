//
// zhtta.rs
//
// Server for the drawsomething like multiplayer game
// Running on Rust 0.9
//
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu and David Evans
// Version 0.5

// To see debug! outputs set the RUST_LOG environment variable, e.g.: export RUST_LOG="zhtta=debug"

#[feature(globs)];
extern mod extra;

use std::io::*;
use std::io::net::ip::{SocketAddr};
use std::{os, str, libc, from_str};
use std::path::Path;
use std::hashmap::HashMap;
use std::rand::random;

use extra::getopts;
use extra::arc::MutexArc;

static SERVER_NAME : &'static str = "Zhtta Version 0.5";

static PENDING_PATH : &'static str = "/Users/zihaowang/Dropbox/Documents/UVa/Spring 2014/CS 4414/CS4414-Project/server/pending";

static ASCII_ART_PATH : &'static str = "/Users/zihaowang/Dropbox/Documents/UVa/Spring 2014/CS 4414/CS4414-Project/server/ascii_art";

static IP : &'static str = "127.0.0.1";
static PORT : uint = 4414;

static HTTP_OK : &'static str = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n";
static HTTP_BAD : &'static str = "HTTP/1.1 404 Not Found\r\n\r\n";

struct HTTP_Request {
    // Use peer_name as the key to access TcpStream in hashmap.

    // (Due to a bug in extra::arc in Rust 0.9, it is very inconvenient to use TcpStream without the "Freeze" bound.
    //  See issue: https://github.com/mozilla/rust/issues/12139)
    peer_name: ~str,
    path: ~Path,
}

struct WebServer {
    ip: ~str,
    port: uint,

    request_queue_arc: MutexArc<~[HTTP_Request]>,
    stream_map_arc: MutexArc<HashMap<~str, Option<std::io::net::tcp::TcpStream>>>,

    notify_port: Port<()>,
    shared_notify_chan: SharedChan<()>,

    user_map: MutexArc<HashMap<~str, ~str>>,
    question_map: MutexArc<HashMap<~str, bool>>,

    questions: MutexArc<~[~str]>,
}

impl WebServer {
    fn new(ip: &str, port: uint) -> WebServer {
        let (notify_port, shared_notify_chan) = SharedChan::new();
        let mut user_file_reader = File::open(&Path::new("users.txt")).expect("Invliad file");
        let content : ~str = user_file_reader.read_to_str().to_owned();
        let entries : ~[&str] = content.split('\n').collect();
        let mut user_map : HashMap<~str, ~str> = HashMap::new();
        for each_user in entries.iter() {
            let each_line : ~str = each_user.to_owned().to_owned();
            let user_tuple : ~[&str] = each_line.split(',').collect();
            if user_tuple.len() == 2 {
                user_map.insert(user_tuple[0].to_owned(), user_tuple[1].to_owned());
            }
        }
        let mut question_file_reader = File::open(&Path::new("questions.txt")).expect("Invalid file");
        let question_content : ~str = question_file_reader.read_to_str().to_owned();
        let questions : ~[&str] = question_content.split('\n').collect();
        let mut problem_map : HashMap<~str, bool> = HashMap::new();
        let mut problems : ~[~str] = ~[];
        for each_question in questions.iter() {
            let each_line : ~str = each_question.to_owned().to_owned();
            let question_tuple : ~[&str] = each_line.split(',').collect();
            if question_tuple.len() == 2 {
                match question_tuple[1].to_owned() {
                    ~"true" => problem_map.insert(question_tuple[0].to_owned(), true),
                    ~"false" => problem_map.insert(question_tuple[0].to_owned(), false),
                    _ => true,
                };
                problems.push(question_tuple[0].to_owned());
            }
        }
        WebServer {
            ip: ip.to_owned(),
            port: port,

            request_queue_arc: MutexArc::new(~[]),
            stream_map_arc: MutexArc::new(HashMap::new()),

            notify_port: notify_port,
            shared_notify_chan: shared_notify_chan,

            user_map: MutexArc::new(user_map),
            question_map: MutexArc::new(problem_map),

            questions: MutexArc::new(problems),
        }
    }

    fn run(&mut self) {
        self.listen();
        self.dequeue_static_file_request();
    }

    fn listen(&mut self) {
        let addr = from_str::<SocketAddr>(format!("{:s}:{:u}", self.ip, self.port)).expect("Address error.");

        let request_queue_arc = self.request_queue_arc.clone();
        let shared_notify_chan = self.shared_notify_chan.clone();
        let stream_map_arc = self.stream_map_arc.clone();
        let user_map_arc = self.user_map.clone();
        let question_map_arc = self.question_map.clone();
        let questions_arc = self.questions.clone();

        spawn(proc() {
            let mut acceptor = net::tcp::TcpListener::bind(addr).listen();
            println!("{:s} listening on {:s}.",
                     SERVER_NAME, addr.to_str());

            for stream in acceptor.incoming() {
                let (queue_port, queue_chan) = Chan::new();
                queue_chan.send(request_queue_arc.clone());

                let notify_chan = shared_notify_chan.clone();
                let stream_map_arc = stream_map_arc.clone();
                let user_map = user_map_arc.clone();
                let question_map = question_map_arc.clone();
                let questions = questions_arc.clone();

                // Spawn a task to handle the connection.
                spawn(proc() {
                    let request_queue_arc = queue_port.recv();

                    let mut stream = stream;

                    let mut buf = [0, ..700];
                    stream.read(buf);
                    let request_str = str::from_utf8(buf);
                    debug!("Request:\n{:s}", request_str);

                    let req_group : ~[&str]= request_str.splitn(' ', 3).collect();
                    if req_group.len() > 2 {
                        let path_str = "." + req_group[1].to_owned();

                        let mut path_obj = ~os::getcwd();
                        path_obj.push(path_str.clone());

                        let ext_str = match path_obj.extension_str() {
                            Some(e) => e,
                            None => "",
                        };

                        debug!("Requested path: [{:s}]", path_obj.as_str().expect("error"));
                        debug!("Requested path: [{:s}]", path_str);
                        debug!("ext_str is: {:s}", ext_str);

                        if path_str.starts_with("./login") {
                            debug!("=====in login branch====");
                            let args_vec : ~[&str] = path_str.slice_from(8).split('&').collect();
                            let mut username : &str = &"";
                            let mut password : &str = &"";
                            for each in args_vec.iter() {
                                let splitted_pair : ~[&str] = each.split('=').collect();
                                match splitted_pair[0] {
                                    "username" => username = splitted_pair[1],
                                    "password" => password = splitted_pair[1],
                                            _ => (),
                                };
                            }
                            debug!("username is {:s}", username);
                            debug!("password is {:s}", password);
                            WebServer::login_user(stream, username, password, user_map);
                        } else if path_str.starts_with("./retrieve") {
                            debug!("======retrieve brach=======");
                            let username : &str = path_str.slice_from(20);
                            WebServer::retrieve_question(stream, username);
                        } else if path_str.starts_with("./new_round") {
                            debug!("======generating new word====");
                            WebServer::generate_new_word(stream, questions);
                        } else if path_str.starts_with("./get_ascii_art") {
                            debug!("======generating ascii art=====");
                            let word : ~str = path_str.slice_from(20).to_owned().clone();
                            WebServer::generate_ascii_art(stream, question_map, word);
                        } else if path_str.starts_with("./check_user") {
                            debug!("======checking presence of user===");
                            let username : &str = path_str.slice_from(22);
                            WebServer::check_user_presence(stream, username, user_map);
                        } else if path_str.starts_with("./upload_question") {
                            debug!("=======uploading question========");
                            let args_vec : ~[&str] = path_str.slice_from(18).split('&').collect();
                            let mut recipient : &str = &"";
                            let mut sender : &str = &"";
                            let mut ascii_option : &str = &"";
                            let mut content : &str = &"";
                            let mut key_word : &str = &"";
                            for each in args_vec.iter() {
                                let splitted_pair : ~[&str] = each.split('=').collect();
                                match splitted_pair[0] {
                                    "recipient" => recipient = splitted_pair[1],
                                    "sender" => sender = splitted_pair[1],
                                    "ascii_option" => ascii_option = splitted_pair[1],
                                    "content" => content = splitted_pair[1],
                                    "word" => key_word = splitted_pair[1],
                                    _ => (),
                                };
                            }
                            debug!("===finished parsing arguments===");
                            WebServer::upload_question(stream, recipient, sender, key_word, ascii_option, content);
                        } else {
                            debug!("===== Static Page request =====");
                            WebServer::enqueue_static_file_request(stream, path_obj, stream_map_arc, request_queue_arc, notify_chan);
                        }
                    }
                });
            }
        });
    }

    fn login_user(stream: Option<std::io::net::tcp::TcpStream>, username: &str, password: &str, user_map: MutexArc<HashMap<~str, ~str>>) {
        let (username_port, username_chan) = Chan::new();
        let (password_port, password_chan) = Chan::new();
        let (stream_port, stream_chan) = Chan::new();
        username_chan.send(username.to_owned().clone());
        password_chan.send(password.to_owned().clone());
        stream_chan.send(stream);
        user_map.access(|local_user_map| {
            let received_name = username_port.recv();
            let received_pass = password_port.recv();
            let mut received_stream = stream_port.recv();
            match local_user_map.find(&received_name.to_owned()) {
                    Some(stored_pass) => if stored_pass.to_owned() == received_pass.to_owned() {
                        received_stream.write(HTTP_OK.as_bytes());
                    } else {
                        received_stream.write(HTTP_BAD.as_bytes());
                    },
                    None => received_stream.write(HTTP_BAD.as_bytes()),
                };
        });
    }

    fn retrieve_question(stream: Option<std::io::net::tcp::TcpStream>, username: &str) {
        let mut stream = stream;
        let file_path = &Path::new(PENDING_PATH + "/" + username.to_owned() + ".txt");
        match file_path.exists() {
            true => { let mut file_content = File::open(&Path::new(file_path));
                      stream.write(HTTP_OK.as_bytes());
                      stream.write(file_content.read_to_end());
            },
            false => { debug!("does not exist");
                       File::create(file_path);
                       stream.write(HTTP_BAD.as_bytes());
            },
        };
        let mut erase_file = File::open_mode(file_path, Truncate, Write);
        erase_file.write_str(&"");
    }

    fn generate_new_word(stream: Option<std::io::net::tcp::TcpStream>, questions: MutexArc<~[~str]>) {
        let (stream_port, stream_chan) = Chan::new();
        stream_chan.send(stream);
        questions.access(|question_list| {
            let mut received_stream = stream_port.recv();
            let list_length : f32 = question_list.len() as f32;
            let rand_fact = random::<f32>();
            let rand_index : uint = (list_length * rand_fact) as uint;
            let generated_word = question_list[rand_index].clone();
            received_stream.write(HTTP_OK.as_bytes());
            received_stream.write(generated_word.as_bytes());
        });
    }

    fn generate_ascii_art(stream: Option<std::io::net::tcp::TcpStream>, question_map: MutexArc<HashMap<~str, bool>>, word: ~str) {
        let mut stream = stream;
        let (word_port, word_chan) = Chan::new();
        word_chan.send(word.clone());
        let contains_art : bool = question_map.access(|local_question_map| {
            let received_word = word_port.recv();
            *local_question_map.get(&received_word.to_owned())
        });
        match contains_art {
            true => { let file_path = &Path::new(ASCII_ART_PATH + "/" + word.clone() + ".txt");
                      let mut file_content = File::open(&Path::new(file_path));
                      stream.write(HTTP_OK.as_bytes());
                      stream.write(file_content.read_to_end());
            },
            false => stream.write(HTTP_BAD.as_bytes()),
        };
    }

    fn check_user_presence(stream: Option<std::io::net::tcp::TcpStream>, username: &str, user_map: MutexArc<HashMap<~str, ~str>>) {
        let (stream_port, stream_chan) = Chan::new();
        let (username_port, username_chan) = Chan::new();
        username_chan.send(username.to_owned());
        stream_chan.send(stream);
        user_map.access(|local_user_map| {
            let received_username = username_port.recv();
            let mut received_stream = stream_port.recv();
            match local_user_map.find(&received_username.to_owned()) {
              Some(_) => received_stream.write(HTTP_OK.as_bytes()),
              None => received_stream.write(HTTP_BAD.as_bytes()),
            };
        });
    }

    fn upload_question(stream: Option<std::io::net::tcp::TcpStream>, recipient: &str, sender: &str, key_word: &str, ascii_option: &str, content: &str) {
        let mut stream = stream;
        let mut str_content : ~str = ~"sender: ";
        str_content = str_content.append(sender);
        str_content = str_content.append(&"\n");
        str_content = str_content.append(&"key: ");
        str_content = str_content.append(key_word);
        str_content = str_content.append(&"\n");
        debug!("content is {}", content);
        match ascii_option {
            "True" => str_content = str_content.append(&"ASCII\n"),
            "False" => { str_content = str_content.append(content);
                         str_content = str_content.append(&"\n");
                       },
            _ => (),
        };
        let file_path = &Path::new(PENDING_PATH + "/" + recipient.to_owned() + ".txt");
        debug!("{}", PENDING_PATH + "/" + recipient.to_owned() + ".txt");
        match file_path.exists() {
            true => { let mut file_content = File::open_mode(file_path, Append, ReadWrite);
                      file_content.write_str(str_content);
                      stream.write(HTTP_OK.as_bytes());
            },
            false => { debug!("false");
                       File::create(file_path);
                       let mut file_content = File::open_mode(file_path, Append, ReadWrite);
                       file_content.write_str(str_content);
                       stream.write(HTTP_OK.as_bytes());
            },
        };
    }

    // TODO: Streaming file.
    // TODO: Application-layer file caching.
    fn respond_with_static_file(stream: Option<std::io::net::tcp::TcpStream>, path: &Path) {
        let mut stream = stream;
        let mut file_reader = File::open(path).expect("Invalid file!");
        stream.write(HTTP_OK.as_bytes());
        stream.write(file_reader.read_to_end());
    }

    // TODO: Smarter Scheduling.
    fn enqueue_static_file_request(stream: Option<std::io::net::tcp::TcpStream>, path_obj: &Path, stream_map_arc: MutexArc<HashMap<~str, Option<std::io::net::tcp::TcpStream>>>, req_queue_arc: MutexArc<~[HTTP_Request]>, notify_chan: SharedChan<()>) {
        // Save stream in hashmap for later response.
        let mut stream = stream;
        let peer_name = WebServer::get_peer_name(&mut stream);
        let (stream_port, stream_chan) = Chan::new();
        stream_chan.send(stream);
        unsafe {
            // Use an unsafe method, because TcpStream in Rust 0.9 doesn't have "Freeze" bound.
            stream_map_arc.unsafe_access(|local_stream_map| {
                let stream = stream_port.recv();
                local_stream_map.swap(peer_name.clone(), stream);
            });
        }

        // Enqueue the HTTP request.
        let req = HTTP_Request { peer_name: peer_name.clone(), path: ~path_obj.clone() };
        let (req_port, req_chan) = Chan::new();
        req_chan.send(req);

        debug!("Waiting for queue mutex lock.");
        req_queue_arc.access(|local_req_queue| {
            debug!("Got queue mutex lock.");
            let req: HTTP_Request = req_port.recv();
            local_req_queue.push(req);
            debug!("A new request enqueued, now the length of queue is {:u}.", local_req_queue.len());
        });

        notify_chan.send(()); // Send incoming notification to responder task.


    }

    // TODO: Smarter Scheduling.
    fn dequeue_static_file_request(&mut self) {
        let req_queue_get = self.request_queue_arc.clone();
        let stream_map_get = self.stream_map_arc.clone();

        // Port<> cannot be sent to another task. So we have to make this task as the main task that can access self.notify_port.

        let (request_port, request_chan) = Chan::new();
        loop {
            self.notify_port.recv();    // waiting for new request enqueued.

            req_queue_get.access( |req_queue| {
                match req_queue.shift_opt() { // FIFO queue.
                    None => { /* do nothing */ }
                    Some(req) => {
                        request_chan.send(req);
                        debug!("A new request dequeued, now the length of queue is {:u}.", req_queue.len());
                    }
                }
            });

            let request = request_port.recv();

            // Get stream from hashmap.
            // Use unsafe method, because TcpStream in Rust 0.9 doesn't have "Freeze" bound.
            let (stream_port, stream_chan) = Chan::new();
            unsafe {
                stream_map_get.unsafe_access(|local_stream_map| {
                    let stream = local_stream_map.pop(&request.peer_name).expect("no option tcpstream");
                    stream_chan.send(stream);
                });
            }

            // TODO: Spawning more tasks to respond the dequeued requests concurrently. You may need a semophore to control the concurrency.
            let stream = stream_port.recv();
            WebServer::respond_with_static_file(stream, request.path);
            // Close stream automatically.
            debug!("=====Terminated connection from [{:s}].=====", request.peer_name);
        }
    }

    fn get_peer_name(stream: &mut Option<std::io::net::tcp::TcpStream>) -> ~str {
        match *stream {
            Some(ref mut s) => {
                         match s.peer_name() {
                            Some(pn) => {pn.to_str()},
                            None => (~"")
                         }
                       },
            None => (~"")
        }
    }
}

fn get_args() -> (~str, uint) {
    fn print_usage(program: &str) {
        println!("Usage: {:s} [options]", program);
        println!("--ip     \tIP address, \"{:s}\" by default.", IP);
        println!("--port   \tport number, \"{:u}\" by default.", PORT);
        println("-h --help \tUsage");
    }

    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();
    let program = args[0].clone();

    let opts = ~[
        getopts::optopt("ip"),
        getopts::optopt("port"),
        getopts::optflag("h"),
        getopts::optflag("help")
    ];

    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };

    if matches.opt_present("h") || matches.opt_present("help") {
        print_usage(program);
        unsafe { libc::exit(1); }
    }

    let ip_str = if matches.opt_present("ip") {
                    matches.opt_str("ip").expect("invalid ip address?").to_owned()
                 } else {
                    IP.to_owned()
                 };

    let port:uint = if matches.opt_present("port") {
                        from_str::from_str(matches.opt_str("port").expect("invalid port number?")).expect("not uint?")
                    } else {
                        PORT
                    };

    (ip_str, port)
}

fn main() {
    let (ip_str, port) = get_args();
    let mut zhtta = WebServer::new(ip_str, port);
    zhtta.run();
}
