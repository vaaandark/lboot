//! Used to parse the configuration file and get the boot entries.
//! 
//! In the configuration file, each entry should be like:
//! 
//! ```toml
//! [[entry]]
//! name = 'Linux 6.5.5'
//! vmlinux = 'efi\boot\bzImage.efi'
//! param = 'initrd=efi\boot\initramfs-linux.img'
//! ```
//! 
//! The use of more entries is also allowed.
//! 
//! **warning**: the field `vmlinux` must be an absolute path with
//! backslash as path separator.
//! What is currently implemented is only a subset of toml syntax
//! and does not support escape characters.

extern crate alloc;

use crate::error::{LbootError, Result};
use alloc::vec::Vec;
use core::fmt::Display;
use uefi::prelude::BootServices;
use uefi::proto::media::file::{File, FileAttribute, FileMode};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::SearchType;
use uefi::data_types::CStr16;
use uefi::Identify;

/// Since type [`CStr16`] is an unsized type, [`BoxedCStr16`] is used to
/// wrap its field `str` and the pointed buffer.
#[derive(Debug)]
pub struct BoxedCStr16<'a> {
	_vec: Vec<u16>,
	pub str: &'a CStr16,
}

impl<'a> BoxedCStr16<'a> {
	/// Create a [`BoxedCStr16`] with a **nul-terminated** vector.
	pub fn new(v: Vec<u16>) -> BoxedCStr16<'a> {
		let ptr = v.as_ptr() as *const uefi::Char16;
		unsafe {
			BoxedCStr16 {
				_vec: v,
				str: CStr16::from_ptr(ptr),
			}
		}
	}
}

impl<'a> Display for BoxedCStr16<'a> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", self.str)
	}
}

/// Entry Information when mutli-boot.
#[derive(Debug, Default)]
pub struct Entry<'a> {
	/// The name of the entry.
	pub name: Option<BoxedCStr16<'a>>,
	/// The path of the kernel boot excutable bzImage,
	/// which must be an absolute path with backslash as path separator.
	pub vmlinux: Option<BoxedCStr16<'a>>,
	/// Boot parameters passed to the kernel.
	pub param: Option<BoxedCStr16<'a>>,
}

impl<'a> Entry<'a> {
	fn print_option(
		s: &Option<BoxedCStr16<'a>>,
		f: &mut core::fmt::Formatter<'_>,
	) -> core::fmt::Result {
		if let Some(s) = s {
			write!(f, "{}", s)
		} else {
			write!(f, "Unknown")
		}
	}
}

impl<'a> Display for Entry<'a> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		Entry::print_option(&self.name, f)?;
		write!(f, "@")?;
		write!(f, "[")?;
		Entry::print_option(&self.vmlinux, f)?;
		write!(f, "] -- ")?;
		Entry::print_option(&self.param, f)?;
		Ok(())
	}
}

/// The configuration file loaded in memory.
#[derive(Debug)]
pub struct Config {
	contents: Vec<u8>,
}

const BUFF_SIZE: usize = 1024;

#[derive(Debug)]
enum LineType {
	Name(Vec<u16>),
	Vmlinux(Vec<u16>),
	Param(Vec<u16>),
	Blank,
	NewEntry,
	Unknown,
}

impl Config {
	/// Load configuration file into memory.
	pub fn load_from_file(bt: &BootServices, filename: &CStr16) -> Result<Config> {
		let simple_fs_handle = *bt
			.locate_handle_buffer(SearchType::ByProtocol(&SimpleFileSystem::GUID))?
			.first()
			.ok_or(LbootError::CannotOpenConfig)?;
		let mut simple_fs = bt
			.open_protocol_exclusive::<SimpleFileSystem>(simple_fs_handle)?;
		let mut root = simple_fs.open_volume()?;
		let mut file = root
			.open(filename, FileMode::Read, FileAttribute::empty())
			.map_err(|_| LbootError::CannotOpenConfig)?
			.into_regular_file()
			.ok_or(LbootError::CannotOpenConfig)?;
		let mut buff = [0; BUFF_SIZE];
		file.read(&mut buff)?;
		if let Some(end) = buff.iter().position(|&x| x == b'\0') {
			let buff = &buff[..end];
			let contents: Vec<u8> = Vec::from(buff);
			return Ok(Config { contents });
		}
		Err(LbootError::CannotOpenConfig)
	}

	fn trim(s: &[u8]) -> &[u8] {
		let len = s.len();
		let mut start = 0;
		let mut end = len;
		for i in s {
			if i.is_ascii_whitespace() {
				start += 1;
			} else {
				break;
			}
		}
		for i in s.iter().rev() {
			if i.is_ascii_whitespace() {
				end -= 1;
			} else {
				break;
			}
		}
		if start < end - 1
			&& ((s[start] == b'\'' && s[end - 1] == b'\'')
				|| (s[start] == b'"' && s[end - 1] == b'"'))
		{
			start += 1;
			end -= 1;
		}
		&s[start..end]
	}

	fn build_u16_vec(s: &[u8]) -> Vec<u16> {
		let mut result: Vec<u16> = s.iter().map(|x| *x as u16).collect();
		result.push(b'\0' as u16);
		result
	}

	fn parse_line(line: &[u8]) -> LineType {
		if line.is_empty() {
			return LineType::Blank;
		}
		if line == "[[entry]]".as_bytes() {
			return LineType::NewEntry;
		}
		if let Some(split) = line.iter().position(|x| *x == b'=') {
			let key = Config::trim(&line[..split]);
			let value = Config::build_u16_vec(Config::trim(&line[split + 1..]));
			if key.is_empty() || value.is_empty() {
				return LineType::Unknown;
			}
			return match key {
				b"name" => LineType::Name(value),
				b"vmlinux" => LineType::Vmlinux(value),
				b"param" => LineType::Param(value),
				_ => LineType::Unknown,
			};
		}
		LineType::Blank
	}

	/// Parse the loaded file and return boot entries.
	pub fn parse(&self) -> Result<Vec<Entry>> {
		let contents = &self.contents;
		let mut result: Vec<Entry<'_>> = Vec::new();
		let mut rest = 0;
		while let Some(len) = contents[rest..].iter().position(|x| *x == b'\n') {
			let line = &contents[rest..rest + len];
			let line_type = Config::parse_line(line);
            let last_entry = result.last_mut().ok_or(LbootError::WrongConfig);
			match line_type {
                LineType::Name(name) => last_entry?.name = Some(BoxedCStr16::new(name)),
                LineType::Vmlinux(vmlinux) => last_entry?.vmlinux = Some(BoxedCStr16::new(vmlinux)),
                LineType::Param(param) => last_entry?.param = Some(BoxedCStr16::new(param)),
				LineType::Blank => (),
				LineType::Unknown => return Err(LbootError::WrongConfig),
				LineType::NewEntry => {
					result.push(Entry::default());
				}
			}
			rest += len + 1;
		}
		Ok(result)
	}
}
