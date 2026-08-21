use std::{fs::File, io::Write};

use shared_memory::{Shmem, ShmemConf};

pub struct Connection<T> {
    shmem: Shmem,
    _pd: core::marker::PhantomData<T>,
}

type Result<T> = std::result::Result<T, ConnectionError>;

#[non_exhaustive]
pub struct Server;
#[non_exhaustive]
pub struct Client;

impl<T> Connection<T> {
    pub fn new_server(endpoint: &str) -> Result<Connection<Server>> {
        let server_port = std::env::var_os("JESSIE_SERVER_PORT").unwrap_or("8080".into());
        let addr = format!("jessie_server::{server_port:#?}/server/{endpoint}");
        let shmem = match ShmemConf::new().flink(addr).size(2_000_000).open() {
            Ok(v) => v,
            _ => return Err(ConnectionError::FailedConnection),
        }; // Thats 2mb but surely that wont create any problems!
        let _pd = Default::default();
        Ok(Connection { shmem, _pd })
    }

    pub fn new_client(endpoint: &str) -> Result<Connection<Client>> {
        let server_port = std::env::var_os("JESSIE_SERVER_PORT").unwrap_or("8080".into());
        let addr = format!("jessie_server::{server_port:#?}/server/{endpoint}");
        let shmem = ShmemConf::new()
            .flink(addr)
            .size(100_000)
            .open()
            .map_err(|_| ConnectionError::FailedConnection)?;
        let _pd = Default::default();
        Ok(Connection { shmem, _pd })
    }

    fn as_file(&mut self) -> Result<File> {
        let path = self
            .shmem
            .get_flink_path()
            .ok_or(ConnectionError::FailedConnection)?;
        let file = File::create_new(path).map_err(|_| ConnectionError::CouldntCreateFile)?;
        Ok(file)
    }
}

impl Connection<Server> {
    pub fn write(&mut self, data: impl WriteableData<Server>) -> Result<()> {
        let mut file = self.as_file()?;
        let serialized = serde_json::to_vec(&data).map_err(|_| ConnectionError::WriteFailed)?;
        file.write(&serialized)
            .map_err(|_| ConnectionError::WriteFailed)?;
        Ok(())
    }
}

pub trait WriteableData<Type>: serde::Serialize {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionError {
    FailedConnection,
    CouldntCreateFile,
    WriteFailed,
}
