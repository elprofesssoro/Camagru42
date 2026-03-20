use std::{
	net::{TcpListener, TcpStream},
	io::{Result, BufReader, prelude::*},
	fs,
};

pub fn server() -> Result<()> 
{
	let listener = TcpListener::bind("127.0.0.1:80").unwrap();

	for stream in listener.incoming() 
	{
		handle_connection(stream?);
	}

	Ok(())
}


fn handle_connection(mut stream: TcpStream) 
{
	let buf_reader = BufReader::new(&stream);
	let http_request: Vec<_> = buf_reader
    .lines()
    .map(|result| result.unwrap())
    .take_while(|line| !line.is_empty())
    .collect();

    println!("Request: {http_request:#?}");
	let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("log.html").unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}