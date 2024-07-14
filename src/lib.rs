use std::{env, io::{Cursor, Read, Write}, net::{Ipv4Addr, SocketAddrV4, TcpStream}, os::unix::net::UnixStream, path::Path, sync::Mutex};

#[repr(u8)]
#[derive(Debug)]
pub enum PINEResult<T> {
    Ok(Vec<T>) = 0,
    Fail = 255
}

#[repr(u8)]
#[derive(Debug)]
pub enum PINECommand {
    MsgRead8 { mem: u32 } = 0,
    MsgRead16 { mem: u32 } = 1,
    MsgRead32 { mem: u32 } = 2,
    MsgRead64 { mem: u32 } = 3,
    MsgWrite8 { mem: u32, val: u8 } = 4,
    MsgWrite16 { mem: u32, val: u16 } = 5,
    MsgWrite32 { mem: u32, val: u32 } = 6,
    MsgWrite64 { mem: u32, val: u64 } = 7,
    MsgVersion = 8,
    MsgSaveState { sta: u8 } = 9,
    MsgLoadState { sta: u8 } = 10,
    MsgTitle = 11,
    MsgID = 12,
    MsgUUID = 13,
    MsgGameVersion = 14,
    MsgStatus = 15,
    MsgUnimplemented = 255
}

impl PINECommand {
    fn to_opcode(&self) -> u8 {
        match self {
            PINECommand::MsgRead8 { .. } => 0,
            PINECommand::MsgRead16 { .. } => 1,
            PINECommand::MsgRead32 { .. } => 2,
            PINECommand::MsgRead64 { .. } => 3,
            PINECommand::MsgWrite8 { .. } => 4,
            PINECommand::MsgWrite16 { .. } => 5,
            PINECommand::MsgWrite32 { .. } => 6,
            PINECommand::MsgWrite64 { .. } => 7,
            PINECommand::MsgVersion => 8,
            PINECommand::MsgSaveState { .. } => 9,
            PINECommand::MsgLoadState { .. } => 10,
            PINECommand::MsgTitle => 11,
            PINECommand::MsgID => 12,
            PINECommand::MsgUUID => 13,
            PINECommand::MsgGameVersion => 14,
            PINECommand::MsgStatus => 15,
            PINECommand::MsgUnimplemented => 255,
        }
    }
}

impl std::fmt::Display for PINECommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum PINEResponse {
    ResRead8 { val: u8 },
    ResRead16 { val: u16 },
    ResRead32 { val: u32 },
    ResRead64 { val: u64 },
    ResWrite8,
    ResWrite16,
    ResWrite32,
    ResWrite64,
    ResVersion { version: String },
    ResSaveState,
    ResLoadState,
    ResTitle { title: String },
    ResID { id: String },
    ResUUID { uuid: String },
    ResGameVersion { version: String },
    ResStatus { status: PINEStatus },
    ResUnimplemented
}

impl std::fmt::Display for PINEResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[repr(u32)]
#[derive(Debug)]
pub enum PINEStatus {
    Running = 0,
    Paused = 1,
    Shutdown = 2,
    Unknown
}

impl PINEStatus {
    fn from(i: u32) -> PINEStatus {
        match i {
            0 => PINEStatus::Running,
            1 => PINEStatus::Paused,
            2 => PINEStatus::Shutdown,
            _ => PINEStatus::Unknown
        }
    }
}

impl std::fmt::Display for PINEStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

enum PINESocket {
    Unix(UnixStream),
    Windows(TcpStream)
}

impl Read for PINESocket {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            PINESocket::Unix(s) => s.read(buf),
            PINESocket::Windows(s) => s.read(buf),
        }
    }
}

impl Write for PINESocket {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            PINESocket::Unix(s) => s.write(buf),
            PINESocket::Windows(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            PINESocket::Unix(s) => s.flush(),
            PINESocket::Windows(s) => s.flush(),
        }
    }
}

pub struct PINE {
    socket: PINESocket,
    mutex: Mutex<()>
}

