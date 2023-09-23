use std::fs;
use std::io::{Seek, Write};

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


