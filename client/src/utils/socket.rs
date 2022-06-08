use std::{
    f32::consts,
    io::{self, BufRead, Read, Write},
    net::TcpStream,
};

const MESSAGE_SIZE: usize = 64;

pub struct SocketReader {
    pub reader: io::BufReader<TcpStream>
}

impl SocketReader {
    pub fn new(stream: TcpStream) -> SocketReader {
        SocketReader {
            reader: io::BufReader::new(stream.try_clone().unwrap())
        }
    }

    pub fn receive_bytes(&mut self, n: usize) -> Option<Vec<u8>> {
        let mut n_received = 0;
        
        let mut v: Vec<u8> = Vec::new();

        loop {
            let received: Vec<u8> = self.reader.fill_buf().unwrap().to_vec();

            println!("n received: {}", n_received);

            v.extend(received.clone());

            n_received = n_received + received.len();
            self.reader.consume(received.len());

            if n_received == n {
                break;
            }
        }
        
        Some(v)
    }

    pub fn receive(&mut self) -> Option<String> {
        let mut mensaje = String::new();
        match self.reader.read_line(&mut mensaje) {
            Err(err) => {
                println!("{}", err);
                return None;
            }
            Ok(bytes) => {
                if bytes == 0 {
                    return None;
                } else {
                    mensaje.pop();
                    return Some(mensaje);
                }
            }
        }
    }
}

pub struct SocketWriter {
    pub writer: io::LineWriter<TcpStream>,
}

impl SocketWriter {
    pub fn new(stream: TcpStream) -> SocketWriter {
        SocketWriter {
            writer: io::LineWriter::new(stream),
        }
    }

    pub fn send(&mut self, mensaje: String) {
        if let Err(err) = self.writer.write_all((mensaje + "\n").as_bytes()) {
            panic!("{}", err);
        }
    }
}