impl PINE {
    pub fn connect(target: &str, slot: u16, auto: bool) -> Result<PINE, String> {
        match env::consts::OS {
            "linux" | "macos" => {
                let dir = match env::var(if env::consts::OS == "linux" { "XDG_RUNTIME_DIR" } else { "TMPDIR" }) {
                    Ok(x) => x,
                    Err(_) => "/tmp".to_string()
                };
                let filename = format!("{target}.sock{}", if auto { "".to_string() } else { format!(".{}", slot) });
                let path = Path::new(&dir).join(filename);
                if !path.exists() {
                    return Err(format!("Could not find Unix socket at {path:?}. Ensure PINE is enabled in the target emulator."));
                }

                let stream = match UnixStream::connect(path) {
                    Ok(x) => x,
                    Err(err) => return Err(format!("Could not connect to Unix socket: {err}")),
                };
                return Ok(PINE { socket: PINESocket::Unix(stream), mutex: Mutex::new(()) });
            },

            "windows" => {
                let addr = Ipv4Addr::new(127, 0, 0, 1);
                let socket = SocketAddrV4::new(addr, slot);
                let stream = match TcpStream::connect(socket) {
                    Ok(x) => x,
                    Err(err) => return Err(format!("Could not connect to TCP socket: {err}")),
                };
                return Ok(PINE { socket: PINESocket::Windows(stream), mutex: Mutex::new(()) });
            },

            _ => panic!("Unsupported operating system")
        };
    }

    pub fn connect_pcsx2(slot: Option<u16>) -> Result<PINE, String> {
        PINE::connect("pcsx2", match slot { Some(x) => x, None => 28011 }, slot.is_none())
    }

    pub fn connect_rpcs3(slot: Option<u16>) -> Result<PINE, String> {
        PINE::connect("rpcs3", match slot { Some(x) => x, None => 28012 }, slot.is_none())
    }

    pub fn connect_duckstation(slot: Option<u16>) -> Result<PINE, String> {
        PINE::connect("duckstation", match slot { Some(x) => x, None => 28011 }, slot.is_none())
    }

    pub fn send_raw(&mut self, buffer: &[u8]) -> Result<PINEResult<u8>, std::io::Error> {
        // Acquire lock
        let _unused = self.mutex.lock().unwrap();

        // Write buffer to socket
        self.socket.write_all(buffer)?;

        // Read response header
        let res_size = read_u32(&mut self.socket)?;
        let res_result = read_u8(&mut self.socket)?;
        if res_result != 0 {
            return Ok(PINEResult::Fail);
        }

        // Read buffer
        let mut res_buffer = vec![0; res_size as usize - 5];
        self.socket.read_exact(res_buffer.as_mut_slice())?;
        return Ok(PINEResult::Ok(res_buffer));
    }

    pub fn send(&mut self, batch: &mut PINEBatch) -> Result<PINEResult<PINEResponse>, std::io::Error> {
        let buffer = batch.finalize();
        let res = self.send_raw(buffer)?;
        let res_buffer = match res {
            PINEResult::Ok(x) => x,
            PINEResult::Fail => return Ok(PINEResult::Fail),
        };

        // Parse responses
        let mut res = Vec::<PINEResponse>::with_capacity(batch.commands.len());
        let reader = &mut Cursor::new(res_buffer);
        for command in batch.commands.iter() {
            res.push(match command {
                PINECommand::MsgRead8 { .. } => PINEResponse::ResRead8 { val: read_u8(reader)? },
                PINECommand::MsgRead16 { .. } => PINEResponse::ResRead16 { val: read_u16(reader)? },
                PINECommand::MsgRead32 { .. } => PINEResponse::ResRead32 { val: read_u32(reader)? },
                PINECommand::MsgRead64 { .. } => PINEResponse::ResRead64 { val: read_u64(reader)? },
                PINECommand::MsgWrite8 { .. } => PINEResponse::ResWrite8,
                PINECommand::MsgWrite16 { .. } => PINEResponse::ResWrite16,
                PINECommand::MsgWrite32 { .. } => PINEResponse::ResWrite32,
                PINECommand::MsgWrite64 { .. } => PINEResponse::ResWrite64,
                PINECommand::MsgVersion => PINEResponse::ResVersion { version: read_string(reader)? },
                PINECommand::MsgSaveState { .. } => PINEResponse::ResSaveState,
                PINECommand::MsgLoadState { .. } => PINEResponse::ResLoadState,
                PINECommand::MsgTitle => PINEResponse::ResTitle { title: read_string(reader)? },
                PINECommand::MsgID => PINEResponse::ResID { id: read_string(reader)? },
                PINECommand::MsgUUID => PINEResponse::ResUUID { uuid: read_string(reader)? },
                PINECommand::MsgGameVersion => PINEResponse::ResGameVersion { version: read_string(reader)? },
                PINECommand::MsgStatus => PINEResponse::ResStatus { status: PINEStatus::from(read_u32(reader)?) },
                PINECommand::MsgUnimplemented => PINEResponse::ResUnimplemented,
            });
        }
        
        Ok(PINEResult::Ok(res))
    }

