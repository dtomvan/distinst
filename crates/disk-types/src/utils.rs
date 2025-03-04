use std::fmt::Debug;
use std::path::Path;
use std::str::FromStr;
use std::{fs, io};

pub fn read_file<T: FromStr>(path: &Path) -> io::Result<T>
where
    <T as FromStr>::Err: Debug,
{
    fs::read_to_string(path)?
        .trim()
        .parse::<T>()
        .map_err(|why| io::Error::new(io::ErrorKind::Other, format!("{:?}", why)))
}
