use base16384::Base16384Utf8;
use blake3::Hasher;
use std::fs;
use std::io::{Seek, Write};

pub struct FileMetaData {
    /// 文件数据
    pub file: Vec<u8>,
    /// 文件大小
    pub file_size: u64,
    /// 文件 blake3
    pub file_blake3: String,
}

impl FileMetaData {
    pub fn as_string(&self) -> String {
        // format!(
        //     "\n|{}\n|file:{}\n|{}\n|by-shenjack",
        //     self.file,
        //     self.file_size,
        //     self.file_blake3
        // )
        // 将 file 以 Base16384 形式写入到文件中
        let file_data = Base16384Utf8::encode(&self.file);
        format!(
            "\n|{}\n|file:{}\n|{}\n|by-shenjack",
            file_data, self.file_size, self.file_blake3
        )
    }

    pub fn new(file: Vec<u8>) -> Self {
        let file_size = file.len() as u64;

        let mut file_blake3 = Hasher::new();
        file_blake3.update(&file);
        let mut blake_data = file_blake3.finalize_xof();

        let mut buff = [0; 256];
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
        if !data.ends_with("\n|by-shenjack") {
            return None;
        }
        let data = data.trim_end_matches("|by-shenjack");
        let data: Vec<&str> = data.split("\n|").collect();
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
    let exe_path = exe_path.with_file_name(format!(
        "{}-new.exe",
        exe_path.file_name().unwrap().to_str().unwrap()
    ));

    let mut new_exe = fs::File::create(exe_path).unwrap();
    std::io::copy(&mut exe, &mut new_exe).unwrap();

    // 将文件指针移动到文件末尾
    new_exe.seek(std::io::SeekFrom::End(0)).unwrap();

    let data = "Hello, World!".to_string();
    let file_meta_data = FileMetaData::new(data.as_bytes().to_vec());

    new_exe
        .write_all(file_meta_data.as_string().as_bytes())
        .unwrap();

    new_exe.flush().unwrap();
    println!("new exe size: {}", new_exe.metadata().unwrap().len());
}