    pub fn shutdown(self) -> Result<(), std::io::Error> {
        match self.socket {
            PINESocket::Unix(x) => x.shutdown(std::net::Shutdown::Both),
            PINESocket::Windows(x) => x.shutdown(std::net::Shutdown::Both),
        }
    }
}

pub struct PINEBatch {
    buffer: Vec<u8>,
    commands: Vec<PINECommand>
}

impl PINEBatch {
    pub fn new() -> Self {
        Self { buffer: vec![0x00, 0x00, 0x00, 0x00], commands: vec![] } // First 4 bytes are for the message length
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.commands.clear();
    }

    pub fn add(&mut self, command: PINECommand) {
        self.buffer.push(command.to_opcode());

        match command {
            PINECommand::MsgRead8 { mem } => self.buffer.extend_from_slice(&u32::to_le_bytes(mem)),
            PINECommand::MsgRead16 { mem } => self.buffer.extend_from_slice(&u32::to_le_bytes(mem)),
            PINECommand::MsgRead32 { mem } => self.buffer.extend_from_slice(&u32::to_le_bytes(mem)),
            PINECommand::MsgRead64 { mem } => self.buffer.extend_from_slice(&u32::to_le_bytes(mem)),
            PINECommand::MsgWrite8 { mem, val } => {
                self.buffer.extend_from_slice(&u32::to_le_bytes(mem));
                self.buffer.push(val);
            }
            PINECommand::MsgWrite16 { mem, val } => {
                self.buffer.extend_from_slice(&u32::to_le_bytes(mem));
                self.buffer.extend_from_slice(&u16::to_le_bytes(val));
            }
            PINECommand::MsgWrite32 { mem, val } => {
                self.buffer.extend_from_slice(&u32::to_le_bytes(mem));
                self.buffer.extend_from_slice(&u32::to_le_bytes(val));
            }
            PINECommand::MsgWrite64 { mem, val } => {
                self.buffer.extend_from_slice(&u32::to_le_bytes(mem));
                self.buffer.extend_from_slice(&u64::to_le_bytes(val));
            }
            PINECommand::MsgSaveState { sta } => self.buffer.push(sta),
            PINECommand::MsgLoadState { sta } => self.buffer.push(sta),
            _ => {}
        }

        self.commands.push(command);
    }

    fn finalize(&mut self) -> &[u8] {
        let size = self.buffer.len() as u32;
        self.buffer.splice(0..4, u32::to_le_bytes(size));
        return self.buffer.as_slice();
    }
}

impl Default for PINEBatch {
    fn default() -> Self {
        PINEBatch::new()
    }
}

macro_rules! read_impl {
    ($reader:ident, $ty:ident, $size:expr) => {
        let mut buf: [u8; $size] = [0; $size];
        $reader.read_exact(&mut buf)?;
        return Ok($ty::from_le_bytes(buf));
    };
}

fn read_u64<R: Read>(reader: &mut R) -> Result<u64, std::io::Error> { read_impl!(reader, u64, 8); }
fn read_u32<R: Read>(reader: &mut R) -> Result<u32, std::io::Error> { read_impl!(reader, u32, 4); }
fn read_u16<R: Read>(reader: &mut R) -> Result<u16, std::io::Error> { read_impl!(reader, u16, 2); }
fn read_u8 <R: Read>(reader: &mut R) -> Result<u8, std::io::Error>  { read_impl!(reader, u8,  1); }
fn read_string<R: Read>(reader: &mut R) -> Result<String, std::io::Error> {
    let size = read_u32(reader)?;
    let mut buffer: Vec<u8> = vec![0; size as usize];
    reader.read_exact(buffer.as_mut_slice())?;
    let mut s = std::str::from_utf8(&buffer).unwrap().to_string();
    s.pop(); // Remove null terminator
    return Ok(s);
}
