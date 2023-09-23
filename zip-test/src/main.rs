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
    /// 用文件数据创建一个新的 FileMetaData
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

    pub fn as_vec(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.file.len() + 128);
        data.extend_from_slice(self.file.as_slice());
        let str_data = format!(
            "\n|len:{}\n|{}\n|by-shenjack",
            self.file_size, self.file_blake3
        );
        data.extend_from_slice(str_data.as_bytes());
        data
    }

    pub fn flake3(&self) -> String {
        self.file_blake3.clone()
    }

    /// 从给定数据中解析出 文件+文件大小+文件blake3
    pub fn from_data(data: &[u8]) -> Option<Self> {
        let string_data = String::from_utf8_lossy(data);
        if !string_data.ends_with("\n|by-shenjack") {
            println!("not end with |by-shenjack");
            return None;
        }
        let data = string_data.trim_end_matches("\n|by-shenjack");
        let mut data: Vec<&str> = data.rsplitn(3, "\n|").collect();
        data.reverse();
        // PE/ELFxxxxxxxxx(raw binary data)
        // |len:xxxxxx
        // |xxxxxflake3xxxx  (|by-shenjack)

        println!("data: {:?} len: {}", data, data.len());
        if data.len() != 3 {
            println!("data len != 3 {}", data.len());
            return None;
        }
        let file_size = data[1].trim_start_matches("len:").parse::<u64>().unwrap();
        let file_blake3 = data[2].to_string();
        // 从末尾取 file_size 长度的数据
        let file = data[0][data[0].len() - file_size as usize..]
            .as_bytes()
            .to_vec();
        Some(Self {
            file,
            file_size,
            file_blake3,
        })
    }

    /// 向数据中添加文件数据
    pub fn add_data(&self, data: &[u8]) -> Vec<u8> {
        let file_data = self.as_vec();
        let mut new_data = Vec::with_capacity(data.len() + file_data.len());
        new_data.extend_from_slice(data);
        new_data.extend(file_data);
        new_data
    }
}

fn main() {
    // 获取命令行参数
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        extract_file();
        return;
    }
    if args.len() >= 3 {
        println!(
            "zip-test {}",
            match args[1].as_str() {
                "add" => add_file(&args[2]),
                _ => extract_file(),
            }
        );
    } else {
        print_help();
    }
}

fn print_help() {
    println!("zip-test {:?}", std::env::args());
    println!("zip-test [add|extract]");
}

fn extract_file() -> bool {
    let exe_path = std::env::current_exe().unwrap();
    let meta_data = FileMetaData::from_data(fs::read(&exe_path).unwrap().as_slice());
    if meta_data.is_none() {
        println!("extract failed | No meta data");
        return false;
    }
    let meta_data = meta_data.unwrap();
    let mut extract_zip = fs::File::create("extract.zip").unwrap();
    extract_zip.write_all(meta_data.file.as_slice()).unwrap();
    if let Ok(()) = extract_zip.flush() {
        println!("extract success");
        true
    } else {
        println!("extract failed");
        false
    }
}

fn add_file(path: &String) -> bool {
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

    let data = fs::read(path).unwrap();
    let file_meta_data = FileMetaData::new(data);

    new_exe
        .write_all(file_meta_data.as_vec().as_slice())
        .unwrap();

    new_exe.flush().unwrap();
    println!("new exe size: {}", new_exe.metadata().unwrap().len());
    true
}
