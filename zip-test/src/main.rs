use std::fs;
use std::io::{Seek, Write};
use blake3::Hasher;
use base16384::Base16384Utf8;

pub struct FileMetaData {
    pub file: Vec<u8>,
    pub file_size: u64,
    pub file_blake3: String,
}

impl FileMetaData {
    pub fn as_string(&self) -> String {
        format!("|{:?}|file:{}|{}|by-shenjack", self.file.as_slice(), self.file_size, self.file_blake3)
    }

    pub fn new(file: Vec<u8>) -> Self {
        let file_size = file.len() as u64;

        let mut file_blake3 = Hasher::new();
        file_blake3.update(&file);
        let mut blake_data = file_blake3.finalize_xof();

        let mut buff = [0; 512];
        blake_data.fill(&mut buff);

        let blake3_str = Base16384Utf8::encode(&buff);
        Self {
            file,
            file_size,
            file_blake3: blake3_str,
        }
    }

    /// 从给定数据中解析出 文件+文件大小+文件blake3
    pub fn from_data(data: &[u8]) -> Option<Self> {
        let data = String::from_utf8_lossy(data);
        if !data.ends_with("|by-shenjack") {
            return None;
        }
        let data = data.trim_end_matches("|by-shenjack");
        let data: Vec<&str> = data.split('|').collect();
        // xxxxxxxx|xxxxxxxxx(data)|file:xxxxxx|xxxxxx  (|by-shenjack)
        if data.len() < 4 {
            return None;
        }
        let file = data[1].as_bytes().to_vec();
        let file_size = data[2].trim_start_matches("file:").parse::<u64>().unwrap();
        let file_blake3 = data[3].to_string();
        Some(Self {
            file,
            file_size,
            file_blake3,
        })
    }

    /// 向数据中添加文件数据
    pub fn add_data(&self, data: &[u8]) -> Vec<u8> {
        let file_data = self.as_string();
        let file_data = file_data.as_bytes();
        let mut new_data = Vec::with_capacity(data.len() + file_data.len());
        new_data.extend_from_slice(data);
        new_data.extend_from_slice(file_data);
        new_data
    }
}

fn main() {
    // 获取当前可执行文件的路径
    let exe_path = std::env::current_exe().unwrap();
    // 打开当前可执行文件
    let mut exe = fs::File::open(&exe_path).unwrap();

    println!("exe size: {}", exe.metadata().unwrap().len());

    // 将修改完的文件写入到一个新的文件中
    let exe_path = exe_path.with_file_name(format!("{}-new.exe", exe_path.file_name().unwrap().to_str().unwrap()));

    let mut new_exe = fs::File::create(exe_path).unwrap();
    exe.seek(std::io::SeekFrom::Start(0)).unwrap();
    std::io::copy(&mut exe, &mut new_exe).unwrap();

    new_exe.write_all(&[1; 100]).unwrap();

    new_exe.flush().unwrap();
    println!("new exe size: {}", new_exe.metadata().unwrap().len());
}


